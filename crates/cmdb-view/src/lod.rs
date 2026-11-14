use crate::Snapshot;
use crate::error::ViewError;
use crate::resource::ResourceView;
use std::collections::BTreeMap;

pub const MAX_LOD_BUCKETS: usize = 1_024;

pub const MAX_TOP_DEGREE: usize = 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LodLimits {
    max_kinds: usize,
    max_accounts: usize,
    max_top_degree: usize,
}

impl LodLimits {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountBucket {
    pub key: String,

    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DegreeEntry<'a> {
    pub id: &'a str,

    pub kind: &'a str,

    pub degree: usize,
}

#[derive(Clone, Debug)]
pub struct LodSummary<'a> {
    pub total_resources: usize,

    pub kinds: Vec<CountBucket>,

    pub accounts: Vec<CountBucket>,

    pub top_degree: Vec<DegreeEntry<'a>>,

    pub truncated_kinds: bool,

    pub truncated_accounts: bool,

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
