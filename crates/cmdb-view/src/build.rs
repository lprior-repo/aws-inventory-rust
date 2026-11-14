use crate::error::ViewError;
use crate::graph::EdgeView;
use crate::meta::{Coverage, FailureView, ScopeCoverage};
use crate::raw::{RawEdgeRecord, RawFailureRecord, RawRecord, RawResourceRecord, RawScopeRecord};
use crate::resource::ResourceView;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{self, Write};

pub const MAX_SNAPSHOT_BYTES: usize = 1_073_741_824;

const MAX_RESOURCES: usize = 2_000_000;

const MAX_EDGES: usize = 5_000_000;

const MAX_SCOPES: usize = 500_000;

const MAX_FAILURES: usize = 500_000;

pub(crate) struct Collected {
    pub(crate) resources: Vec<ResourceView>,
    pub(crate) edges: Vec<EdgeView>,
    pub(crate) coverage: Coverage,
}

pub(crate) struct Finalized {
    pub(crate) resources: Vec<ResourceView>,
    pub(crate) edges: Vec<EdgeView>,
    pub(crate) coverage: Coverage,
}

pub(crate) fn collect(records: Vec<RawRecord>) -> Result<Collected, ViewError> {
    let mut resources = Vec::new();
    let mut edges = Vec::new();
    let mut coverage = Coverage::default();
    for record in records {
        match record {
            RawRecord::Resource(record) => push_resource(&mut resources, record)?,
            RawRecord::Edge(record) => push_edge(&mut edges, record)?,
            RawRecord::Scope(record) => push_scope(&mut coverage, record),
            RawRecord::Failure(record) => push_failure(&mut coverage, record),
        }
    }
    Ok(Collected {
        resources,
        edges,
        coverage,
    })
}

fn push_resource(
    resources: &mut Vec<ResourceView>,
    record: RawResourceRecord,
) -> Result<(), ViewError> {
    if resources.len() >= MAX_RESOURCES {
        return Err(ViewError::RecordLimit("resources"));
    }
    let raw = record.resource;
    let properties_bytes = json_size(&raw.properties)?;
    resources.push(ResourceView {
        id: raw.id,
        account_id: raw.account_id,
        region: raw.region,
        kind: raw.kind,
        name: raw.name,
        tags: raw.tags,
        properties_bytes,
    });
    Ok(())
}

fn push_edge(edges: &mut Vec<EdgeView>, record: RawEdgeRecord) -> Result<(), ViewError> {
    if edges.len() >= MAX_EDGES {
        return Err(ViewError::RecordLimit("edges"));
    }
    edges.push(EdgeView {
        from_id: record.edge.from_id,
        to_id: record.edge.to_id,
        relation: record.edge.relation,
    });
    Ok(())
}

fn push_scope(coverage: &mut Coverage, record: RawScopeRecord) {
    if coverage.scopes.len() >= MAX_SCOPES {
        coverage.scopes_truncated = true;
        return;
    }
    coverage.scopes.push(ScopeCoverage {
        account_id: record.scope.account_id,
        region: record.scope.region,
        collector: record.scope.collector,
        parent_id: record.scope.parent_id,
        complete: record.complete,
        failed: record.failed,
    });
}

fn push_failure(coverage: &mut Coverage, record: RawFailureRecord) {
    if coverage.failures.len() >= MAX_FAILURES {
        coverage.failures_truncated = true;
        return;
    }
    coverage.failures.push(FailureView {
        account_id: record.scope.account_id,
        region: record.scope.region,
        collector: record.scope.collector,
        category: record.category,
        message: record.message,
    });
}

pub(crate) fn finalize(collected: Collected) -> Finalized {
    let Collected {
        mut resources,
        mut edges,
        coverage,
    } = collected;
    sort_resources(&mut resources);
    sort_edges(&mut edges);
    Finalized {
        resources,
        edges,
        coverage,
    }
}

fn sort_resources(resources: &mut Vec<ResourceView>) {
    resources.sort_by(|left, right| left.id.cmp(&right.id));
    resources.dedup_by(|left, right| left.id == right.id);
    resources.sort_by(|left, right| {
        (&left.account_id, &left.region, &left.kind, &left.id).cmp(&(
            &right.account_id,
            &right.region,
            &right.kind,
            &right.id,
        ))
    });
}

fn sort_edges(edges: &mut Vec<EdgeView>) {
    edges.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    edges.dedup_by(|kept, candidate| {
        kept.from_id == candidate.from_id
            && kept.to_id == candidate.to_id
            && kept.relation == candidate.relation
    });
}

pub(crate) fn index(resources: &[ResourceView]) -> BTreeMap<String, usize> {
    resources
        .iter()
        .enumerate()
        .map(|(position, resource)| (resource.id.clone(), position))
        .collect()
}

pub(crate) fn count_by<F>(
    resources: &[ResourceView],
    key: F,
) -> Result<BTreeMap<String, usize>, ViewError>
where
    F: for<'a> Fn(&'a ResourceView) -> &'a str,
{
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for resource in resources {
        let slot = counts.entry(key(resource).to_owned()).or_insert(0);
        *slot = slot.checked_add(1).ok_or(ViewError::SizeOverflow)?;
    }
    Ok(counts)
}

struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("property size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn json_size(value: &Value) -> Result<usize, ViewError> {
    let mut counter = ByteCounter(0);
    serde_json::to_writer(&mut counter, value).map_err(|_| ViewError::SizeOverflow)?;
    Ok(counter.0)
}
