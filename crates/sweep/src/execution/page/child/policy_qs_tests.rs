mod children;
mod firstwave;
mod fixture;
mod multi_emission;
mod quicksight;
mod regional;
mod scenario;
mod typed;

use super::super::{PageRequest, assembly, consumer_index::ConsumerIndex, cursor};
use super::*;
use crate::execution::routing;
use inventory_core::catalog::runtime::{BindingSource, PlanScope, RuntimeCatalog};
use inventory_engine::{
    inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease},
    limiter::{ApiExecutor, ExecutorConfig},
};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Wire = http::Request<bytes::Bytes>;

struct Scenario<'a, 'c> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'c>,
    executor: ApiExecutor,
}

fn text<'a>(value: &'a Json, field: &str) -> TestResult<&'a str> {
    value
        .get(field)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("native request omitted {field}").into())
}
fn body(wire: &Wire) -> TestResult<Json> {
    if wire
        .headers()
        .get("smithy-protocol")
        .is_some_and(|value| value == "rpc-v2-cbor")
    {
        return crate::test_cbor::cbor_to_json(wire.body());
    }
    Ok(serde_json::from_slice(wire.body())?)
}
fn bytes(value: Json) -> TestResult<Vec<u8>> {
    Ok(serde_json::to_vec(&value)?)
}
fn paginated(mut value: Json, token: &str, second: bool) -> TestResult<Vec<u8>> {
    if !second {
        value
            .as_object_mut()
            .ok_or("native response not an object")?
            .insert(token.to_owned(), json!("page-two"));
    }
    bytes(value)
}
fn count(rows: &[(String, String, String)], kind: &str) -> usize {
    rows.iter().filter(|(saved, _, _)| saved == kind).count()
}
