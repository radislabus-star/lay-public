use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Mutex;

use super::events::{
    decode_verified_spool_record, SpoolRecordV1, TypedEventSpoolManifestV1, TypedEventSpoolShardV1,
    VerifiedSpoolShardReaderV1, VerifiedSpoolShardWriterV1,
};

const SPOOL_RECORD_ACCOUNTING_BYTES: usize = 56;

#[derive(Clone, Debug)]
pub(super) struct ExternalSpoolSortConfigV1 {
    pub(super) root: PathBuf,
    pub(super) maximum_buffer_bytes: usize,
    pub(super) maximum_open_runs: usize,
    pub(super) write_buffer_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SortedTypedEventSpoolManifestV1 {
    pub(super) schema_version: u16,
    pub(super) split_seed: u64,
    pub(super) compiler_version: u32,
    pub(super) normalization_version: u32,
    pub(super) shards: Vec<TypedEventSpoolShardV1>,
    pub(super) input_records: u64,
    pub(super) unique_records: u64,
    pub(super) initial_runs: u64,
    pub(super) merge_passes: u32,
}

#[derive(Clone, Debug)]
struct SortableSpoolRecordV1 {
    record: SpoolRecordV1,
    primary_identity: Vec<u8>,
}

impl SortableSpoolRecordV1 {
    fn decode(record: SpoolRecordV1, split_seed: u64) -> Result<Self, String> {
        let event = decode_verified_spool_record(&record, split_seed)?;
        let lemma = event.lemma();
        if lemma.language.as_bytes().contains(&0) || lemma.normalized_lemma.as_bytes().contains(&0)
        {
            return Err("productive lexical sort identity contains NUL".to_string());
        }
        let mut primary_identity =
            Vec::with_capacity(lemma.language.len() + 1 + lemma.normalized_lemma.len());
        primary_identity.extend_from_slice(lemma.language.as_bytes());
        primary_identity.push(0);
        primary_identity.extend_from_slice(lemma.normalized_lemma.as_bytes());
        Ok(Self {
            primary_identity,
            record,
        })
    }

