use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use std::sync::Arc;

use rayon::prelude::*;

use super::super::format;
use super::super::model::WaveCoupling;
use super::super::v8::{self, V8Artifact};
use super::config::reverse_cache_bytes;
use super::LexicalGrokkingMemory;

pub(super) enum RelationStore {
    Eager,
    LazyV8(V8Artifact),
}

pub(super) enum CouplingView<'a> {
    Borrowed(&'a [WaveCoupling]),
    Shared(Arc<[WaveCoupling]>),
}

impl Deref for CouplingView<'_> {
    type Target = [WaveCoupling];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(relations) => relations,
            Self::Shared(relations) => relations,
        }
    }
}

#[derive(Default)]
pub(super) struct ReverseCache {
    bytes: usize,
    order: VecDeque<u32>,
    entries: HashMap<u32, Arc<[WaveCoupling]>>,
}

impl LexicalGrokkingMemory {
    pub(super) fn forward_coupling_views(&self, atom_ids: &[u32]) -> Vec<CouplingView<'_>> {
        match &self.relations {
            RelationStore::Eager => atom_ids
                .iter()
                .map(|atom_id| self.forward_couplings(*atom_id))
                .collect(),
            RelationStore::LazyV8(artifact) => artifact
                .postings(atom_ids)
                .unwrap_or_else(|_| {
                    atom_ids
                        .iter()
                        .map(|_| Arc::from(Vec::<WaveCoupling>::new()))
                        .collect()
                })
                .into_iter()
                .map(CouplingView::Shared)
                .collect(),
        }
    }

    pub(super) fn frontier_reverse_batch(
        &self,
        frontier: &[(u32, super::ForwardActivation)],
    ) -> (Vec<Arc<[WaveCoupling]>>, bool) {
        let cache_budget = reverse_cache_bytes();
        let mut resolved = vec![None; frontier.len()];
        if cache_budget != 0 {
            if let Ok(cache) = self.reverse_cache.lock() {
                for (index, (terminal_id, _)) in frontier.iter().enumerate() {
                    resolved[index] = cache.entries.get(terminal_id).cloned();
                }
            }
        }
        let missing = frontier
            .iter()
            .enumerate()
            .filter_map(|(index, (terminal_id, _))| {
                resolved[index].is_none().then_some((index, *terminal_id))
            })
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return (resolved.into_iter().flatten().collect(), cache_budget != 0);
        }
        let decoded = v8::runtime_pool_install(|| {
            missing
                .par_iter()
                .map(|(index, terminal_id)| {
                    let relations: Arc<[WaveCoupling]> =
                        format::reconstruct_compact_center_reverse(&self.package, *terminal_id)
                            .unwrap_or_default()
                            .into();
                    (*index, *terminal_id, relations)
                })
                .collect::<Vec<_>>()
        });
        if cache_budget == 0 {
            for (index, _, relations) in decoded {
                resolved[index] = Some(relations);
            }
        } else if let Ok(mut cache) = self.reverse_cache.lock() {
            for (index, terminal_id, relations) in decoded {
                if let Some(existing) = cache.entries.get(&terminal_id) {
                    resolved[index] = Some(Arc::clone(existing));
                    continue;
                }
                let relation_bytes = relations
                    .len()
                    .saturating_mul(std::mem::size_of::<WaveCoupling>());
                while cache.bytes.saturating_add(relation_bytes) > cache_budget {
                    let Some(evicted_id) = cache.order.pop_front() else {
                        break;
                    };
                    let Some(evicted) = cache.entries.remove(&evicted_id) else {
                        continue;
                    };
                    cache.bytes = cache.bytes.saturating_sub(
                        evicted
                            .len()
                            .saturating_mul(std::mem::size_of::<WaveCoupling>()),
                    );
                }
                if cache.bytes.saturating_add(relation_bytes) <= cache_budget {
                    cache.bytes = cache.bytes.saturating_add(relation_bytes);
                    cache.order.push_back(terminal_id);
                    cache.entries.insert(terminal_id, Arc::clone(&relations));
                }
                resolved[index] = Some(relations);
            }
        } else {
            for (index, _, relations) in decoded {
                resolved[index] = Some(relations);
            }
        }
        (
            resolved
                .into_iter()
                .map(|relations| relations.unwrap_or_else(|| Arc::from([])))
                .collect(),
            false,
        )
    }

    pub(super) fn forward_couplings(&self, atom_id: u32) -> CouplingView<'_> {
        match &self.relations {
            RelationStore::Eager => {
                let Some(record) = self.package.atoms.get(atom_id as usize) else {
                    return CouplingView::Borrowed(&[]);
                };
                let start = record.coupling_start as usize;
                let end = start.saturating_add(record.coupling_count as usize);
                CouplingView::Borrowed(
                    self.package
                        .forward_couplings
                        .get(start..end)
                        .unwrap_or_default(),
                )
            }
            RelationStore::LazyV8(artifact) => CouplingView::Shared(
                artifact
                    .posting(atom_id)
                    .unwrap_or_else(|_| Arc::from(Vec::<WaveCoupling>::new())),
            ),
        }
    }

    pub(in crate::nanda_wave::lexical_grokking) fn forward_degree(&self, atom_id: u32) -> usize {
        match &self.relations {
            RelationStore::Eager => self
                .package
                .atoms
                .get(atom_id as usize)
                .map(|record| record.coupling_count as usize)
                .unwrap_or_default(),
            RelationStore::LazyV8(artifact) => artifact.posting_degree(atom_id),
        }
    }

    pub(in crate::nanda_wave::lexical_grokking) fn forward_relation_count(&self) -> usize {
        match &self.relations {
            RelationStore::Eager => self.package.forward_couplings.len(),
            RelationStore::LazyV8(artifact) => artifact.forward_relation_count(),
        }
    }

    pub(in crate::nanda_wave::lexical_grokking) fn reverse_relation_count(&self) -> usize {
        match &self.relations {
            RelationStore::Eager => self.package.reverse_couplings.len(),
            RelationStore::LazyV8(artifact) => artifact.reverse_relation_count(),
        }
    }

    pub(super) fn reverse_couplings(&self, terminal_id: u32) -> CouplingView<'_> {
        if matches!(self.relations, RelationStore::Eager) {
            let Some(center) = self.package.centers.get(terminal_id as usize) else {
                return CouplingView::Borrowed(&[]);
            };
            let start = center.coupling_start as usize;
            let end = start.saturating_add(center.coupling_count as usize);
            return CouplingView::Borrowed(
                self.package
                    .reverse_couplings
                    .get(start..end)
                    .unwrap_or_default(),
            );
        }
        CouplingView::Shared(self.reverse_couplings_shared(terminal_id))
    }

    fn reverse_couplings_shared(&self, terminal_id: u32) -> Arc<[WaveCoupling]> {
        let cache_budget = reverse_cache_bytes();
        if cache_budget != 0 {
            if let Ok(cache) = self.reverse_cache.lock() {
                if let Some(relations) = cache.entries.get(&terminal_id) {
                    return Arc::clone(relations);
                }
            }
        }
        let relations: Arc<[WaveCoupling]> =
            format::reconstruct_compact_center_reverse(&self.package, terminal_id)
                .unwrap_or_default()
                .into();
        let relation_bytes = relations
            .len()
            .saturating_mul(std::mem::size_of::<WaveCoupling>());
        if relation_bytes <= cache_budget {
            if let Ok(mut cache) = self.reverse_cache.lock() {
                if let Some(existing) = cache.entries.get(&terminal_id) {
                    return Arc::clone(existing);
                }
                while cache.bytes.saturating_add(relation_bytes) > cache_budget {
                    let Some(evicted_id) = cache.order.pop_front() else {
                        break;
                    };
                    let Some(evicted) = cache.entries.remove(&evicted_id) else {
                        continue;
                    };
                    cache.bytes = cache.bytes.saturating_sub(
                        evicted
                            .len()
                            .saturating_mul(std::mem::size_of::<WaveCoupling>()),
                    );
                }
                cache.bytes = cache.bytes.saturating_add(relation_bytes);
                cache.order.push_back(terminal_id);
                cache.entries.insert(terminal_id, Arc::clone(&relations));
            }
        }
        relations
    }
}
