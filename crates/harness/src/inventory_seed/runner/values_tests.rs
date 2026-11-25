use super::*;
use crate::inventory_seed::{
    FieldPath, StepId, load,
    model::{Selector, output_path},
};
use aws_wire::request::{CompressionPolicy, RequestContext, RequestLimits};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const EC2: &str = "format = 1\nservice = 'ec2'\n[[step]]\nid = 'vpc'\noperation = 'CreateVpc'\nkind = 'AWS::EC2::VPC'\nidentity = 'Vpc.VpcId'\n[[step]]\nid = 'instance'\noperation = 'RunInstances'\nkind = ''\n[step.input]\nImageId = 'ami-example'\nMinCount = 1\nMaxCount = 1\nSecurityGroupIds = ['sg-first', 'sg-second']\nTagSpecifications = [{ResourceType = 'instance', Tags = [{Key = 'parent', Value = {from = 'vpc', path = 'Vpc.VpcId'}}]}]";

fn bytes() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )
}

fn state() -> Result<SeedState, SeedError> {
    Ok(SeedState::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        "aws".into(),
        1,
    ))
}

fn parent_output(step: &PreparedStep<'_>) -> Result<Record, SeedError> {
    let path = output_path(
        step.operation.service,
        step.operation.operation.output()?,
        &FieldPath::new(&["Vpc", "VpcId"])?,
    )?;
    let fields = path
        .iter()
        .map(|selector| match selector {
            Selector::Field(field) => Ok(*field),
            _ => Err(SeedError::Invalid("expected modeled response fields")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    wire::input_record([(fields, Value::Text("vpc-observed".into()))])
}

fn request(
    step: &PreparedStep<'_>,
    state: &SeedState,
) -> Result<(Record, String), Box<dyn std::error::Error>> {
    let inputs = step
        .inputs
        .iter()
        .map(|input| materialize_input(state, input))
        .collect::<Result<Vec<_>, _>>()?;
    let record = wire::input_record(inputs)?;
    let request = aws_wire::request::query::encode(RequestContext {
        service: step.operation.service,
        operation: step.operation.operation,
        protocol: step.operation.protocol,
        input: record.root(),
        endpoint: "https://localhost",
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    })?;
    let body = std::str::from_utf8(request.body())?.to_owned();
    Ok((record, body))
}

#[test]
fn given_toml_lists_and_structures_when_prepared_then_request_uses_modeled_fields_and_observed_binding()
-> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = load(EC2)?;
    let models = SeedModels::load(&runtime, &scenario)?;
    let parent = models.steps.first().ok_or("missing parent")?;
    let mut state = state()?;
    record_output(
        &mut state,
        SeedOutput {
            step: StepId::new(0),
            resources: SeedResources::None,
            record: parent_output(parent)?,
        },
    )?;
    let step = models.steps.get(1).ok_or("missing instance")?;
    let (record, body) = request(step, &state)?;
    let path = output_path(
        step.operation.service,
        step.operation.operation.input()?,
        &FieldPath::new(&["TagSpecifications", "0", "Tags", "0", "Value"])?,
    )?;
    assert_eq!(text_path(record.root(), &path)?, "vpc-observed");
    assert!(body.contains("vpc-observed"));
    assert!(body.contains("sg-first"));
    assert!(body.contains("sg-second"));
    Ok(())
}

#[test]
fn given_toml_map_when_prepared_then_request_preserves_named_keys() -> TestResult {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = load(
        "format = 1\nservice = 'sns'\n[[step]]\nid = 'topic'\noperation = 'CreateTopic'\nkind = ''\n[step.input]\nName = 'seed'\nAttributes = {'seed-label:key' = 'display-seed'}",
    )?;
    let models = SeedModels::load(&runtime, &scenario)?;
    let step = models.steps.first().ok_or("missing topic")?;
    let (record, body) = request(step, &state()?)?;
    let mut path = output_path(
        step.operation.service,
        step.operation.operation.input()?,
        &FieldPath::new(&["Attributes"])?,
    )?;
    path.push(Selector::Name("seed-label:key".into()));
    assert_eq!(text_path(record.root(), &path)?, "display-seed");
    assert!(body.contains("seed-label%3Akey"));
    assert!(body.contains("display-seed"));
    Ok(())
}

fn blob_request(encoded: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let bytes = bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let document = format!(
        "format=1\nservice='translate'\n[[step]]\nid='import'\noperation='ImportTerminology'\nkind=''\n[step.input]\nName='inventory-blob'\nMergeStrategy='OVERWRITE'\nTerminologyData={{File='{encoded}',Format='CSV'}}"
    );
    let scenario = load(&document)?;
    let models = SeedModels::load(&runtime, &scenario)?;
    let step = models.steps.first().ok_or("missing import step")?;
    let state = state()?;
    let inputs = step
        .inputs
        .iter()
        .map(|input| materialize_input(&state, input))
        .collect::<Result<Vec<_>, _>>()?;
    let record = wire::input_record(inputs)?;
    json_request(step, &record)
}

fn json_request(
    step: &PreparedStep<'_>,
    record: &Record,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let request = aws_wire::request::body::encode(RequestContext {
        service: step.operation.service,
        operation: step.operation.operation,
        protocol: step.operation.protocol,
        input: record.root(),
        endpoint: "https://localhost",
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    })?;
    Ok(serde_json::from_slice(request.body())?)
}

#[test]
fn given_toml_blob_when_materialized_and_encoded_then_wire_has_single_base64_and_unchanged_text()
-> TestResult {
    let request = blob_request("AP8=")?;
    assert_eq!(
        request.pointer("/TerminologyData/File"),
        Some(&serde_json::json!("AP8="))
    );
    assert_eq!(
        request.pointer("/TerminologyData/Format"),
        Some(&serde_json::json!("CSV"))
    );
    assert_eq!(
        request.get("Name"),
        Some(&serde_json::json!("inventory-blob"))
    );
    Ok(())
}
