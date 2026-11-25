mod cleanrooms;
mod cognito;
mod connect;
mod consumption;
mod domains;
use consumption::run;

use super::{TestResult, compile, cursor::Cursor, decode, resource_key};
use crate::{
    execution::page::{child, consumer_index::ConsumerIndex, rows},
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
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};

const OWNERS: [&str; 2] = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
];

struct Page {
    body: Json,
    id: String,
    wire: NativeRequest,
}

struct Owner {
    value: String,
    pages: [Page; 2],
}

struct Case {
    operation: &'static str,
    kind: &'static str,
    parent_field: &'static str,
    requires_request_identity: bool,
    owners: [Owner; 2],
}

struct NativeRequest {
    method: &'static str,
    path: String,
    query: BTreeSet<String>,
    body: Json,
}

impl NativeRequest {
    fn get(path: String, query: &[&str]) -> Self {
        Self {
            method: "GET",
            path,
            query: query.iter().map(|value| (*value).to_owned()).collect(),
            body: Json::Null,
        }
    }

    fn post(path: String, body: Json) -> Self {
        Self {
            method: "POST",
            path,
            query: BTreeSet::new(),
            body,
        }
    }

    fn assert_matches(&self, wire: &http::Request<bytes::Bytes>) -> TestResult {
        assert_eq!(wire.method().as_str(), self.method);
        assert_eq!(wire.uri().path(), self.path);
        let query = wire.uri().query().map_or_else(BTreeSet::new, |query| {
            query.split('&').map(str::to_owned).collect()
        });
        assert_eq!(query, self.query);
        let body = if wire.body().is_empty() {
            Json::Null
        } else {
            serde_json::from_slice(wire.body())?
        };
        assert_eq!(body, self.body);
        Ok(())
    }
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

fn response(items: &str, row: Json, token: &str, later: bool) -> Json {
    if later {
        json!({items:[row]})
    } else {
        json!({items:[row],token:"later"})
    }
}
