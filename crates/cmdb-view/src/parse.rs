use crate::build::{self, Finalized};
use crate::error::ViewError;
use crate::graph::{self, EdgeView};
use crate::meta::{Coverage, SNAPSHOT_ARTIFACT, SUPPORTED_VERSION, SnapshotMeta};
use crate::raw::{RawDocument, RawHeader, RawRecord};
use crate::resource::ResourceView;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Snapshot {
    pub(crate) meta: SnapshotMeta,
    pub(crate) resources: Vec<ResourceView>,
    pub(crate) resource_index: BTreeMap<String, usize>,
    pub(crate) edges: Vec<EdgeView>,
    pub(crate) adjacency: BTreeMap<String, Vec<usize>>,
    pub(crate) kind_counts: BTreeMap<String, usize>,
    pub(crate) account_counts: BTreeMap<String, usize>,
    pub(crate) coverage: Coverage,
}

impl Snapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self, ViewError> {
        if bytes.len() > build::MAX_SNAPSHOT_BYTES {
            return Err(ViewError::InputLimit("snapshot bytes"));
        }
        let RawDocument { header, records } = serde_json::from_slice(bytes)?;
        validate_header(&header)?;
        Self::from_parts(&header, records)
    }

    fn from_parts(header: &RawHeader, records: Vec<RawRecord>) -> Result<Self, ViewError> {
        let meta = SnapshotMeta::from_header(header);
        let Finalized {
            resources,
            edges,
            coverage,
        } = build::finalize(build::collect(records)?);
        let resource_index = build::index(&resources);
        let kind_counts = build::count_by(&resources, |resource| resource.kind.as_str())?;
        let account_counts = build::count_by(&resources, |resource| resource.account_id.as_str())?;
        let adjacency = graph::build_adjacency(&edges);
        Ok(Self {
            meta,
            resources,
            resource_index,
            edges,
            adjacency,
            kind_counts,
            account_counts,
            coverage,
        })
    }

    pub fn coverage(&self) -> &Coverage {
        &self.coverage
    }
}

fn validate_header(header: &RawHeader) -> Result<(), ViewError> {
    if header.artifact != SNAPSHOT_ARTIFACT {
        return Err(ViewError::UnsupportedArtifact(header.artifact.clone()));
    }
    if header.version != SUPPORTED_VERSION {
        return Err(ViewError::UnsupportedVersion(header.version));
    }
    Ok(())
}
