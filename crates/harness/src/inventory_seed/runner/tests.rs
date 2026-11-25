use super::*;
use crate::inventory_seed::model::{
    PreparedIdentity, PreparedNestedBinding, PreparedResource, PreparedTemplate, Selector,
};
use crate::inventory_seed::{SeedIdentityEvidence, SeedScope, StepId};
use inventory_core::record::Key;
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn state() -> Result<SeedState, SeedError> {
    Ok(SeedState::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        String::from("aws"),
        1,
    ))
}

fn spec(identity: PreparedIdentity, scope: SeedScope) -> Result<PreparedResource, SeedError> {
    Ok(PreparedResource {
        kind: Kind::parse("AWS::Example::Resource")?,
        identity,
        scope,
        expected_identity: None,
    })
}

fn observed() -> Result<Record, SeedError> {
    wire::input_record([
        (
            vec![FieldId::new(1)?],
            Value::Map(BTreeMap::from([(
                Key::Name(String::from("QueueArn")),
                Value::Text(String::from("observed-arn")),
            )])),
        ),
        (
            vec![FieldId::new(2)?],
            Value::List(vec![Value::Map(BTreeMap::from([(
                Key::Field(FieldId::new(3)?),
                Value::Text(String::from("observed-child")),
            )]))]),
        ),
    ])
}

#[test]
fn given_account_scope_when_response_identity_recorded_then_key_is_global_and_transport_region_stays()
-> TestResult {
    let mut state = state()?;
    let spec = spec(
        PreparedIdentity::Response(vec![Selector::Field(FieldId::new(1)?)]),
        SeedScope::Account,
    )?;
    let record = wire::input_record([(
        vec![FieldId::new(1)?],
        Value::Text(String::from("observed-id")),
    )])?;
    let step = StepId::new(0);
    let resources = identity::resources(&state, step, &spec, &record, &[])?;
    let [resource] = resources.as_slice() else {
        return Err("expected one response identity".into());
    };
    assert_eq!(resource.key.region, Region::Global);
    assert_eq!(resource.id.as_str(), "observed-id");
    assert_eq!(resource.evidence, SeedIdentityEvidence::Response);
    record_output(
        &mut state,
        SeedOutput {
            step,
            resources,
            record,
        },
    )?;
    let oracle = state.generic_oracle(&[])?;
    assert_eq!(
        oracle
            .resources
            .first()
            .ok_or("missing resource")?
            .key
            .region,
        Region::Global
    );
    assert_eq!(oracle.region.as_str(), "us-east-1");
    Ok(())
}

#[test]
fn given_unit_response_when_request_identity_recorded_then_explicit_provenance_and_context_are_used()
-> TestResult {
    let mut state = state()?;
    let spec = spec(
        PreparedIdentity::Request {
            input_field: vec![FieldId::new(2)?],
            template: Some(String::from(
                "arn:{partition}:example:{region}:{account}:{value}",
            )),
        },
        SeedScope::Regional,
    )?;
    let record = wire::input_record(std::iter::empty::<(Vec<FieldId>, Value)>())?;
    let inputs = [(
        vec![FieldId::new(2)?],
        Value::Text(String::from("alias/requested")),
    )];
    let step = StepId::new(0);
    let resources = identity::resources(&state, step, &spec, &record, &inputs)?;
    let [resource] = resources.as_slice() else {
        return Err("expected one request identity".into());
    };
    assert_eq!(
        resource.id.as_str(),
        "arn:aws:example:us-east-1:123456789012:alias/requested"
    );
    assert_eq!(resource.evidence, SeedIdentityEvidence::Request);
    assert_eq!(resource.key.region.as_str(), "us-east-1");
    record_output(
        &mut state,
        SeedOutput {
            step,
            resources,
            record,
        },
    )?;
    assert_eq!(state.generic_oracle(&[])?.resources.len(), 1);
    Ok(())
}

