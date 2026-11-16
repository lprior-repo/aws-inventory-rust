mod fixture;
mod monitor;
mod definitions;
mod twinmaker;
mod arn_scope;
mod access_assertions;
use crate::sweep_runtime::execution::page::cursor;

use fixture::{TestResult, Scenario, catalog, plan};
use serde_json::{Value as Json, json};

fn arn(service: &str, resource: &str) -> String {
    format!("arn:aws:{service}:us-east-1:123456789012:{resource}")
}

fn query<'a>(wire: &'a http::Request<bytes::Bytes>, name: &str) -> TestResult<&'a str> {
    wire.uri().query().and_then(|query| query.split('&').find_map(|entry| {
        let (key, value) = entry.split_once('=')?;
        (key == name).then_some(value)
    })).ok_or_else(|| format!("wire request omitted query parameter {name}").into())
}

