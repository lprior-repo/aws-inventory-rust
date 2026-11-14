//! Level-of-detail summaries: counts per kind/account and top-degree nodes.

use crate::Snapshot;
use crate::error::ViewError;
use crate::resource::ResourceView;
use std::collections::BTreeMap;

/// Largest number of buckets one LOD axis may return.
pub const MAX_LOD_BUCKETS: usize = 1_024;
/// Largest top-degree list one LOD summary may return.
pub const MAX_TOP_DEGREE: usize = 1_000;

/// Caps applied when materializing a summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LodLimits {
    max_kinds: usize,
    max_accounts: usize,
    max_top_degree: usize,
}

impl LodLimits {
    /// Validate each cap against its global maximum.
    ///
    /// # Errors
    /// Rejects zero caps, kind/account caps above 1,024 or degree caps above 1,000.
    pub fn new(
        max_kinds: usize,
        max_accounts: usize,
        max_top_degree: usize,
    ) -> Result<Self, ViewError> {
        if !(1..=MAX_LOD_BUCKETS).contains(&max_kinds)
            || !(1..=MAX_LOD_BUCKETS).contains(&max_accounts)
            || !(1..=MAX_TOP_DEGREE).contains(&max_top_degree)
        {
            return Err(ViewError::InvalidLodLimit);
        }
        Ok(Self {
            max_kinds,
            max_accounts,
            max_top_degree,
        })
    }
}

impl Default for LodLimits {
    fn default() -> Self {
        Self {
            max_kinds: 256,
            max_accounts: 256,
            max_top_degree: 25,
        }
    }
}

/// One `(key, count)` bucket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountBucket {
    /// Resource kind or account spelling for the summary axis.
    pub key: String,
    /// Number of retained resource views in this bucket.
    pub count: usize,
}

/// One node plus its incident-edge count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DegreeEntry<'a> {
    /// Identity of a retained resource with at least one incident edge.
    pub id: &'a str,
    /// Kind copied from the retained resource view.
    pub kind: &'a str,
    /// Incident edge records; a self-loop counts once.
    pub degree: usize,
}

/// Compact summary used for high-level rendering of large views.
#[derive(Clone, Debug)]
pub struct LodSummary<'a> {
    /// Retained resource count, independent of the requested summary caps.
    pub total_resources: usize,
    /// Sorted by count descending, then key ascending.
    pub kinds: Vec<CountBucket>,
    /// Sorted by count descending, then key ascending.
    pub accounts: Vec<CountBucket>,
    /// Nodes with at least one incident edge, degree descending then id ascending.
    pub top_degree: Vec<DegreeEntry<'a>>,
    /// More kind buckets exist than the requested cap.
    pub truncated_kinds: bool,
    /// More account buckets exist than the requested cap.
    pub truncated_accounts: bool,
    /// More present resources have incident edges than the requested cap.
    pub truncated_top_degree: bool,
}

pub(crate) fn summarize<'a>(
    resources: &'a [ResourceView],
    kind_counts: &BTreeMap<String, usize>,
    account_counts: &BTreeMap<String, usize>,
    adjacency: &'a BTreeMap<String, Vec<usize>>,
    index: &'a BTreeMap<String, usize>,
    limits: LodLimits,
) -> LodSummary<'a> {
    let (kinds, truncated_kinds) = buckets(kind_counts, limits.max_kinds);
    let (accounts, truncated_accounts) = buckets(account_counts, limits.max_accounts);
    let (top_degree, truncated_top_degree) =
        top_degree(adjacency, resources, index, limits.max_top_degree);
    LodSummary {
        total_resources: resources.len(),
        kinds,
        accounts,
        top_degree,
        truncated_kinds,
        truncated_accounts,
        truncated_top_degree,
    }
}

fn top_degree<'a>(
    adjacency: &'a BTreeMap<String, Vec<usize>>,
    resources: &'a [ResourceView],
    index: &'a BTreeMap<String, usize>,
    cap: usize,
) -> (Vec<DegreeEntry<'a>>, bool) {
    let mut entries: Vec<DegreeEntry<'a>> = adjacency
        .iter()
        .filter(|(_, incidents)| !incidents.is_empty())
        .filter_map(|(id, incidents)| degree_entry(resources, index, id, incidents.len()))
        .collect();
    entries.sort_by(rank);
    let truncated = entries.len() > cap;
    entries.truncate(cap);
    (entries, truncated)
}

fn degree_entry<'a>(
    resources: &'a [ResourceView],
    index: &BTreeMap<String, usize>,
    id: &'a str,
    degree: usize,
) -> Option<DegreeEntry<'a>> {
    let resource = lookup(resources, index, id)?;
    Some(DegreeEntry {
        id,
        kind: resource.kind.as_str(),
        degree,
    })
}

fn rank(left: &DegreeEntry<'_>, right: &DegreeEntry<'_>) -> std::cmp::Ordering {
    right
        .degree
        .cmp(&left.degree)
        .then_with(|| left.id.cmp(right.id))
}

fn buckets(counts: &BTreeMap<String, usize>, cap: usize) -> (Vec<CountBucket>, bool) {
    let mut buckets: Vec<CountBucket> = counts
        .iter()
        .map(|(key, &count)| CountBucket {
            key: key.clone(),
            count,
        })
        .collect();
    buckets.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.key.cmp(&right.key))
    });
    let truncated = buckets.len() > cap;
    buckets.truncate(cap);
    (buckets, truncated)
}

fn lookup<'a>(
    resources: &'a [ResourceView],
    index: &BTreeMap<String, usize>,
    id: &str,
) -> Option<&'a ResourceView> {
    let position = *index.get(id)?;
    resources.get(position)
}

impl Snapshot {
    /// Level-of-detail summary bounded by `limits`.
    pub fn lod(&self, limits: LodLimits) -> LodSummary<'_> {
        summarize(
            &self.resources,
            &self.kind_counts,
            &self.account_counts,
            &self.adjacency,
            &self.resource_index,
            limits,
        )
    }
}
