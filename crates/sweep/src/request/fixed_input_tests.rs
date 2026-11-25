mod engines;
mod literals;
mod regional;
use super::encode;
use crate::{execution::routing, input::root_bindings};
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeCatalogError},
    inventory::RegionId,
    record::{FieldId, Key, Record, Value},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::Path,
    time::{Duration, SystemTime},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n.fixed#Service":{"type":"service","version":"1","operations":[{"target":"n.fixed#ListClusters"}],"traits":{"aws.api#service":{"sdkId":"Fixed","endpointPrefix":"fixed","arnNamespace":"fixed"},"aws.auth#sigv4":{"name":"fixed"},"PROTOCOL":{}}},
"n.fixed#ListClusters":{"type":"operation","input":{"target":"n.fixed#Input"},"output":{"target":"n.fixed#Output"},"traits":{"smithy.api#http":{"method":"POST","uri":"/clusters"}}},
"n.fixed#Input":{"type":"structure","members":{"Filters":{"target":"n.fixed#Filters"},"Attributes":{"target":"n.fixed#Attributes"}}},
"n.fixed#Filters":{"type":"list","member":{"target":"n.fixed#Filter","traits":{"smithy.api#xmlName":"Filter"}}},
"n.fixed#Filter":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"Values":{"target":"n.fixed#Values","traits":{"smithy.api#required":{}}},"Threshold":{"target":"n.fixed#SmallCount"},"Ratio":{"target":"n.fixed#Ratio"},"Payload":{"target":"n.fixed#ShortBlob"},"Unicode":{"target":"n.fixed#ShortText"},"Single":{"target":"n.fixed#Single"},"Wide":{"target":"smithy.api#Double"},"Mode":{"target":"n.fixed#Mode"},"Legacy":{"target":"n.fixed#Legacy"},"Label":{"target":"n.fixed#Label","traits":{"smithy.api#pattern":"-7$"}},"Unsupported":{"target":"n.fixed#Unsupported"},"Engine":{"target":"n.fixed#Engine"},"Items":{"target":"n.fixed#Modes"},"Attributes":{"target":"n.fixed#ModeMap"}}},
"n.fixed#Values":{"type":"list","member":{"target":"n.fixed#Engine","traits":{"smithy.api#xmlName":"Value"}}},
"n.fixed#Engine":{"type":"enum","members":{"Neptune":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"neptune"}},"DocDB":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"docdb"}},"Aurora":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"aurora-mysql"}}}},
"n.fixed#SmallCount":{"type":"byte","traits":{"smithy.api#range":{"min":0,"max":7}}},
"n.fixed#Ratio":{"type":"double","traits":{"smithy.api#range":{"min":0,"max":1}}},
"n.fixed#ShortBlob":{"type":"blob","traits":{"smithy.api#length":{"max":2}}},
"n.fixed#ShortText":{"type":"string","traits":{"smithy.api#length":{"max":2}}},
"n.fixed#Single":{"type":"float"},
"n.fixed#Mode":{"type":"intEnum","members":{"One":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":1}},"Seven":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":7}}}},
"n.fixed#Legacy":{"type":"string","traits":{"smithy.api#enum":[{"value":"owned"},{"value":"shared"}]}},
"n.fixed#Label":{"type":"string","traits":{"smithy.api#pattern":"^ok-[0-9]+$"}},
"n.fixed#Unsupported":{"type":"string","traits":{"smithy.api#pattern":"(?=a)"}},
"n.fixed#Attributes":{"type":"map","key":{"target":"n.fixed#Owner"},"value":{"target":"smithy.api#String"}},
"n.fixed#Owner":{"type":"enum","members":{"Owner":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"owner"}},"Secondary":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"secondary"}}}},
"n.fixed#Modes":{"type":"list","member":{"target":"n.fixed#Mode"}},
"n.fixed#ModeMap":{"type":"map","key":{"target":"n.fixed#Owner"},"value":{"target":"n.fixed#Mode"}},
"n.fixed#Output":{"type":"structure","members":{"Items":{"target":"n.fixed#Items"}}},
"n.fixed#Items":{"type":"list","member":{"target":"n.fixed#Cluster"}},
"n.fixed#Cluster":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Engine":{"target":"smithy.api#String"}}}
}}"#;
const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Structured input fixture",
    default_unmapped_reason: None,
    cfn: &[],
    relations: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    relation_targets: &[],
    bootstraps: &[],
    api_only: &[kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::FixedCluster",
        reason: "Synthetic protocol fixture",
    }],
    operations: &[kind_map::OperationMapping {
        service: "fixed",
        operation: "ListClusters",
        kind: "AWS::ApiOnly::FixedCluster",
    }],
};
fn filters(engine: &str) -> Result<overlay::FixedValue, overlay::NameError> {
    Ok(overlay::FixedValue::list([overlay::FixedValue::structure(
        [
            ("Name", overlay::FixedValue::String("engine".into())),
            ("Values", overlay::FixedValue::Strings(vec![engine.into()])),
        ],
    )?]))
}
fn compiled(
    protocol: &str,
    literal: overlay::FixedValue,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    compiled_inputs(
        protocol,
        literal,
        overlay::FixedValue::map([("owner", overlay::FixedValue::String("inventory".into()))])?,
        fixed_endpoint()?,
    )
}
fn compiled_inputs(
    protocol: &str,
    literal: overlay::FixedValue,
    attributes: overlay::FixedValue,
    verified: endpoints::VerifiedEndpoint,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let model = directory.path().join("models/fixed/service/2020-01-01");
    std::fs::create_dir_all(&model)?;
    std::fs::write(
        model.join("model.json"),
        MODEL.replace("PROTOCOL", protocol),
    )?;
    let mut policy = overlay::OperationOverlay::default();
    policy.class = Some(overlay::OverlayClass::Enumerate);
    policy.scope = Some(overlay::Scope::Region);
    policy.items = Some(overlay::ItemSelector::parse("Items")?);
    policy.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    policy.paginator = Some(overlay::Paginator::None);
    policy
        .fixed
        .insert(overlay::MemberPath::new("Filters")?, literal);
    policy
        .fixed
        .insert(overlay::MemberPath::new("Attributes")?, attributes);
    let policies = [overlay::ServiceOverlay::new(
        "fixed",
        vec![("ListClusters", policy)],
    )?];
    let checked = overlay::check_policies(
        directory.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let registry = smithy::Registry::parse(
        "Attributes\nEngine\nFilters\nId\nItems\nLabel\nLegacy\nMode\nName\nPayload\nRatio\nSingle\nThreshold\nUnicode\nUnsupported\nValues\nWide\n",
    )?;
    let base = smithy::build_catalog(directory.path(), &registry, "fixed-input-fixture")?;
    let kinds = KINDS.compile(&names::Exceptions::none())?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &BTreeMap::from([("fixed".to_owned(), verified)]),
    )?)
}
fn fixed_endpoint() -> Result<endpoints::VerifiedEndpoint, Box<dyn std::error::Error>> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"fixed","signingRegion":"us-east-1"}]});
    let endpoint = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://fixed.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("fixed-endpoint.json"), &endpoint)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://fixed.test","properties":properties}}}]),
    )?;
    let verified = rules
        .run_cases(&cases)?
        .verified_endpoint(Some("fixed"))
        .map_err(|error| format!("fixed-input fixture endpoint failed: {error:?}"))?;
    Ok(verified)
}
fn exception_endpoint() -> Result<endpoints::VerifiedEndpoint, Box<dyn std::error::Error>> {
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[
        {"type":"endpoint","conditions":[{"fn":"stringEquals","argv":[{"ref":"Region"},"eu-west-1"]}],"endpoint":{"url":"https://fixed-fips.eu-west-1.test","properties":{"authSchemes":[{"name":"sigv4","signingName":"fixed","signingRegion":"eu-west-1"}]}}},
        {"type":"endpoint","conditions":[],"endpoint":{"url":"https://fixed.{Region}.test","properties":{"authSchemes":[{"name":"sigv4","signingName":"fixed","signingRegion":"{Region}"}]}}}
    ]});
    let rules = endpoints::RuleSet::parse(Path::new("fixed-exception-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([
        {"params":{"Region":"us-west-2"},"expect":{"endpoint":{"url":"https://fixed.us-west-2.test","properties":{"authSchemes":[{"name":"sigv4","signingName":"fixed","signingRegion":"us-west-2"}]}}}},
        {"params":{"Region":"eu-west-1"},"expect":{"endpoint":{"url":"https://fixed-fips.eu-west-1.test","properties":{"authSchemes":[{"name":"sigv4","signingName":"fixed","signingRegion":"eu-west-1"}]}}}}
    ]))?;
    let verified = rules
        .run_cases(&cases)?
        .verified_endpoint(Some("fixed"))
        .map_err(|error| format!("fixed-exception endpoint failed: {error:?}"))?;
    Ok(verified)
}
fn signed(
    protocol: &str,
    engine: &str,
) -> Result<http::Request<bytes::Bytes>, Box<dyn std::error::Error>> {
    signed_literal(protocol, filters(engine)?)
}
fn signed_literal(
    protocol: &str,
    literal: overlay::FixedValue,
) -> Result<http::Request<bytes::Bytes>, Box<dyn std::error::Error>> {
    signed_bytes(&compiled(protocol, literal)?)
}
fn signed_bytes(bytes: &[u8]) -> Result<http::Request<bytes::Bytes>, Box<dyn std::error::Error>> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let plan = catalog
        .lookup("fixed", "ListClusters")
        .ok_or("missing plan")?;
    let base = root_bindings(plan, None, None)?;
    let mut request = encode(plan, "https://fixed.test", &base, &[])?;
    let credentials = aws_wire::sign::Credentials::new("AKIDEXAMPLE", "example-secret", None)?;
    let scope = aws_wire::sign::Scope {
        region: aws_wire::sign::Region::new("us-east-1")?,
        name: aws_wire::sign::SigningName::new("fixed")?,
    };
    let uri = request.uri().to_string();
    let headers = aws_wire::sign::sign(
        &credentials,
        &scope,
        SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_700_000_000))
            .ok_or("invalid signing time")?,
        aws_wire::sign::Request {
            method: request.method(),
            uri: &uri,
            headers: request.headers(),
            body: aws_wire::sign::Body::Bytes(request.body()),
            payload_header: aws_wire::sign::PayloadHeader::Include,
        },
        aws_wire::sign::PathMode::Standard,
    )?;
    request.headers_mut().extend(headers);
    Ok(request)
}

