use super::runtime::{GrokkingCandidate, LexicalGrokkingMemory, ReadoutMode};

pub(super) struct L1QueryField<'a> {
    surface: &'a str,
}

impl<'a> L1QueryField<'a> {
    pub(super) fn new(surface: &'a str) -> Self {
        Self { surface }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ReadoutRequest {
    limit: usize,
    mode: ReadoutMode,
}

impl ReadoutRequest {
    pub(super) fn new(limit: usize, mode: ReadoutMode) -> Self {
        Self { limit, mode }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SearchCompleteness {
    NotRequested,
    ExactSingleton,
    LegacyBoundedFrontier,
    NoLegacyEvidence,
}

pub(super) struct PeakSearchResult {
    pub(super) candidates: Vec<GrokkingCandidate>,
    pub(super) completeness: SearchCompleteness,
}

impl PeakSearchResult {
    fn new(candidates: Vec<GrokkingCandidate>, completeness: SearchCompleteness) -> Self {
        Self {
            candidates,
            completeness,
        }
    }
}

pub(super) trait L1PeakSearch {
    fn search(
        &self,
        field: &LexicalGrokkingMemory,
        query: L1QueryField<'_>,
        request: ReadoutRequest,
    ) -> PeakSearchResult;
}

pub(super) struct LegacyBirthSearch;

impl L1PeakSearch for LegacyBirthSearch {
    fn search(
        &self,
        field: &LexicalGrokkingMemory,
        query: L1QueryField<'_>,
        request: ReadoutRequest,
    ) -> PeakSearchResult {
        if request.limit == 0 {
            return PeakSearchResult::new(Vec::new(), SearchCompleteness::NotRequested);
        }
        if request.limit == 1 && request.mode == ReadoutMode::Full {
            if let Some(candidate) = field.exact_singleton_readout(query.surface) {
                return PeakSearchResult::new(vec![candidate], SearchCompleteness::ExactSingleton);
            }
        }
        let Some(prepared) = field.prepare_readout(query.surface, request.limit) else {
            return PeakSearchResult::new(Vec::new(), SearchCompleteness::NoLegacyEvidence);
        };
        PeakSearchResult::new(
            field.finish_readout(query.surface, request.limit, request.mode, &prepared),
            SearchCompleteness::LegacyBoundedFrontier,
        )
    }
}
