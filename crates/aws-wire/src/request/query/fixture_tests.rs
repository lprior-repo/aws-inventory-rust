mod cases;
mod compression;
mod limits;
mod model;

use self::cases::check_fixture;
use super::encode;
use crate::request::{CompressionPolicy, Protocol, RequestContext, RequestLimits};
use catalog_tool::smithy::{Registry, build_catalog};
use flate2::read::GzDecoder;
use inventory_core::{
    catalog::{Catalog, Operation, Service, Target},
    record::{Float, Key, Record, Time, Value},
};
use serde_json::{Map as JsonMap, Value as Json};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    io::Read,
};

const AWS_QUERY: &str = include_str!("../../../tests/fixtures/protocol-tests/awsQuery.json");
const EC2_QUERY: &str = include_str!("../../../tests/fixtures/protocol-tests/ec2Query.json");
const AUTO_TOKEN: &str = "00000000-0000-4000-8000-000000000000";
type FixtureResult<T> = Result<T, Box<dyn Error>>;

fn fixture_error(reason: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        reason.into(),
    ))
}
fn fixture_endpoint(case: &Json) -> String {
    let host = case.get("host").and_then(Json::as_str);
    let host = match host {
        Some(host) => host,
        None => "example.com",
    };
    if let Some((scheme, authority_and_base_path)) = host.split_once("://") {
        format!("{scheme}://{authority_and_base_path}")
    } else {
        format!("https://{host}")
    }
}

#[test]
fn published_aws_query_request_cases_are_encoded_or_accounted_for_as_server_cases()
-> FixtureResult<()> {
    check_fixture(AWS_QUERY, "awsQuery", Protocol::AwsQuery)
}

#[test]
fn published_ec2_query_request_cases_are_encoded_or_accounted_for_as_server_cases()
-> FixtureResult<()> {
    check_fixture(EC2_QUERY, "ec2Query", Protocol::Ec2Query)
}
