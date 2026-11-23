use super::*;
use crate::inventory_seed::scenario::StepId;
use crate::inventory_seed::{SeedIdentityEvidence, model::Selector};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn output(
    state: &SeedState,
    step: u8,
    identity: &str,
    observed: &str,
) -> Result<SeedOutput, SeedError> {
    let kind = Kind::parse("AWS::Example::Resource")?;
    let identity = ResourceId::parse(identity)?;
    let key = ResourceKey {
        account: state.account.clone(),
        region: state.region.clone(),
        kind: kind.clone(),
        id: identity.clone(),
    };
    let record = wire::input_record([(vec![FieldId::new(1)?], Value::Text(observed.to_owned()))])?;
    let resource = SeedResourceIdentity {
        step: StepId::new(step as u16),
        kind,
        id: identity,
        key,
        evidence: SeedIdentityEvidence::Response,
    };
    Ok(SeedOutput {
        step: StepId::new(step as u16),
        resource: Some(resource),
        record,
    })
}

fn state(observed: &str) -> Result<SeedState, SeedError> {
    let mut state = SeedState::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        String::from("aws"),
    );
    state
        .outputs
        .push(output(&state, 0, "target-id", "target-id")?);
    state
        .outputs
        .push(output(&state, 1, "source-id", observed)?);
    Ok(state)
}

fn relationship() -> Result<PreparedRelationship, SeedError> {
    Ok(PreparedRelationship {
        source: StepId::new(1),
        target: StepId::new(0),
        relation: inventory_core::inventory::Relation::parse("RELATED_TO")?,
        observed_path: vec![Selector::Field(FieldId::new(1)?)],
    })
}

#[test]
fn given_non_ec2_resources_when_oracle_built_then_observed_edges_and_exact_keys_are_returned()
-> TestResult {
    let oracle = state("target-id")?.generic_oracle(&[relationship()?])?;
    assert_eq!(oracle.resources.len(), 2);
    let edge = oracle.edges.first().ok_or("missing verified edge")?;
    assert_eq!(edge.from.id.as_str(), "source-id");
    assert_eq!(edge.to.id.as_str(), "target-id");
    assert_eq!(edge.relation.as_str(), "RELATED_TO");
    assert_eq!(edge.source, EdgeSource::ReviewedRule);
    assert_eq!(
        oracle.resources.first().ok_or("missing resource")?.key,
        edge.to
    );
    Ok(())
}

#[test]
fn given_mismatched_or_partial_observed_identity_when_oracle_built_then_edge_is_rejected()
-> TestResult {
    for observed in ["target", "target-id-extra", "other-id"] {
        assert!(state(observed)?.generic_oracle(&[relationship()?]).is_err());
    }
    Ok(())
}

#[test]
fn given_single_resource_without_relations_when_oracle_built_then_resources_are_retained()
-> TestResult {
    let mut state = state("target-id")?;
    state.outputs.truncate(1);
    let oracle = state.generic_oracle(&[])?;
    assert_eq!(oracle.resources.len(), 1);
    assert!(oracle.edges.is_empty());
    Ok(())
}

#[test]
fn given_expected_identity_when_checked_then_only_exact_equality_is_accepted() -> TestResult {
    let identity = ResourceId::parse("target-id")?;
    assert!(validate_exact_identity(Some("target-id"), &identity).is_ok());
    for expected in ["target", "target-id-extra", "other-id"] {
        assert!(validate_exact_identity(Some(expected), &identity).is_err());
    }
    Ok(())
}

#[test]
fn given_earlier_output_reference_when_materialized_then_observed_value_is_used() -> TestResult {
    let state = state("target-id")?;
    let path = vec![FieldId::new(2)?];
    let input = PreparedInput {
        path: path.clone(),
        value: PreparedInputValue::PreviousOutput {
            step: StepId::new(0),
            path: vec![Selector::Field(FieldId::new(1)?)],
        },
    };
    assert_eq!(
        materialize_input(&state, &input)?,
        (path, Value::Text(String::from("target-id")))
    );
    let missing = PreparedInput {
        path: vec![FieldId::new(2)?],
        value: PreparedInputValue::PreviousOutput {
            step: StepId::new(2),
            path: vec![Selector::Field(FieldId::new(1)?)],
        },
    };
    assert!(materialize_input(&state, &missing).is_err());
    Ok(())
}
