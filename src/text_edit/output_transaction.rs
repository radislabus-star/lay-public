use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) const OUTPUT_TRANSACTION_RECORD_BYTES: usize = 256;
pub(crate) const OUTPUT_TRANSACTION_SLOT_BYTES: usize = 4096;
pub(crate) const OUTPUT_TRANSACTION_QUEUE_CAPACITY: usize = 32;
pub(crate) const OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const OUTPUT_TRANSACTION_SLOT_CAPACITY: usize =
    OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES / OUTPUT_TRANSACTION_SLOT_BYTES;
const RECORD_MAGIC: [u8; 8] = *b"LAYOTX01";
const RECORD_SCHEMA: u16 = 1;
const CHECKSUM_OFFSET: usize = OUTPUT_TRANSACTION_RECORD_BYTES - 32;
const SLOT_MAGIC: [u8; 8] = *b"LAYOTS02";
const SLOT_SCHEMA: u16 = 2;
const SLOT_RECORD_COUNT_OFFSET: usize = 10;
const SLOT_SEQUENCE_OFFSET: usize = 16;
const SLOT_GENERATION_OFFSET: usize = 24;
const SLOT_PREVIOUS_DIGEST_OFFSET: usize = 56;
const SLOT_RECORDS_OFFSET: usize = 96;
const SLOT_CHECKSUM_OFFSET: usize = OUTPUT_TRANSACTION_SLOT_BYTES - 32;
const SLOT_MAX_RECORDS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutputTransactionRecordKindV1 {
    Prepared,
    Succeeded,
    NoEffect,
    RecoveryRequired,
}

