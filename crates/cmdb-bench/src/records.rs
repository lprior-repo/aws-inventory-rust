use std::collections::BTreeMap;
use std::error::Error;

use inventory_core::{Edge, Resource, Source};
use serde_json::Value;

use crate::layout::{self, GLOBAL_REGION};
use crate::model::OBSERVED_AT;
use crate::rng::Rng;

pub struct ResourceContext<'a> {
    pub account: usize,
    pub account_id: &'a str,
    pub region: &'a str,
    pub global: bool,
    pub payload_bytes: usize,
    pub vpc: Option<&'a str>,
}

pub fn build_resource(
    context: &ResourceContext<'_>,
    index: usize,
    rng: &mut Rng,
) -> Result<Resource, Box<dyn Error>> {
    let (kind, id) = layout::kind_and_id(context.account, index, context.global)?;
    Ok(Resource {
        id,
        account_id: context.account_id.to_owned(),
        region: resource_region(context.region, kind).to_owned(),
        kind: kind.to_owned(),
        name: Some(format!("{kind}-{index}")),
        properties: properties(context, kind, index, rng)?,
        tags: tags(),
        source: Source::Direct,
        observed_at: OBSERVED_AT,
    })
}

fn resource_region<'a>(region: &'a str, kind: &str) -> &'a str {
    if kind == layout::IAM_ROLE {
        GLOBAL_REGION
    } else {
        region
    }
}

fn tags() -> BTreeMap<String, String> {
    BTreeMap::from([("owner".to_owned(), "synthetic".to_owned())])
}

fn properties(
    context: &ResourceContext<'_>,
    kind: &str,
    index: usize,
    rng: &mut Rng,
) -> Result<Value, Box<dyn Error>> {
    let mut value = payload_value(context.payload_bytes, index, rng)?;
    let Some(map) = value.as_object_mut() else {
        return Ok(value);
    };
    if kind == layout::INSTANCE
        && let Some(vpc_id) = context.vpc
    {
        map.insert("vpc_id".to_owned(), Value::String(vpc_id.to_owned()));
    }
    if context.global && index == 0 {
        map.insert("global_service".to_owned(), Value::Bool(true));
    }
    Ok(value)
}

fn payload_value(
    payload_bytes: usize,
    index: usize,
    rng: &mut Rng,
) -> Result<Value, Box<dyn Error>> {
    let bound = u64::try_from(payload_bytes.saturating_add(1))?;
    let payload_len = usize::try_from(rng.below(bound))?;
    Ok(serde_json::json!({
        "index": index,
        "synthetic": true,
        "payload": "x".repeat(payload_len),
    }))
}

pub fn make_edge(context: &ResourceContext<'_>, from_id: &str, to_id: &str) -> Edge {
    let relation = if context.vpc == Some(to_id) {
        "IN_VPC"
    } else {
        "IN_SECURITY_GROUP"
    };
    Edge {
        from_id: from_id.to_owned(),
        to_id: to_id.to_owned(),
        relation: relation.to_owned(),
        source: Source::Direct,
        observed_at: OBSERVED_AT,
        properties: serde_json::json!({"synthetic": true}),
    }
}
