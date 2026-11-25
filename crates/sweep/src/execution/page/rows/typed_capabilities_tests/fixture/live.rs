use super::*;
use crate::{
    call,
    execution::{
        draining, now_ms,
        page::{PageRequest, assembly, consumer_index::ConsumerIndex, cursor, preparation},
        routing,
    },
    input,
};
use inventory_engine::{
    inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease},
    limiter::{ApiExecutor, ExecutorConfig},
};
#[path = "live/scenario.rs"]
mod scenario;

pub(super) struct Scenario {
    directory: tempfile::TempDir,
    pub catalog: RuntimeCatalog<'static>,
    sweep: SweepId,
    service: String,
    account: AccountId,
    home: RegionId,
}

pub(super) struct NativePage<'a, 'catalog> {
    pub plan: &'a CallablePlan<'catalog>,
    pub lease: &'a WorkLease,
    pub request: &'a Record,
    pub wire: &'a http::Request<bytes::Bytes>,
}

impl NativePage<'_, '_> {
    pub fn text(&self, name: &str) -> TestResult<Option<String>> {
        let id = smithy::Registry::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../catalog/field-ids.txt"
        )))?
        .id(name)
        .ok_or("request field absent from registry")?;
        match record::path_value(self.request.root(), &[id])? {
            None => Ok(None),
            Some(ValueRef::Text(value)) => Ok(Some(value.to_owned())),
            _ => Err("native request selector is not text".into()),
        }
    }
}

impl NativePage<'_, '_> {
    pub fn body_text(&self, name: &str) -> TestResult<Option<String>> {
        let body: serde_json::Value = serde_json::from_slice(self.wire.body())?;
        Ok(body
            .get(name)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned))
    }
}

pub(super) fn keys(kind: &str, ids: &[String]) -> Vec<(String, String, String, String)> {
    let mut keys = ids
        .iter()
        .map(|id| {
            (
                "123456789012".into(),
                "us-east-1".into(),
                kind.into(),
                id.clone(),
            )
        })
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

type EdgeRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

pub(super) fn edges(
    kind: &str,
    target_kind: &str,
    relation: &str,
    pairs: &[(String, String)],
) -> Vec<EdgeRow> {
    let mut edges = pairs
        .iter()
        .map(|(from, to)| {
            (
                "123456789012".into(),
                "us-east-1".into(),
                kind.into(),
                from.clone(),
                "123456789012".into(),
                "us-east-1".into(),
                target_kind.into(),
                to.clone(),
                relation.into(),
            )
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}
