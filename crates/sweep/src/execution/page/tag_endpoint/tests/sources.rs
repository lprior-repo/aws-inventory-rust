use super::*;
mod list_map;
use inventory_core::{
    catalog::runtime::RuntimeProtocol,
    inventory::{ResourceId, Tag, TagObservation},
    record::{Exclusions, Key},
};
use inventory_engine::inventory::{PageCommit, SchemaRegistry};
use serde_json::{Value as Json, json};

struct Source {
    service: &'static str,
    version: &'static str,
    parents: &'static [&'static str],
    operation: &'static str,
    input: &'static str,
    member: Option<&'static str>,
    map: bool,
}

fn decode(plan: &CallablePlan<'_>, body: &Json) -> TestResult<call::DecodedResponse> {
    let protocol = match plan.protocol() {
        RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        RuntimeProtocol::RestJson1 => aws_wire::request::Protocol::RestJson1,
        _ => return Err("fixture requires JSON wire protocol".into()),
    };
    let body = serde_json::to_vec(body)?;
    let record = aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body: &body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    })?;
    Ok(call::DecodedResponse {
        record,
        headers: http::HeaderMap::new(),
    })
}

