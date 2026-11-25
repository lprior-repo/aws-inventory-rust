use crate::{
    execution::{
        page::{
            child::{
                Context,
                policy_dfgk_tests::{project_children, projected_references},
            },
            consumer_index::ConsumerIndex,
            cursor,
        },
        routing,
    },
    input, record, request,
};
use aws_wire::{
    request::Protocol,
    response::{DecodeLimits, ResponseContext},
};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanScope, RuntimeCatalog, RuntimeProtocol},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::Record,
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use serde_json::Value as Json;
use std::{collections::BTreeSet, error::Error};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(super) type Edge = (String, String, String);
mod scenario;

pub(super) fn compiled(
    service: &str,
    version: &str,
    operations: &[&str],
    first_kind: &str,
) -> TestResult<Vec<u8>> {
    record::policy_lp_tests::fixture::compiled(service, version, operations, first_kind)
}

pub(super) fn plan<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
    service: &str,
    operation: &str,
) -> TestResult<&'a CallablePlan<'catalog>> {
    catalog
        .lookup(service, operation)
        .ok_or_else(|| format!("compiled {service}.{operation} absent").into())
}

pub(super) struct Scenario<'a, 'catalog> {
    pub directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub account: AccountId,
    pub home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

pub(super) fn page(mut response: Json, token: &str, second: bool) -> TestResult<Json> {
    if !second {
        response
            .as_object_mut()
            .ok_or("native response is not object")?
            .insert(token.to_owned(), Json::String("next-page".to_owned()));
    }
    Ok(response)
}

pub(super) fn native_input(wire: &http::Request<bytes::Bytes>) -> TestResult<String> {
    Ok(format!(
        "{} {}",
        wire.uri(),
        std::str::from_utf8(wire.body())?
    ))
}

pub(super) fn assert_cursor(wire: &http::Request<bytes::Bytes>, second: bool) -> TestResult {
    assert_eq!(native_input(wire)?.contains("next-page"), second);
    Ok(())
}

pub(super) fn text(wire: &Json, field: &str) -> TestResult<String> {
    wire.get(field)
        .and_then(Json::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("native request field {field} absent").into())
}

pub(super) fn bound(plan: &CallablePlan<'_>, request: &Record) -> TestResult<String> {
    let binding = plan
        .bindings()
        .iter()
        .find(|binding| {
            matches!(
                binding.source,
                inventory_core::catalog::runtime::BindingSource::Operation { .. }
            )
        })
        .ok_or("parent binding absent")?;
    match record::path_value(request.root(), &binding.path)? {
        Some(inventory_core::record::ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err("bound parent request is not text".into()),
    }
}

pub(super) fn assert_parent(wire: &http::Request<bytes::Bytes>, parent: &str) -> TestResult {
    let text = native_input(wire)?
        .replace("%3A", ":")
        .replace("%3a", ":")
        .replace("%2F", "/")
        .replace("%2f", "/");
    assert!(
        text.contains(parent),
        "discovered parent absent from native request: {parent}"
    );
    Ok(())
}