impl OutputTransactionRecordKindV1 {
    const fn tag(self) -> u8 {
        match self {
            Self::Prepared => 1,
            Self::Succeeded => 2,
            Self::NoEffect => 3,
            Self::RecoveryRequired => 4,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, OutputTransactionErrorV1> {
        match tag {
            1 => Ok(Self::Prepared),
            2 => Ok(Self::Succeeded),
            3 => Ok(Self::NoEffect),
            4 => Ok(Self::RecoveryRequired),
            _ => Err(OutputTransactionErrorV1::CorruptRecord),
        }
    }

    const fn is_terminal(self) -> bool {
        !matches!(self, Self::Prepared)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutputTransactionIntentV1 {
    pub(crate) event_id: [u8; 32],
    pub(crate) lineage_id: [u8; 32],
    pub(crate) before_digest: [u8; 32],
    pub(crate) intended_after_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutputTransactionTerminalV1 {
    pub(crate) kind: OutputTransactionRecordKindV1,
    pub(crate) observed_after_digest: [u8; 32],
}

impl OutputTransactionTerminalV1 {
    pub(crate) fn new(
        kind: OutputTransactionRecordKindV1,
        observed_after_digest: [u8; 32],
    ) -> Result<Self, OutputTransactionErrorV1> {
        if !kind.is_terminal() {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        }
        Ok(Self {
            kind,
            observed_after_digest,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OutputTransactionRecordV1 {
    sequence: u64,
    kind: OutputTransactionRecordKindV1,
    event_id: [u8; 32],
    lineage_id: [u8; 32],
    before_digest: [u8; 32],
    intended_after_digest: [u8; 32],
    observed_after_digest: [u8; 32],
    previous_digest: [u8; 32],
}

impl OutputTransactionRecordV1 {
    fn encode(self) -> [u8; OUTPUT_TRANSACTION_RECORD_BYTES] {
        let mut bytes = [0_u8; OUTPUT_TRANSACTION_RECORD_BYTES];
        bytes[0..8].copy_from_slice(&RECORD_MAGIC);
        bytes[8..10].copy_from_slice(&RECORD_SCHEMA.to_le_bytes());
        bytes[10] = self.kind.tag();
        bytes[12..20].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[20..52].copy_from_slice(&self.event_id);
        bytes[52..84].copy_from_slice(&self.lineage_id);
        bytes[84..116].copy_from_slice(&self.before_digest);
        bytes[116..148].copy_from_slice(&self.intended_after_digest);
        bytes[148..180].copy_from_slice(&self.observed_after_digest);
        bytes[180..212].copy_from_slice(&self.previous_digest);
        let checksum = Sha256::digest(&bytes[..CHECKSUM_OFFSET]);
        bytes[CHECKSUM_OFFSET..].copy_from_slice(&checksum);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<(Self, [u8; 32]), OutputTransactionErrorV1> {
        if bytes.len() != OUTPUT_TRANSACTION_RECORD_BYTES
            || bytes[..8] != RECORD_MAGIC
            || u16::from_le_bytes([bytes[8], bytes[9]]) != RECORD_SCHEMA
            || bytes[11] != 0
            || bytes[212..CHECKSUM_OFFSET].iter().any(|byte| *byte != 0)
        {
            return Err(OutputTransactionErrorV1::CorruptRecord);
        }
        let expected = Sha256::digest(&bytes[..CHECKSUM_OFFSET]);
        if expected.as_slice() != &bytes[CHECKSUM_OFFSET..] {
            return Err(OutputTransactionErrorV1::ChecksumMismatch);
        }
        let mut checksum = [0_u8; 32];
        checksum.copy_from_slice(&bytes[CHECKSUM_OFFSET..]);
        let mut event_id = [0_u8; 32];
        event_id.copy_from_slice(&bytes[20..52]);
        let mut lineage_id = [0_u8; 32];
        lineage_id.copy_from_slice(&bytes[52..84]);
        let mut before_digest = [0_u8; 32];
        before_digest.copy_from_slice(&bytes[84..116]);
        let mut intended_after_digest = [0_u8; 32];
        intended_after_digest.copy_from_slice(&bytes[116..148]);
        let mut observed_after_digest = [0_u8; 32];
        observed_after_digest.copy_from_slice(&bytes[148..180]);
        let mut previous_digest = [0_u8; 32];
        previous_digest.copy_from_slice(&bytes[180..212]);
        Ok((
            Self {
                sequence: u64::from_le_bytes(bytes[12..20].try_into().unwrap()),
                kind: OutputTransactionRecordKindV1::from_tag(bytes[10])?,
                event_id,
                lineage_id,
                before_digest,
                intended_after_digest,
                observed_after_digest,
                previous_digest,
            },
            checksum,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutputTransactionStateV1 {
    Ready,
    Prepared,
    EffectInFlight,
    TerminalPending,
    RefusedBeforeEffect,
    RecoveryRequired,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutputTransactionErrorV1 {
    Io,
    CorruptRecord,
    ChecksumMismatch,
    ChainMismatch,
    SequenceMismatch,
    EventMismatch,
    LineageMismatch,
    InvalidTransition,
    QueueSaturated,
    OwnerStopped,
}

impl From<io::Error> for OutputTransactionErrorV1 {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct JournalSyncIoReceiptV1 {
    physical_write_calls: u64,
    physical_bytes: u64,
    direct_dsync: bool,
}

trait JournalIoV1: Send + 'static {
    fn append(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn sync_data(&mut self) -> io::Result<JournalSyncIoReceiptV1>;
}

#[repr(align(4096))]
struct AlignedJournalSlotV1([u8; OUTPUT_TRANSACTION_SLOT_BYTES]);

fn encode_direct_journal_slot(
    slot: &mut [u8; OUTPUT_TRANSACTION_SLOT_BYTES],
    sequence: u64,
    generation: [u8; 32],
    previous_slot_digest: [u8; 32],
    records: &[[u8; OUTPUT_TRANSACTION_RECORD_BYTES]],
) -> io::Result<[u8; 32]> {
    if records.is_empty() || records.len() > SLOT_MAX_RECORDS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "direct journal slot requires one or two logical records",
        ));
    }
    slot.fill(0);
    slot[..8].copy_from_slice(&SLOT_MAGIC);
    slot[8..10].copy_from_slice(&SLOT_SCHEMA.to_le_bytes());
    slot[SLOT_RECORD_COUNT_OFFSET] = records.len() as u8;
    slot[SLOT_SEQUENCE_OFFSET..SLOT_SEQUENCE_OFFSET + 8].copy_from_slice(&sequence.to_le_bytes());
    slot[SLOT_GENERATION_OFFSET..SLOT_GENERATION_OFFSET + 32].copy_from_slice(&generation);
    slot[SLOT_PREVIOUS_DIGEST_OFFSET..SLOT_PREVIOUS_DIGEST_OFFSET + 32]
        .copy_from_slice(&previous_slot_digest);
    for (index, record) in records.iter().enumerate() {
        let start = SLOT_RECORDS_OFFSET + index * OUTPUT_TRANSACTION_RECORD_BYTES;
        slot[start..start + OUTPUT_TRANSACTION_RECORD_BYTES].copy_from_slice(record);
    }
    let digest: [u8; 32] = Sha256::digest(&slot[..SLOT_CHECKSUM_OFFSET]).into();
    slot[SLOT_CHECKSUM_OFFSET..].copy_from_slice(&digest);
    Ok(digest)
}

struct DirectFileJournalIoV1 {
    file: File,
    generation: [u8; 32],
    previous_slot_digest: [u8; 32],
    slot_sequence: u64,
    pending_records: Vec<[u8; OUTPUT_TRANSACTION_RECORD_BYTES]>,
    slot: Box<AlignedJournalSlotV1>,
}

impl DirectFileJournalIoV1 {
    fn create(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_DIRECT | libc::O_DSYNC)
            .open(path)?;
        let allocated = unsafe {
            libc::fallocate(
                file.as_raw_fd(),
                0,
                0,
                OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES as i64,
            )
        };
        if allocated != 0 {
            return Err(io::Error::last_os_error());
        }
        file.sync_all()?;
        if let Some(parent) = path.parent() {
            File::open(parent)?.sync_all()?;
        }
        let mut generation_hasher = Sha256::new();
        generation_hasher.update(path.as_os_str().as_encoded_bytes());
        generation_hasher.update(std::process::id().to_le_bytes());
        generation_hasher.update(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_le_bytes(),
        );
        Ok(Self {
            file,
            generation: generation_hasher.finalize().into(),
            previous_slot_digest: [0; 32],
            slot_sequence: 0,
            pending_records: Vec::with_capacity(SLOT_MAX_RECORDS),
            slot: Box::new(AlignedJournalSlotV1([0; OUTPUT_TRANSACTION_SLOT_BYTES])),
        })
    }
}

impl JournalIoV1 for DirectFileJournalIoV1 {
    fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() != OUTPUT_TRANSACTION_RECORD_BYTES
            || self.pending_records.len() >= SLOT_MAX_RECORDS
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "direct journal durability unit accepts one or two exact records",
            ));
        }
        self.pending_records.push(bytes.try_into().unwrap());
        Ok(())
    }

    fn sync_data(&mut self) -> io::Result<JournalSyncIoReceiptV1> {
        if self.pending_records.is_empty() {
            return Ok(JournalSyncIoReceiptV1 {
                direct_dsync: true,
                ..JournalSyncIoReceiptV1::default()
            });
        }
        let digest = encode_direct_journal_slot(
            &mut self.slot.0,
            self.slot_sequence,
            self.generation,
            self.previous_slot_digest,
            &self.pending_records,
        )?;
        let slot_index = self.slot_sequence as usize % OUTPUT_TRANSACTION_SLOT_CAPACITY;
        let offset = (slot_index * OUTPUT_TRANSACTION_SLOT_BYTES) as libc::off_t;
        let written = unsafe {
            libc::pwrite(
                self.file.as_raw_fd(),
                self.slot.0.as_ptr().cast(),
                OUTPUT_TRANSACTION_SLOT_BYTES,
                offset,
            )
        };
        if written < 0 {
            return Err(io::Error::last_os_error());
        }
        if written as usize != OUTPUT_TRANSACTION_SLOT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "direct journal slot write was partial",
            ));
        }
        self.previous_slot_digest = digest;
        self.slot_sequence = self.slot_sequence.saturating_add(1);
        self.pending_records.clear();
        Ok(JournalSyncIoReceiptV1 {
            physical_write_calls: 1,
            physical_bytes: OUTPUT_TRANSACTION_SLOT_BYTES as u64,
            direct_dsync: true,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct OutputTransactionJournalStatsV1 {
    pub(crate) appended_records: u64,
    pub(crate) appended_bytes: u64,
    pub(crate) sync_calls: u64,
    pub(crate) sync_io_us: u64,
    pub(crate) physical_write_calls: u64,
    pub(crate) physical_bytes: u64,
    pub(crate) direct_dsync_units: u64,
    pub(crate) foreground_waits: u64,
    pub(crate) tail_flushes: u64,
    pub(crate) co_commits: u64,
    pub(crate) owner_nice: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct OutputTransactionFaultMatrixV1 {
    pub(crate) checksum_damage_rejected: bool,
    pub(crate) direct_slot_checksum_damage_rejected: bool,
    pub(crate) direct_slot_torn_write_rejected: bool,
    pub(crate) direct_slot_foreign_generation_rejected: bool,
    pub(crate) direct_slot_wrap_chain_preserved: bool,
    pub(crate) saturation_refused: bool,
    pub(crate) prepare_failure_refused_before_effect: bool,
    pub(crate) terminal_failure_requires_recovery: bool,
}

impl OutputTransactionFaultMatrixV1 {
    pub(crate) const fn passes(self) -> bool {
        self.checksum_damage_rejected
            && self.direct_slot_checksum_damage_rejected
            && self.direct_slot_torn_write_rejected
            && self.direct_slot_foreign_generation_rejected
            && self.direct_slot_wrap_chain_preserved
            && self.saturation_refused
            && self.prepare_failure_refused_before_effect
            && self.terminal_failure_requires_recovery
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PrepareCommitReceiptV1 {
    pub(crate) event_id: [u8; 32],
    pub(crate) foreground_waits: u8,
    pub(crate) sync_elapsed_us: u64,
    pub(crate) published_previous_terminal: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TerminalCommitReceiptV1 {
    pub(crate) published: bool,
    pub(crate) sync_elapsed_us: u64,
}

struct OrderedJournalV1<I: JournalIoV1> {
    io: I,
    state: OutputTransactionStateV1,
    sequence: u64,
    chain_digest: [u8; 32],
    active: Option<OutputTransactionIntentV1>,
    terminal_pending: bool,
    stats: OutputTransactionJournalStatsV1,
}

impl<I: JournalIoV1> OrderedJournalV1<I> {
    fn new(io: I) -> Self {
        Self {
            io,
            state: OutputTransactionStateV1::Ready,
            sequence: 0,
            chain_digest: [0; 32],
            active: None,
            terminal_pending: false,
            stats: OutputTransactionJournalStatsV1::default(),
        }
    }

    fn append_record(
        &mut self,
        kind: OutputTransactionRecordKindV1,
        intent: OutputTransactionIntentV1,
        observed_after_digest: [u8; 32],
    ) -> Result<(), OutputTransactionErrorV1> {
        let record = OutputTransactionRecordV1 {
            sequence: self.sequence,
            kind,
            event_id: intent.event_id,
            lineage_id: intent.lineage_id,
            before_digest: intent.before_digest,
            intended_after_digest: intent.intended_after_digest,
            observed_after_digest,
            previous_digest: self.chain_digest,
        };
        let bytes = record.encode();
        self.io.append(&bytes)?;
        self.chain_digest.copy_from_slice(&bytes[CHECKSUM_OFFSET..]);
        self.sequence = self.sequence.saturating_add(1);
        self.stats.appended_records = self.stats.appended_records.saturating_add(1);
        self.stats.appended_bytes = self
            .stats
            .appended_bytes
            .saturating_add(OUTPUT_TRANSACTION_RECORD_BYTES as u64);
        Ok(())
    }

    fn sync(&mut self, foreground: bool) -> Result<u64, OutputTransactionErrorV1> {
        let started = Instant::now();
        let io_receipt = self.io.sync_data()?;
        let elapsed_us = started.elapsed().as_micros() as u64;
        self.stats.sync_calls = self.stats.sync_calls.saturating_add(1);
        self.stats.sync_io_us = self.stats.sync_io_us.saturating_add(elapsed_us);
        self.stats.physical_write_calls = self
            .stats
            .physical_write_calls
            .saturating_add(io_receipt.physical_write_calls);
        self.stats.physical_bytes = self
            .stats
            .physical_bytes
            .saturating_add(io_receipt.physical_bytes);
        if io_receipt.direct_dsync {
            self.stats.direct_dsync_units = self.stats.direct_dsync_units.saturating_add(1);
        }
        if foreground {
            self.stats.foreground_waits = self.stats.foreground_waits.saturating_add(1);
        }
        Ok(elapsed_us)
    }

    fn prepare(
        &mut self,
        intent: OutputTransactionIntentV1,
    ) -> Result<PrepareCommitReceiptV1, OutputTransactionErrorV1> {
        if !matches!(
            self.state,
            OutputTransactionStateV1::Ready | OutputTransactionStateV1::TerminalPending
        ) {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        }
        let co_commit = self.terminal_pending;
        if let Some(active) = self.active {
            if active.lineage_id != intent.lineage_id {
                return Err(OutputTransactionErrorV1::LineageMismatch);
            }
        }
        if self
            .append_record(OutputTransactionRecordKindV1::Prepared, intent, [0; 32])
            .is_err()
        {
            self.state = if co_commit {
                OutputTransactionStateV1::RecoveryRequired
            } else {
                OutputTransactionStateV1::RefusedBeforeEffect
            };
            return Err(OutputTransactionErrorV1::Io);
        }
        let sync_elapsed_us = match self.sync(true) {
            Ok(elapsed) => elapsed,
            Err(error) => {
                self.state = if co_commit {
                    OutputTransactionStateV1::RecoveryRequired
                } else {
                    OutputTransactionStateV1::RefusedBeforeEffect
                };
                return Err(error);
            }
        };
        self.active = Some(intent);
        self.terminal_pending = false;
        self.state = OutputTransactionStateV1::Prepared;
        if co_commit {
            self.stats.co_commits = self.stats.co_commits.saturating_add(1);
        }
        Ok(PrepareCommitReceiptV1 {
            event_id: intent.event_id,
            foreground_waits: 1,
            sync_elapsed_us,
            published_previous_terminal: co_commit,
        })
    }

    fn begin_effect(&mut self, event_id: [u8; 32]) -> Result<(), OutputTransactionErrorV1> {
        if self.state != OutputTransactionStateV1::Prepared {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        }
        if self.active.map(|intent| intent.event_id) != Some(event_id) {
            return Err(OutputTransactionErrorV1::EventMismatch);
        }
        self.state = OutputTransactionStateV1::EffectInFlight;
        Ok(())
    }

    fn finish(
        &mut self,
        event_id: [u8; 32],
        terminal: OutputTransactionTerminalV1,
    ) -> Result<(), OutputTransactionErrorV1> {
        if self.state != OutputTransactionStateV1::EffectInFlight {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        }
        let Some(intent) = self.active else {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        };
        if intent.event_id != event_id {
            return Err(OutputTransactionErrorV1::EventMismatch);
        }
        if self
            .append_record(terminal.kind, intent, terminal.observed_after_digest)
            .is_err()
        {
            self.state = OutputTransactionStateV1::RecoveryRequired;
            return Err(OutputTransactionErrorV1::Io);
        }
        self.terminal_pending = true;
        self.state = OutputTransactionStateV1::TerminalPending;
        Ok(())
    }

    fn flush_terminal(
        &mut self,
        foreground: bool,
    ) -> Result<TerminalCommitReceiptV1, OutputTransactionErrorV1> {
        if self.state == OutputTransactionStateV1::Ready && !self.terminal_pending {
            return Ok(TerminalCommitReceiptV1 {
                published: false,
                sync_elapsed_us: 0,
            });
        }
        if self.state != OutputTransactionStateV1::TerminalPending || !self.terminal_pending {
            return Err(OutputTransactionErrorV1::InvalidTransition);
        }
        let sync_elapsed_us = match self.sync(foreground) {
            Ok(elapsed) => elapsed,
            Err(error) => {
                self.state = OutputTransactionStateV1::RecoveryRequired;
                return Err(error);
            }
        };
        self.stats.tail_flushes = self.stats.tail_flushes.saturating_add(1);
        self.terminal_pending = false;
        self.active = None;
        self.state = OutputTransactionStateV1::Ready;
        Ok(TerminalCommitReceiptV1 {
            published: true,
            sync_elapsed_us,
        })
    }

    fn stats(&self) -> OutputTransactionJournalStatsV1 {
        self.stats
    }

    fn state(&self) -> OutputTransactionStateV1 {
        self.state
    }
}

enum JournalCommandV1 {
    Prepare(
        OutputTransactionIntentV1,
        SyncSender<Result<PrepareCommitReceiptV1, OutputTransactionErrorV1>>,
    ),
    BeginEffect([u8; 32], SyncSender<Result<(), OutputTransactionErrorV1>>),
    Finish(
        [u8; 32],
        OutputTransactionTerminalV1,
        SyncSender<Result<(), OutputTransactionErrorV1>>,
    ),
    Flush(
        bool,
        SyncSender<Result<TerminalCommitReceiptV1, OutputTransactionErrorV1>>,
    ),
    Snapshot(SyncSender<(OutputTransactionStateV1, OutputTransactionJournalStatsV1)>),
    Stop(SyncSender<()>),
}

pub(crate) struct OrderedJournalOwnerV1 {
    commands: SyncSender<JournalCommandV1>,
    join: Option<JoinHandle<()>>,
}

impl OrderedJournalOwnerV1 {
    pub(crate) fn start(
        path: &Path,
        tail_flush_deadline: Duration,
    ) -> Result<Self, OutputTransactionErrorV1> {
        let io = DirectFileJournalIoV1::create(path)?;
        let (commands, receiver) = mpsc::sync_channel(OUTPUT_TRANSACTION_QUEUE_CAPACITY);
        let join = thread::Builder::new()
            .name("lay-output-journal".to_string())
            .spawn(move || run_owner(OrderedJournalV1::new(io), receiver, tail_flush_deadline))
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?;
        Ok(Self {
            commands,
            join: Some(join),
        })
    }

    fn enqueue(&self, command: JournalCommandV1) -> Result<(), OutputTransactionErrorV1> {
        match self.commands.try_send(command) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(OutputTransactionErrorV1::QueueSaturated),
            Err(TrySendError::Disconnected(_)) => Err(OutputTransactionErrorV1::OwnerStopped),
        }
    }

    pub(crate) fn prepare(
        &self,
        intent: OutputTransactionIntentV1,
    ) -> Result<PrepareCommitReceiptV1, OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Prepare(intent, tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?
    }

    pub(crate) fn begin_effect(&self, event_id: [u8; 32]) -> Result<(), OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::BeginEffect(event_id, tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?
    }

    pub(crate) fn finish(
        &self,
        event_id: [u8; 32],
        terminal: OutputTransactionTerminalV1,
    ) -> Result<(), OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Finish(event_id, terminal, tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?
    }

    pub(crate) fn flush_terminal(
        &self,
    ) -> Result<TerminalCommitReceiptV1, OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Flush(false, tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?
    }

    pub(crate) fn wait_before_native_state_change(
        &self,
    ) -> Result<TerminalCommitReceiptV1, OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Flush(true, tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Result<(OutputTransactionStateV1, OutputTransactionJournalStatsV1), OutputTransactionErrorV1>
    {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Snapshot(tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)
    }

    pub(crate) fn stop(mut self) -> Result<(), OutputTransactionErrorV1> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.enqueue(JournalCommandV1::Stop(tx))?;
        rx.recv()
            .map_err(|_| OutputTransactionErrorV1::OwnerStopped)?;
        if self.join.take().is_some_and(|join| join.join().is_err()) {
            return Err(OutputTransactionErrorV1::OwnerStopped);
        }
        Ok(())
    }
}

fn run_owner<I: JournalIoV1>(
    mut journal: OrderedJournalV1<I>,
    receiver: Receiver<JournalCommandV1>,
    tail_flush_deadline: Duration,
) {
    journal.stats.owner_nice = unsafe { libc::getpriority(libc::PRIO_PROCESS, 0) };
    loop {
        let command = if journal.state() == OutputTransactionStateV1::TerminalPending {
            match receiver.recv_timeout(tail_flush_deadline) {
                Ok(command) => command,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    let _ = journal.flush_terminal(false);
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(command) => command,
                Err(_) => break,
            }
        };
        match command {
            JournalCommandV1::Prepare(intent, reply) => {
                let _ = reply.send(journal.prepare(intent));
            }
            JournalCommandV1::BeginEffect(event_id, reply) => {
                let _ = reply.send(journal.begin_effect(event_id));
            }
            JournalCommandV1::Finish(event_id, terminal, reply) => {
                let _ = reply.send(journal.finish(event_id, terminal));
            }
            JournalCommandV1::Flush(foreground, reply) => {
                let _ = reply.send(journal.flush_terminal(foreground));
            }
            JournalCommandV1::Snapshot(reply) => {
                let _ = reply.send((journal.state(), journal.stats()));
            }
            JournalCommandV1::Stop(reply) => {
                if journal.state() == OutputTransactionStateV1::TerminalPending {
                    let _ = journal.flush_terminal(false);
                }
                journal.state = OutputTransactionStateV1::Closed;
                let _ = reply.send(());
                break;
            }
        }
    }
}

#[derive(Default)]
struct FaultJournalIoV1 {
    bytes: Vec<u8>,
    fail_append_at: Option<usize>,
    fail_sync_at: Option<usize>,
    append_calls: usize,
    sync_calls: usize,
}

impl JournalIoV1 for FaultJournalIoV1 {
    fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.append_calls += 1;
        if self.fail_append_at == Some(self.append_calls) {
            return Err(io::Error::other("injected append failure"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn sync_data(&mut self) -> io::Result<JournalSyncIoReceiptV1> {
        self.sync_calls += 1;
        if self.fail_sync_at == Some(self.sync_calls) {
            return Err(io::Error::other("injected sync failure"));
        }
        Ok(JournalSyncIoReceiptV1::default())
    }
}

pub(crate) fn run_output_transaction_fault_matrix() -> OutputTransactionFaultMatrixV1 {
    let intent = OutputTransactionIntentV1 {
        event_id: [1; 32],
        lineage_id: [2; 32],
        before_digest: [3; 32],
        intended_after_digest: [4; 32],
    };
    let record = OutputTransactionRecordV1 {
        sequence: 0,
        kind: OutputTransactionRecordKindV1::Prepared,
        event_id: intent.event_id,
        lineage_id: intent.lineage_id,
        before_digest: intent.before_digest,
        intended_after_digest: intent.intended_after_digest,
        observed_after_digest: [0; 32],
        previous_digest: [0; 32],
    };
    let mut damaged = record.encode();
    damaged[40] ^= 1;
    let checksum_damage_rejected = OutputTransactionRecordV1::decode(&damaged)
        == Err(OutputTransactionErrorV1::ChecksumMismatch);

    let encoded_record = record.encode();
    let record_digest: [u8; 32] = encoded_record[CHECKSUM_OFFSET..].try_into().unwrap();
    let mut direct_slot = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
    let direct_slot_digest =
        encode_direct_journal_slot(&mut direct_slot, 0, [7; 32], [0; 32], &[encoded_record])
            .expect("fixed direct slot encodes");
    let mut damaged_direct_slot = direct_slot;
    damaged_direct_slot[SLOT_RECORDS_OFFSET + 7] ^= 1;
    let direct_slot_checksum_damage_rejected = matches!(
        decode_direct_journal_slot(&damaged_direct_slot),
        Err(OutputTransactionErrorV1::ChecksumMismatch)
    );
    let direct_slot_torn_write_rejected = matches!(
        decode_direct_journal_slot(&direct_slot[..OUTPUT_TRANSACTION_SLOT_BYTES / 2]),
        Err(OutputTransactionErrorV1::CorruptRecord)
    );

    let mut next_record = record;
    next_record.sequence = 1;
    next_record.previous_digest = record_digest;
    let mut foreign_slot = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
    encode_direct_journal_slot(
        &mut foreign_slot,
        1,
        [8; 32],
        direct_slot_digest,
        &[next_record.encode()],
    )
    .expect("foreign-generation slot encodes");
    let direct_slot_foreign_generation_rejected = matches!(
        validate_decoded_slot_chain(vec![
            decode_direct_journal_slot(&direct_slot).unwrap().unwrap(),
            decode_direct_journal_slot(&foreign_slot).unwrap().unwrap(),
        ]),
        Err(OutputTransactionErrorV1::ChainMismatch)
    );

    let mut wrapped_record = record;
    wrapped_record.sequence = OUTPUT_TRANSACTION_SLOT_CAPACITY as u64;
    wrapped_record.previous_digest = [5; 32];
    let wrapped_record_bytes = wrapped_record.encode();
    let wrapped_record_digest: [u8; 32] =
        wrapped_record_bytes[CHECKSUM_OFFSET..].try_into().unwrap();
    let mut wrapped_first = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
    let wrapped_first_digest = encode_direct_journal_slot(
        &mut wrapped_first,
        OUTPUT_TRANSACTION_SLOT_CAPACITY as u64,
        [9; 32],
        [6; 32],
        &[wrapped_record_bytes],
    )
    .expect("wrapped first slot encodes");
    let mut wrapped_next_record = record;
    wrapped_next_record.sequence = OUTPUT_TRANSACTION_SLOT_CAPACITY as u64 + 1;
    wrapped_next_record.previous_digest = wrapped_record_digest;
    let mut wrapped_second = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
    encode_direct_journal_slot(
        &mut wrapped_second,
        OUTPUT_TRANSACTION_SLOT_CAPACITY as u64 + 1,
        [9; 32],
        wrapped_first_digest,
        &[wrapped_next_record.encode()],
    )
    .expect("wrapped second slot encodes");
    let direct_slot_wrap_chain_preserved = validate_decoded_slot_chain(vec![
        decode_direct_journal_slot(&wrapped_first).unwrap().unwrap(),
        decode_direct_journal_slot(&wrapped_second)
            .unwrap()
            .unwrap(),
    ]) == Ok(2);

    let (queue_tx, _queue_rx) = mpsc::sync_channel(OUTPUT_TRANSACTION_QUEUE_CAPACITY);
    for value in 0..OUTPUT_TRANSACTION_QUEUE_CAPACITY {
        let _ = queue_tx.try_send(value);
    }
    let saturation_refused = matches!(
        queue_tx.try_send(OUTPUT_TRANSACTION_QUEUE_CAPACITY),
        Err(TrySendError::Full(_))
    );

    let mut prepare_failure = OrderedJournalV1::new(FaultJournalIoV1 {
        fail_sync_at: Some(1),
        ..FaultJournalIoV1::default()
    });
    let prepare_failure_refused_before_effect = prepare_failure.prepare(intent).is_err()
        && prepare_failure.state() == OutputTransactionStateV1::RefusedBeforeEffect
        && prepare_failure.begin_effect(intent.event_id).is_err();

    let mut terminal_failure = OrderedJournalV1::new(FaultJournalIoV1 {
        fail_append_at: Some(2),
        ..FaultJournalIoV1::default()
    });
    let terminal = OutputTransactionTerminalV1::new(
        OutputTransactionRecordKindV1::Succeeded,
        intent.intended_after_digest,
    )
    .expect("Succeeded is terminal");
    let terminal_failure_requires_recovery = terminal_failure.prepare(intent).is_ok()
        && terminal_failure.begin_effect(intent.event_id).is_ok()
        && terminal_failure.finish(intent.event_id, terminal).is_err()
        && terminal_failure.state() == OutputTransactionStateV1::RecoveryRequired;

    OutputTransactionFaultMatrixV1 {
        checksum_damage_rejected,
        direct_slot_checksum_damage_rejected,
        direct_slot_torn_write_rejected,
        direct_slot_foreign_generation_rejected,
        direct_slot_wrap_chain_preserved,
        saturation_refused,
        prepare_failure_refused_before_effect,
        terminal_failure_requires_recovery,
    }
}

#[derive(Debug)]
struct DecodedJournalSlotV1 {
    sequence: u64,
    generation: [u8; 32],
    previous_digest: [u8; 32],
    digest: [u8; 32],
    records: Vec<(OutputTransactionRecordV1, [u8; 32])>,
}

fn decode_direct_journal_slot(
    bytes: &[u8],
) -> Result<Option<DecodedJournalSlotV1>, OutputTransactionErrorV1> {
    if bytes.len() != OUTPUT_TRANSACTION_SLOT_BYTES {
        return Err(OutputTransactionErrorV1::CorruptRecord);
    }
    if bytes.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    let record_count = bytes[SLOT_RECORD_COUNT_OFFSET] as usize;
    if bytes[..8] != SLOT_MAGIC
        || u16::from_le_bytes([bytes[8], bytes[9]]) != SLOT_SCHEMA
        || !(1..=SLOT_MAX_RECORDS).contains(&record_count)
        || bytes[11..SLOT_SEQUENCE_OFFSET]
            .iter()
            .any(|byte| *byte != 0)
        || bytes[88..SLOT_RECORDS_OFFSET].iter().any(|byte| *byte != 0)
    {
        return Err(OutputTransactionErrorV1::CorruptRecord);
    }
    let records_end = SLOT_RECORDS_OFFSET + record_count * OUTPUT_TRANSACTION_RECORD_BYTES;
    if bytes[records_end..SLOT_CHECKSUM_OFFSET]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(OutputTransactionErrorV1::CorruptRecord);
    }
    let expected: [u8; 32] = Sha256::digest(&bytes[..SLOT_CHECKSUM_OFFSET]).into();
    if bytes[SLOT_CHECKSUM_OFFSET..] != expected {
        return Err(OutputTransactionErrorV1::ChecksumMismatch);
    }
    let mut generation = [0_u8; 32];
    generation.copy_from_slice(&bytes[SLOT_GENERATION_OFFSET..SLOT_GENERATION_OFFSET + 32]);
    let mut previous_digest = [0_u8; 32];
    previous_digest
        .copy_from_slice(&bytes[SLOT_PREVIOUS_DIGEST_OFFSET..SLOT_PREVIOUS_DIGEST_OFFSET + 32]);
    let mut records = Vec::with_capacity(record_count);
    for index in 0..record_count {
        let start = SLOT_RECORDS_OFFSET + index * OUTPUT_TRANSACTION_RECORD_BYTES;
        records.push(OutputTransactionRecordV1::decode(
            &bytes[start..start + OUTPUT_TRANSACTION_RECORD_BYTES],
        )?);
    }
    Ok(Some(DecodedJournalSlotV1 {
        sequence: u64::from_le_bytes(
            bytes[SLOT_SEQUENCE_OFFSET..SLOT_SEQUENCE_OFFSET + 8]
                .try_into()
                .unwrap(),
        ),
        generation,
        previous_digest,
        digest: expected,
        records,
    }))
}

fn validate_decoded_slot_chain(
    mut slots: Vec<DecodedJournalSlotV1>,
) -> Result<usize, OutputTransactionErrorV1> {
    if slots.is_empty() {
        return Ok(0);
    }
    let generation = slots[0].generation;
    if slots.iter().any(|slot| slot.generation != generation) {
        return Err(OutputTransactionErrorV1::ChainMismatch);
    }
    slots.sort_unstable_by_key(|slot| slot.sequence);
    let mut record_count = 0_usize;
    let mut previous_slot_digest = None;
    let mut previous_record_digest = None;
    let mut previous_record_sequence: Option<u64> = None;
    for (index, slot) in slots.iter().enumerate() {
        if index > 0 && slot.sequence != slots[index - 1].sequence.saturating_add(1) {
            return Err(OutputTransactionErrorV1::SequenceMismatch);
        }
        if let Some(previous) = previous_slot_digest {
            if slot.previous_digest != previous {
                return Err(OutputTransactionErrorV1::ChainMismatch);
            }
        } else if slot.sequence == 0 && slot.previous_digest != [0; 32] {
            return Err(OutputTransactionErrorV1::ChainMismatch);
        }
        for (record, digest) in &slot.records {
            if let Some(previous_sequence) = previous_record_sequence {
                if record.sequence != previous_sequence.saturating_add(1) {
                    return Err(OutputTransactionErrorV1::SequenceMismatch);
                }
            } else if slot.sequence == 0 && record.sequence != 0 {
                return Err(OutputTransactionErrorV1::SequenceMismatch);
            }
            if let Some(previous) = previous_record_digest {
                if record.previous_digest != previous {
                    return Err(OutputTransactionErrorV1::ChainMismatch);
                }
            } else if slot.sequence == 0 && record.previous_digest != [0; 32] {
                return Err(OutputTransactionErrorV1::ChainMismatch);
            }
            previous_record_sequence = Some(record.sequence);
            previous_record_digest = Some(*digest);
            record_count += 1;
        }
        previous_slot_digest = Some(slot.digest);
    }
    Ok(record_count)
}

pub(crate) fn scan_output_transaction_journal(
    path: &Path,
) -> Result<usize, OutputTransactionErrorV1> {
    let mut bytes = Vec::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    if bytes.len() != OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES {
        return Err(OutputTransactionErrorV1::CorruptRecord);
    }
    let mut slots = Vec::new();
    for chunk in bytes.chunks_exact(OUTPUT_TRANSACTION_SLOT_BYTES) {
        if let Some(slot) = decode_direct_journal_slot(chunk)? {
            slots.push(slot);
        }
    }
    validate_decoded_slot_chain(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryIoV1 {
        bytes: Vec<u8>,
        fail_append_at: Option<usize>,
        fail_sync_at: Option<usize>,
        append_calls: usize,
        sync_calls: usize,
    }

    impl JournalIoV1 for MemoryIoV1 {
        fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
            self.append_calls += 1;
            if self.fail_append_at == Some(self.append_calls) {
                return Err(io::Error::other("injected append failure"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(())
        }

        fn sync_data(&mut self) -> io::Result<JournalSyncIoReceiptV1> {
            self.sync_calls += 1;
            if self.fail_sync_at == Some(self.sync_calls) {
                return Err(io::Error::other("injected sync failure"));
            }
            Ok(JournalSyncIoReceiptV1::default())
        }
    }

    fn digest(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn intent(value: u8) -> OutputTransactionIntentV1 {
        OutputTransactionIntentV1 {
            event_id: digest(value),
            lineage_id: digest(9),
            before_digest: digest(value.wrapping_add(1)),
            intended_after_digest: digest(value.wrapping_add(2)),
        }
    }

    fn success(value: u8) -> OutputTransactionTerminalV1 {
        OutputTransactionTerminalV1::new(
            OutputTransactionRecordKindV1::Succeeded,
            digest(value.wrapping_add(2)),
        )
        .unwrap()
    }

    #[test]
    fn fixed_record_round_trips_and_rejects_checksum_damage() {
        let record = OutputTransactionRecordV1 {
            sequence: 7,
            kind: OutputTransactionRecordKindV1::Prepared,
            event_id: digest(1),
            lineage_id: digest(2),
            before_digest: digest(3),
            intended_after_digest: digest(4),
            observed_after_digest: digest(5),
            previous_digest: digest(6),
        };
        let mut bytes = record.encode();
        assert_eq!(OutputTransactionRecordV1::decode(&bytes).unwrap().0, record);
        bytes[40] ^= 1;
        assert_eq!(
            OutputTransactionRecordV1::decode(&bytes),
            Err(OutputTransactionErrorV1::ChecksumMismatch)
        );
    }

    #[test]
    fn direct_slot_round_trips_and_rejects_torn_or_foreign_chain() {
        let first = OutputTransactionRecordV1 {
            sequence: 0,
            kind: OutputTransactionRecordKindV1::Prepared,
            event_id: digest(1),
            lineage_id: digest(2),
            before_digest: digest(3),
            intended_after_digest: digest(4),
            observed_after_digest: digest(5),
            previous_digest: [0; 32],
        };
        let first_bytes = first.encode();
        let first_record_digest = first_bytes[CHECKSUM_OFFSET..].try_into().unwrap();
        let mut first_slot = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
        let first_slot_digest =
            encode_direct_journal_slot(&mut first_slot, 0, digest(7), [0; 32], &[first_bytes])
                .unwrap();
        let decoded_first = decode_direct_journal_slot(&first_slot).unwrap().unwrap();
        assert_eq!(decoded_first.sequence, 0);
        assert_eq!(decoded_first.records[0].0, first);
        assert!(matches!(
            decode_direct_journal_slot(&first_slot[..2048]),
            Err(OutputTransactionErrorV1::CorruptRecord)
        ));

        let mut second = first;
        second.sequence = 1;
        second.previous_digest = first_record_digest;
        let mut second_slot = [0_u8; OUTPUT_TRANSACTION_SLOT_BYTES];
        encode_direct_journal_slot(
            &mut second_slot,
            1,
            digest(7),
            first_slot_digest,
            &[second.encode()],
        )
        .unwrap();
        assert_eq!(
            validate_decoded_slot_chain(vec![
                decoded_first,
                decode_direct_journal_slot(&second_slot).unwrap().unwrap(),
            ]),
            Ok(2)
        );

        second_slot[SLOT_GENERATION_OFFSET] ^= 1;
        let checksum: [u8; 32] = Sha256::digest(&second_slot[..SLOT_CHECKSUM_OFFSET]).into();
        second_slot[SLOT_CHECKSUM_OFFSET..].copy_from_slice(&checksum);
        assert_eq!(
            validate_decoded_slot_chain(vec![
                decode_direct_journal_slot(&first_slot).unwrap().unwrap(),
                decode_direct_journal_slot(&second_slot).unwrap().unwrap(),
            ]),
            Err(OutputTransactionErrorV1::ChainMismatch)
        );
    }

    #[test]
    fn co_commit_uses_one_foreground_barrier_and_releases_previous_terminal() {
        let mut journal = OrderedJournalV1::new(MemoryIoV1::default());
        let first = intent(1);
        let second = intent(2);
        assert_eq!(journal.prepare(first).unwrap().foreground_waits, 1);
        journal.begin_effect(first.event_id).unwrap();
        journal.finish(first.event_id, success(1)).unwrap();
        let receipt = journal.prepare(second).unwrap();
        assert!(receipt.published_previous_terminal);
        assert_eq!(receipt.foreground_waits, 1);
        assert_eq!(journal.stats().sync_calls, 2);
        assert_eq!(journal.stats().co_commits, 1);
    }

    #[test]
    fn prepare_failure_refuses_before_effect() {
        let io = MemoryIoV1 {
            fail_sync_at: Some(1),
            ..MemoryIoV1::default()
        };
        let mut journal = OrderedJournalV1::new(io);
        assert_eq!(
            journal.prepare(intent(1)),
            Err(OutputTransactionErrorV1::Io)
        );
        assert_eq!(
            journal.state(),
            OutputTransactionStateV1::RefusedBeforeEffect
        );
        assert_eq!(
            journal.begin_effect(digest(1)),
            Err(OutputTransactionErrorV1::InvalidTransition)
        );
    }

    #[test]
    fn terminal_failure_requires_recovery() {
        let io = MemoryIoV1 {
            fail_append_at: Some(2),
            ..MemoryIoV1::default()
        };
        let mut journal = OrderedJournalV1::new(io);
        let event = intent(1);
        journal.prepare(event).unwrap();
        journal.begin_effect(event.event_id).unwrap();
        assert_eq!(
            journal.finish(event.event_id, success(1)),
            Err(OutputTransactionErrorV1::Io)
        );
        assert_eq!(journal.state(), OutputTransactionStateV1::RecoveryRequired);
    }

    #[test]
    fn terminal_is_not_published_before_tail_flush() {
        let mut journal = OrderedJournalV1::new(MemoryIoV1::default());
        let event = intent(1);
        journal.prepare(event).unwrap();
        journal.begin_effect(event.event_id).unwrap();
        journal.finish(event.event_id, success(1)).unwrap();
        assert_eq!(journal.state(), OutputTransactionStateV1::TerminalPending);
        let receipt = journal.flush_terminal(false).unwrap();
        assert!(receipt.published);
        assert_eq!(journal.state(), OutputTransactionStateV1::Ready);
    }

    #[test]
    fn bounded_channel_refuses_saturation() {
        let (tx, _rx) = mpsc::sync_channel(OUTPUT_TRANSACTION_QUEUE_CAPACITY);
        for value in 0..OUTPUT_TRANSACTION_QUEUE_CAPACITY {
            tx.try_send(value).unwrap();
        }
        assert!(matches!(tx.try_send(99), Err(TrySendError::Full(99))));
    }

    #[test]
    fn executable_fault_matrix_is_fail_closed() {
        assert!(run_output_transaction_fault_matrix().passes());
    }
}
