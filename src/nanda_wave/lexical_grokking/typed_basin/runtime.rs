use std::sync::atomic::{AtomicU64, Ordering};

use super::super::forward_decoder_index::ForwardDecoderIndex;
use super::super::model::LexicalGrokkingPackage;
use super::super::restoration::RestorationReadout;
use super::super::runtime::{
    truncate_with_reconstruction_tail, GrokkingCandidate, LexicalGrokkingMemory,
};
use super::super::typed_edit_traversal::phase7d_terminal_evidence;
use super::implicit_forward::{reconstruct_candidate, ImplicitCandidate};
use super::settlement::settle_exact_case;
use super::support::ExactSupportField;

pub(in crate::nanda_wave::lexical_grokking) struct TypedBasinRuntime {
    decoder_index: ForwardDecoderIndex,
    support: ExactSupportField,
    query_failures: AtomicU64,
}

pub(in crate::nanda_wave::lexical_grokking) struct TypedBasinReadout {
    pub(in crate::nanda_wave::lexical_grokking) candidates: Vec<GrokkingCandidate>,
    pub(in crate::nanda_wave::lexical_grokking) readout: RestorationReadout,
}

impl TypedBasinRuntime {
    pub(in crate::nanda_wave::lexical_grokking) fn new(
        package: &LexicalGrokkingPackage,
        support: ExactSupportField,
    ) -> Result<Self, String> {
        validate_runtime_dependencies(package)?;
        if support.values().len() != package.atoms.len() {
            return Err("V9 exact-support and atom counts differ".to_string());
        }
        let decoder_index = ForwardDecoderIndex::build(package)?;
        Ok(Self {
            decoder_index,
            support,
            query_failures: AtomicU64::new(0),
        })
    }

    pub(in crate::nanda_wave::lexical_grokking) fn readout(
        &self,
        memory: &LexicalGrokkingMemory,
        surface: &str,
        limit: usize,
    ) -> Result<TypedBasinReadout, String> {
        if limit == 0 {
            return Ok(TypedBasinReadout {
                candidates: Vec::new(),
                readout: super::super::restoration::classify(
                    &[],
                    memory.package.restoration_calibration,
                ),
            });
        }
        let typed =
            phase7d_terminal_evidence(&self.decoder_index, &memory.package.decoder_nodes, surface)?;
        let observed = super::observed_lexical_atoms(memory, surface);
        let mut implicit = typed
            .terminal_ids
            .into_iter()
            .map(|terminal_id| {
                reconstruct_candidate(&memory.package, &self.support, &observed, terminal_id)
            })
            .collect::<Result<Vec<ImplicitCandidate>, String>>()?;
        implicit.sort_unstable_by_key(|candidate| candidate.terminal_id);
        if implicit
            .windows(2)
            .any(|pair| pair[0].terminal_id == pair[1].terminal_id)
        {
            return Err("V9 typed basin contains duplicate terminals".to_string());
        }
        let exact = settle_exact_case(memory, &self.support, surface, &implicit)?;
        let mut candidates = exact.candidates;
        truncate_with_reconstruction_tail(&mut candidates, limit);
        Ok(TypedBasinReadout {
            candidates,
            readout: exact.readout,
        })
    }

    pub(in crate::nanda_wave::lexical_grokking) fn record_failure(&self, error: &str) {
        let count = self.query_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_power_of_two() {
            eprintln!("l11_v9_query_failed count={count} error={error}");
        }
    }

    pub(in crate::nanda_wave::lexical_grokking) fn query_failures(&self) -> u64 {
        self.query_failures.load(Ordering::Relaxed)
    }

    pub(in crate::nanda_wave::lexical_grokking) fn support(&self) -> &ExactSupportField {
        &self.support
    }

    pub(in crate::nanda_wave::lexical_grokking) fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "format": "v9",
            "owner": "phase8i_exact_typed_basin",
            "decoder_index_bytes": self.decoder_index.resident_bytes(),
            "exact_support_bytes": self.support.resident_bytes(),
            "exact_support_overflow_atoms": self.support.metrics.exact_overflow_atoms,
            "maximum_exact_support": self.support.metrics.maximum_exact_support,
            "query_failures": self.query_failures(),
        })
    }
}

fn validate_runtime_dependencies(package: &LexicalGrokkingPackage) -> Result<(), String> {
    if !package.forward_couplings.is_empty()
        || !package.reverse_couplings.is_empty()
        || !package.anti_centers.is_empty()
        || !package.pair_profiles.is_empty()
        || !package.pair_centers.is_empty()
        || !package.center_phase_profiles.is_empty()
        || !package.positive_subcenters.is_empty()
        || !package.anti_subcenters.is_empty()
        || !package.hard_negative_subcenters.is_empty()
        || !package.ambiguity_subcenters.is_empty()
        || !package.keyboard_geometry_units.is_empty()
    {
        return Err("V9 exact runtime contains an unresolved learned or relation bank".to_string());
    }
    Ok(())
}
