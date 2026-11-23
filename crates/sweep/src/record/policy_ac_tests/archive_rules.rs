mod collision;
use super::{TestResult, compile, cursor::Cursor, decode};
use crate::{
    execution::page::{child, rows},
    input, record, request,
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{
        AccountId, DiscoveryContext, OperationId, Region, RegionId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::{Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn rule(name: &str) -> Value {
    json!({"ruleName":name,"filter":{"isPublic":{"eq":["true"]}},
        "createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z"})
}

fn text(
    record: &Record,
    fields: &catalog_tool::smithy::Registry,
    field: &str,
) -> TestResult<String> {
    match record::path_value(
        record.root(),
        &[fields.id(field).ok_or("modeled field absent")?],
    )? {
        Some(ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err(format!("required text absent: {field}").into()),
    }
}