    fn accounted_bytes(&self) -> usize {
        SPOOL_RECORD_ACCOUNTING_BYTES
            .saturating_add(self.primary_identity.len())
            .saturating_add(self.record.canonical_event_bytes.len())
    }
}

pub(super) fn external_sort_verified_spool(
    manifest: &TypedEventSpoolManifestV1,
    config: &ExternalSpoolSortConfigV1,
) -> Result<SortedTypedEventSpoolManifestV1, String> {
    external_sort_verified_spool_with_workers(manifest, config, 1)
}

pub(super) fn external_sort_verified_spool_with_workers(
    manifest: &TypedEventSpoolManifestV1,
    config: &ExternalSpoolSortConfigV1,
    workers: usize,
) -> Result<SortedTypedEventSpoolManifestV1, String> {
    if config.maximum_buffer_bytes < SPOOL_RECORD_ACCOUNTING_BYTES
        || config.maximum_open_runs < 2
        || config.write_buffer_bytes < SPOOL_RECORD_ACCOUNTING_BYTES
        || workers == 0
    {
        return Err(
            "productive external sort budget cannot hold its bounded machinery".to_string(),
        );
    }
    fs::create_dir_all(&config.root).map_err(|error| error.to_string())?;
    let results = if manifest.shards.len() <= 1 || workers == 1 {
        manifest
            .shards
            .iter()
            .enumerate()
            .map(|(shard_index, shard)| {
                sort_one_shard(shard_index, shard, manifest.split_seed, config)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let next = AtomicUsize::new(0);
        let results = Mutex::new(
            (0..manifest.shards.len())
                .map(|_| None)
                .collect::<Vec<Option<Result<SortedShardResultV1, String>>>>(),
        );
        std::thread::scope(|scope| {
            for _ in 0..workers.min(manifest.shards.len()) {
                scope.spawn(|| loop {
                    let shard_index = next.fetch_add(1, AtomicOrdering::Relaxed);
                    let Some(shard) = manifest.shards.get(shard_index) else {
                        break;
                    };
                    let result = sort_one_shard(shard_index, shard, manifest.split_seed, config);
                    results.lock().expect("productive sort result lock")[shard_index] =
                        Some(result);
                });
            }
        });
        results
            .into_inner()
            .map_err(|_| "productive sort result lock poisoned".to_string())?
            .into_iter()
            .map(|result| {
                result.ok_or_else(|| "productive shard sort produced no result".to_string())?
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut output_shards = Vec::with_capacity(manifest.shards.len());
    let mut total_input = 0_u64;
    let mut total_unique = 0_u64;
    let mut total_initial_runs = 0_u64;
    let mut maximum_merge_passes = 0_u32;
    for result in results {
        total_input = total_input
            .checked_add(result.input_records)
            .ok_or_else(|| "productive external sort input count overflow".to_string())?;
        total_unique = total_unique
            .checked_add(result.unique_records)
            .ok_or_else(|| "productive external sort unique count overflow".to_string())?;
        total_initial_runs = total_initial_runs
            .checked_add(result.initial_runs)
            .ok_or_else(|| "productive external sort run count overflow".to_string())?;
        maximum_merge_passes = maximum_merge_passes.max(result.merge_passes);
        output_shards.push(TypedEventSpoolShardV1 {
            path: result.path,
            record_count: result.unique_records,
        });
    }
    let globally_sorted_path = config.root.join("sorted-events-global.p2s");
    let shard_paths = output_shards
        .iter()
        .map(|shard| shard.path.clone())
        .collect::<Vec<_>>();
    let (global_count, global_passes) = merge_all_runs(
        manifest.shards.len(),
        manifest.split_seed,
        &config.root,
        shard_paths,
        &globally_sorted_path,
        config.maximum_open_runs,
        config.write_buffer_bytes,
    )?;
    if global_count != total_unique {
        return Err("productive global spool merge changed the unique denominator".to_string());
    }
    maximum_merge_passes = maximum_merge_passes.saturating_add(global_passes);
    Ok(SortedTypedEventSpoolManifestV1 {
        schema_version: manifest.schema_version,
        split_seed: manifest.split_seed,
        compiler_version: manifest.compiler_version,
        normalization_version: manifest.normalization_version,
        shards: vec![TypedEventSpoolShardV1 {
            path: globally_sorted_path,
            record_count: global_count,
        }],
        input_records: total_input,
        unique_records: total_unique,
        initial_runs: total_initial_runs,
        merge_passes: maximum_merge_passes,
    })
}

struct SortedShardResultV1 {
    path: PathBuf,
    input_records: u64,
    unique_records: u64,
    initial_runs: u64,
    merge_passes: u32,
}

fn sort_one_shard(
    shard_index: usize,
    shard: &TypedEventSpoolShardV1,
    split_seed: u64,
    config: &ExternalSpoolSortConfigV1,
) -> Result<SortedShardResultV1, String> {
    let mut reader = VerifiedSpoolShardReaderV1::open(&shard.path)?;
    let mut chunk = Vec::<SortableSpoolRecordV1>::new();
    let mut chunk_bytes = 0_usize;
    let mut run_paths = Vec::new();
    let mut input_records = 0_u64;
    while let Some(record) = reader.next_record()? {
        input_records = input_records
            .checked_add(1)
            .ok_or_else(|| "productive external sort shard count overflow".to_string())?;
        let sortable = SortableSpoolRecordV1::decode(record, split_seed)?;
        let accounted = sortable.accounted_bytes();
        if !chunk.is_empty()
            && chunk_bytes
                .checked_add(accounted)
                .is_none_or(|bytes| bytes > config.maximum_buffer_bytes)
        {
            run_paths.push(write_initial_run(
                shard_index,
                run_paths.len(),
                &config.root,
                &mut chunk,
                config.write_buffer_bytes,
            )?);
            chunk_bytes = 0;
        }
        chunk_bytes = chunk_bytes.saturating_add(accounted);
        chunk.push(sortable);
    }
    if input_records != shard.record_count {
        return Err("productive external sort shard count disagrees with manifest".to_string());
    }
    if !chunk.is_empty() {
        run_paths.push(write_initial_run(
            shard_index,
            run_paths.len(),
            &config.root,
            &mut chunk,
            config.write_buffer_bytes,
        )?);
    }
    let initial_runs = run_paths.len() as u64;
    let final_path = config
        .root
        .join(format!("sorted-events-{shard_index:05}.p2s"));
    let (unique_records, merge_passes) = merge_all_runs(
        shard_index,
        split_seed,
        &config.root,
        run_paths,
        &final_path,
        config.maximum_open_runs,
        config.write_buffer_bytes,
    )?;
    Ok(SortedShardResultV1 {
        path: final_path,
        input_records,
        unique_records,
        initial_runs,
        merge_passes,
    })
}

fn write_initial_run(
    shard_index: usize,
    run_index: usize,
    root: &Path,
    chunk: &mut Vec<SortableSpoolRecordV1>,
    write_buffer_bytes: usize,
) -> Result<PathBuf, String> {
    chunk.sort_by(sort_order);
    chunk.dedup_by(same_full_event);
    let path = root.join(format!("sort-{shard_index:05}-run-{run_index:08}.p2s"));
    let mut writer = VerifiedSpoolShardWriterV1::create(&path, write_buffer_bytes)?;
    for record in chunk.iter() {
        writer.append(&record.record)?;
    }
    writer.finish()?;
    chunk.clear();
    Ok(path)
}

fn merge_all_runs(
    shard_index: usize,
    split_seed: u64,
    root: &Path,
    mut runs: Vec<PathBuf>,
    final_path: &Path,
    maximum_open_runs: usize,
    write_buffer_bytes: usize,
) -> Result<(u64, u32), String> {
    if runs.is_empty() {
        return VerifiedSpoolShardWriterV1::create(final_path, write_buffer_bytes)?
            .finish()
            .map(|count| (count, 0));
    }
    let mut pass = 0_u32;
    while runs.len() > 1 {
        let mut next = Vec::new();
        for (group_index, group) in runs.chunks(maximum_open_runs).enumerate() {
            let path = root.join(format!(
                "sort-{shard_index:05}-pass-{pass:04}-group-{group_index:08}.p2s"
            ));
            merge_run_group(group, &path, split_seed, write_buffer_bytes)?;
            next.push(path);
        }
        for path in &runs {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
        runs = next;
        pass = pass
            .checked_add(1)
            .ok_or_else(|| "productive external sort merge pass overflow".to_string())?;
    }
    if final_path.exists() {
        fs::remove_file(final_path).map_err(|error| error.to_string())?;
    }
    fs::rename(&runs[0], final_path).map_err(|error| error.to_string())?;
    let count = count_verified_records(final_path)?;
    Ok((count, pass))
}

fn merge_run_group(
    input_paths: &[PathBuf],
    output_path: &Path,
    split_seed: u64,
    write_buffer_bytes: usize,
) -> Result<u64, String> {
    let mut readers = input_paths
        .iter()
        .map(|path| VerifiedSpoolShardReaderV1::open(path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut heads = readers
        .iter_mut()
        .map(|reader| {
            reader
                .next_record()?
                .map(|record| SortableSpoolRecordV1::decode(record, split_seed))
                .transpose()
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut writer = VerifiedSpoolShardWriterV1::create(output_path, write_buffer_bytes)?;
    let mut previous: Option<([u8; 32], Vec<u8>)> = None;
    while let Some(selected) = heads
        .iter()
        .enumerate()
        .filter_map(|(index, head)| head.as_ref().map(|head| (index, head)))
        .min_by(|(_, left), (_, right)| sort_order(left, right))
        .map(|(index, _)| index)
    {
        let head = heads[selected].take().expect("selected merge head");
        let duplicate = previous.as_ref().is_some_and(|(hash, bytes)| {
            *hash == head.record.event_sha256 && *bytes == head.record.canonical_event_bytes
        });
        if !duplicate {
            writer.append(&head.record)?;
            previous = Some((
                head.record.event_sha256,
                head.record.canonical_event_bytes.clone(),
            ));
        }
        heads[selected] = readers[selected]
            .next_record()?
            .map(|record| SortableSpoolRecordV1::decode(record, split_seed))
            .transpose()?;
    }
    writer.finish()
}

fn count_verified_records(path: &Path) -> Result<u64, String> {
    let mut reader = VerifiedSpoolShardReaderV1::open(path)?;
    let mut count = 0_u64;
    while reader.next_record()?.is_some() {
        count = count
            .checked_add(1)
            .ok_or_else(|| "productive sorted spool count overflow".to_string())?;
    }
    Ok(count)
}

fn sort_order(left: &SortableSpoolRecordV1, right: &SortableSpoolRecordV1) -> Ordering {
    left.record
        .kind
        .cmp(&right.record.kind)
        .then_with(|| left.primary_identity.cmp(&right.primary_identity))
        .then_with(|| left.record.split.cmp(&right.record.split))
        .then_with(|| left.record.event_sha256.cmp(&right.record.event_sha256))
        .then_with(|| {
            left.record
                .canonical_event_bytes
                .cmp(&right.record.canonical_event_bytes)
        })
}

fn same_full_event(left: &mut SortableSpoolRecordV1, right: &mut SortableSpoolRecordV1) -> bool {
    left.record.event_sha256 == right.record.event_sha256
        && left.record.canonical_event_bytes == right.record.canonical_event_bytes
}

#[cfg(test)]
mod tests {
    use super::super::events::{
        decode_verified_spool_record, LemmaSplitKeyV1, MorphologyEventV1, TypedEventSpoolConfigV1,
        TypedEventSpoolWriterV1, TypedProductiveEventV1, VerifiedSpoolShardReaderV1,
    };
    use super::super::types::MorphologySlotKeyV1;
    use super::*;

    fn event(index: usize) -> TypedProductiveEventV1 {
        TypedProductiveEventV1::Morphology(MorphologyEventV1 {
            lemma: LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: format!("lemma-{index:03}"),
            },
            normalized_surface: format!("surface-{index:03}"),
            canonical_form_ref: super::super::types::ImportedCanonicalL2FormRefV1(index as u32),
            canonical_feature_mask: 1,
            slot: MorphologySlotKeyV1::new(2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            support: 1,
            provenance: format!("source-{index:03}").into_bytes(),
        })
    }

    fn write_source(
        root: &Path,
        order: impl IntoIterator<Item = usize>,
    ) -> TypedEventSpoolManifestV1 {
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.to_path_buf(),
            shard_count: 2,
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("writer");
        for index in order {
            writer.append(&event(index)).expect("event");
            writer.append(&event(index)).expect("duplicate");
        }
        writer.finish().expect("manifest")
    }

    #[test]
    fn external_sort_is_bounded_deduplicated_and_input_order_deterministic() {
        let base = std::env::temp_dir().join(format!(
            "lay-productive-sort-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let first_source = write_source(&base.join("source-a"), 0..40);
        let second_source = write_source(&base.join("source-b"), (0..40).rev());
        let config = |name: &str| ExternalSpoolSortConfigV1 {
            root: base.join(name),
            maximum_buffer_bytes: 512,
            maximum_open_runs: 3,
            write_buffer_bytes: 1024,
        };
        let first =
            external_sort_verified_spool(&first_source, &config("sorted-a")).expect("first sort");
        let second =
            external_sort_verified_spool_with_workers(&second_source, &config("sorted-b"), 2)
                .expect("second sort");
        assert_eq!(first.input_records, 80);
        assert_eq!(first.unique_records, 40);
        assert!(first.initial_runs > first_source.shards.len() as u64);
        assert!(first.merge_passes >= 2);
        assert_eq!(first.unique_records, second.unique_records);
        for (left, right) in first.shards.iter().zip(&second.shards) {
            assert_eq!(
                fs::read(&left.path).expect("left bytes"),
                fs::read(&right.path).expect("right bytes")
            );
        }
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn external_sort_uses_canonical_lexical_order_not_length_prefixed_wire_order() {
        let base = std::env::temp_dir().join(format!(
            "lay-productive-lexical-sort-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: base.join("source"),
            shard_count: 1,
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("writer");
        for (index, lemma) in ["я", "абажур"].into_iter().enumerate() {
            let mut event = event(index);
            let TypedProductiveEventV1::Morphology(row) = &mut event else {
                unreachable!();
            };
            row.lemma.normalized_lemma = lemma.to_string();
            writer.append(&event).expect("event");
        }
        let source = writer.finish().expect("manifest");
        let sorted = external_sort_verified_spool(
            &source,
            &ExternalSpoolSortConfigV1 {
                root: base.join("sorted"),
                maximum_buffer_bytes: 512,
                maximum_open_runs: 3,
                write_buffer_bytes: 1024,
            },
        )
        .expect("sort");
        let mut reader = VerifiedSpoolShardReaderV1::open(&sorted.shards[0].path).expect("reader");
        let mut lemmas = Vec::new();
        while let Some(record) = reader.next_record().expect("record") {
            lemmas.push(
                decode_verified_spool_record(&record, sorted.split_seed)
                    .expect("decode")
                    .lemma()
                    .normalized_lemma
                    .clone(),
            );
        }
        assert_eq!(lemmas, ["абажур", "я"]);
        fs::remove_dir_all(base).expect("cleanup");
    }
}