#[test]
fn given_auxiliary_response_when_selected_then_map_names_and_list_indices_return_actual_values()
-> TestResult {
    let mut state = state()?;
    record_output(
        &mut state,
        SeedOutput {
            step: StepId::new(0),
            resources: SeedResources::None,
            record: observed()?,
        },
    )?;
    let map_path = [
        Selector::Field(FieldId::new(1)?),
        Selector::Name(String::from("QueueArn")),
    ];
    let list_path = [
        Selector::Field(FieldId::new(2)?),
        Selector::Index(0),
        Selector::Field(FieldId::new(3)?),
    ];
    assert_eq!(
        materialize::output_value(&state, StepId::new(0), &map_path)?,
        Value::Text(String::from("observed-arn"))
    );
    assert_eq!(
        materialize::output_value(&state, StepId::new(0), &list_path)?,
        Value::Text(String::from("observed-child"))
    );
    assert!(
        materialize::output_value(&state, StepId::new(0), &[Selector::Field(FieldId::new(9)?)])
            .is_err()
    );
    assert!(
        materialize::output_value(
            &state,
            StepId::new(0),
            &[Selector::Field(FieldId::new(2)?), Selector::Index(1)]
        )
        .is_err()
    );
    assert!(state.generic_oracle(&[])?.resources.is_empty());
    Ok(())
}

#[test]
fn given_nested_list_placeholder_when_materialized_then_encoded_request_contains_observed_output()
-> TestResult {
    let mut state = state()?;
    record_output(
        &mut state,
        SeedOutput {
            step: StepId::new(0),
            resources: SeedResources::None,
            record: observed()?,
        },
    )?;
    let field = FieldId::new(4)?;
    let template = PreparedTemplate {
        value: Value::Map(BTreeMap::from([(
            Key::Field(field),
            Value::List(vec![Value::Null]),
        )])),
        bindings: vec![PreparedNestedBinding {
            destination: vec![Selector::Field(field), Selector::Index(0)],
            step: StepId::new(0),
            path: vec![
                Selector::Field(FieldId::new(2)?),
                Selector::Index(0),
                Selector::Field(FieldId::new(3)?),
            ],
        }],
    };
    let input = PreparedInput {
        path: vec![FieldId::new(5)?],
        value: PreparedInputValue::Template(template),
    };
    let record = wire::input_record([materialize_input(&state, &input)?])?;
    let path = [
        Selector::Field(FieldId::new(5)?),
        Selector::Field(field),
        Selector::Index(0),
    ];
    assert_eq!(text_path(record.root(), &path)?, "observed-child");
    Ok(())
}

#[test]
fn given_invalid_caller_context_when_parsed_then_account_or_partition_cannot_be_fabricated()
-> TestResult {
    assert_eq!(
        identity::caller_identity("123456789012", "arn:aws:iam::123456789012:user/test")?,
        (AccountId::parse("123456789012")?, String::from("aws"))
    );
    assert!(
        identity::caller_identity("123456789012", "arn:aws:iam::999999999999:user/test").is_err()
    );
    assert!(identity::caller_identity("123456789012", "arn::iam::123456789012:user/test").is_err());
    assert!(identity::caller_identity("123456789012", "not-an-arn").is_err());
    Ok(())
}

#[test]
fn given_invalid_request_identity_when_materialized_then_missing_or_nontext_values_are_rejected()
-> TestResult {
    let state = state()?;
    let spec = spec(
        PreparedIdentity::Request {
            input_field: vec![FieldId::new(2)?],
            template: None,
        },
        SeedScope::Regional,
    )?;
    let record = wire::input_record(std::iter::empty::<(Vec<FieldId>, Value)>())?;
    assert!(identity::resources(&state, StepId::new(0), &spec, &record, &[]).is_err());
    assert!(
        identity::resources(
            &state,
            StepId::new(0),
            &spec,
            &record,
            &[(vec![FieldId::new(2)?], Value::Int(7))]
        )
        .is_err()
    );
    Ok(())
}