fn with_field(
    name: &str,
    value: overlay::FixedValue,
) -> Result<overlay::FixedValue, overlay::NameError> {
    Ok(overlay::FixedValue::list([overlay::FixedValue::structure(
        [
            ("Name", overlay::FixedValue::String("engine".into())),
            (
                "Values",
                overlay::FixedValue::Strings(vec!["neptune".into()]),
            ),
            (name, value),
        ],
    )?]))
}
fn assert_invalid_value(error: &(dyn std::error::Error + 'static)) -> TestResult {
    let Some(overlay::CheckError::Violations(violations)) =
        error.downcast_ref::<overlay::CheckError>()
    else {
        return Err(format!("expected InvalidValue, got {error}").into());
    };
    assert!(
        violations
            .iter()
            .any(|violation| violation.rule() == overlay::ViolationRule::InvalidValue)
    );
    Ok(())
}

fn map_mut(value: &mut Value) -> Result<&mut BTreeMap<Key, Value>, Box<dyn std::error::Error>> {
    match value {
        Value::Map(values) => Ok(values),
        _ => Err("map absent".into()),
    }
}
fn field_mut(value: &mut Value, id: u32) -> Result<&mut Value, Box<dyn std::error::Error>> {
    map_mut(value)?
        .get_mut(&Key::Field(FieldId::new(id)?))
        .ok_or_else(|| "field absent".into())
}
fn first_mut(value: &mut Value) -> Result<&mut Value, Box<dyn std::error::Error>> {
    match value {
        Value::List(values) => values.first_mut().ok_or_else(|| "empty list".into()),
        _ => Err("list absent".into()),
    }
}
fn reseal(root: &mut Value) -> TestResult {
    let record = Record::encode(root)?;
    let extension = record
        .root()
        .get_field(FieldId::new(60_000)?)?
        .ok_or("extension absent")?;
    let inventory_core::record::ValueRef::Map(extension) = extension else {
        return Err("extension not a map".into());
    };
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(extension)?;
    let provenance = field_mut(field_mut(root, 60_000)?, 7)?;
    *field_mut(provenance, 7)? = Value::Bytes(digest.to_vec());
    Ok(())
}

#[path = "fixed_input_constraint_tests.rs"]
mod constraint_tests;
#[path = "fixed_input_enum_tests.rs"]
mod enum_tests;
#[path = "fixed_input_wire_tests.rs"]
mod wire_tests;
