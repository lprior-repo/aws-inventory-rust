mod catalog;
mod lattice;
mod mail;
mod mail_collision;
mod marketplace;
mod metadata;
mod queries;
mod scenario;
mod waf;
mod wafv2;
mod wire;
mod workspaces;
mod zero_collection;
mod zero_influx;
mod zero_influx_tags;

use super::super::{PageRequest, assembly, consumer_index::ConsumerIndex, cursor, preparation};
use super::*;
use crate::{execution::routing, record};
use inventory_core::catalog::runtime::{PlanScope, RuntimeCatalog};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use serde_json::{Value as Json, json};
use std::{collections::BTreeMap, error::Error, sync::LazyLock};
use wire::{decode_rpc_body, json_to_cbor};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
static FIELDS: LazyLock<Result<catalog_tool::smithy::Registry, String>> = LazyLock::new(|| {
    catalog_tool::smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))
    .map_err(|error| error.to_string())
});

struct Scenario<'a, 'catalog> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{
        request::Protocol,
        response::{DecodeLimits, ResponseContext},
    };
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        inventory_core::catalog::runtime::RuntimeProtocol::RpcV2Cbor => Protocol::RpcV2Cbor,
        inventory_core::catalog::runtime::RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        inventory_core::catalog::runtime::RuntimeProtocol::RestXml => Protocol::RestXml,
        _ => return Err("unexpected native protocol in T-Z fixture".into()),
    };
    Ok(aws_wire::response::decode(ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits: DecodeLimits::default(),
    })?)
}

fn text<'a>(record: &'a Record, name: &str) -> TestResult<&'a str> {
    let fields = FIELDS.as_ref().map_err(|error| error.clone())?;
    let path = [fields.id(name).ok_or("named input field absent")?];
    match record::path_value(record.root(), &path)? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err(format!("encoded input omitted {name}").into()),
    }
}

fn page(items: &str, row: Json, next: bool, token_name: &str) -> Json {
    let mut body = serde_json::Map::new();
    body.insert(items.to_owned(), Json::Array(vec![row]));
    if next {
        body.insert(token_name.to_owned(), json!("page-two"));
    }
    Json::Object(body)
}

fn correlated(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| format!("{}:{part}", part.len()))
        .collect::<Vec<_>>()
        .join("|")
}

fn assert_token(wire: &http::Request<bytes::Bytes>, name: &str, resumed: bool) -> TestResult {
    let expected = resumed.then_some("page-two");
    if wire.method() == http::Method::GET {
        let actual = wire
            .uri()
            .query()
            .unwrap_or_default()
            .split('&')
            .find_map(|pair| pair.strip_prefix(&format!("{name}=")));
        assert_eq!(actual, expected);
    } else if wire.headers().get("smithy-protocol")
        == Some(&http::HeaderValue::from_static("rpc-v2-cbor"))
    {
        let body = decode_rpc_body(wire.body())?;
        let actual = body.get(name).and_then(Json::as_str);
        assert_eq!(actual, expected);
    } else {
        let body: Json = if wire.body().is_empty() {
            Json::Null
        } else {
            serde_json::from_slice(wire.body())?
        };
        assert_eq!(body.get(name).and_then(Json::as_str), expected);
    }
    Ok(())
}
