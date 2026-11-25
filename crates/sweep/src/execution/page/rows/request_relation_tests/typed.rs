use super::*;
#[test]
fn given_guarded_membership_when_null_empty_or_hostile_types_arrive_then_only_exact_text_members_emit()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationGuard, RelationPathMode},
        record::FieldId,
    };
    let mut rule = typed_rule()?;
    rule.guard = Some(RelationGuard {
        source: RelationSource::Request,
        path_mode: RelationPathMode::Fields,
        path: vec![FieldId::new(2)?].into_boxed_slice(),
        equals: "active".into(),
    });
    let observation = typed_observation(&rule)?;
    let row = typed_record([(
        1,
        Value::List(vec![
            Value::Null,
            Value::Text("".into()),
            Value::Text("one".into()),
            Value::Text("two".into()),
        ]),
    )])?;
    let home = RegionId::parse("us-east-1")?;
    let request = typed_record([(2, Value::Text("active".into()))])?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 10)?;
    assert_eq!(
        output
            .iter()
            .map(|reference| reference.target.value().as_str())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
    for value in [Value::Null, Value::Text("inactive".into())] {
        let request = typed_record([(2, value)])?;
        let mut output = Vec::new();
        super::relations::Context {
            row: &row,
            request: &request,
            observation: &observation,
            home: &home,
            parent: None,
        }
        .append(&rule, &mut output, 10)?;
        assert_eq!(output.len(), 0);
    }
    let request = typed_record([(2, Value::Bool(true))])?;
    let error = super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut Vec::new(), 10);
    assert!(matches!(
        error,
        Err(SweepError::Operation(
            "relationship guard is not modeled text"
        ))
    ));
    Ok(())
}

#[test]
fn given_root_and_compound_selectors_when_extracted_then_namespaces_and_conjunction_survive()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationPathMode, RelationTargetMode, RelationshipComponentPlan},
        record::{FieldId, Key},
    };
    let mut rule = typed_rule()?;
    rule.source_path = Box::new([]);
    rule.source_path_mode = RelationPathMode::RootText;
    rule.target_mode = RelationTargetMode::CanonicalId;
    rule.target_path = Box::new([]);
    rule.components = vec![RelationshipComponentPlan {
        source: RelationSource::Request,
        source_path: vec![FieldId::new(2)?].into_boxed_slice(),
        source_path_mode: RelationPathMode::Fields,
        target_mode: RelationTargetMode::BoundContextFields,
        target_path: vec![FieldId::new(3)?].into_boxed_slice(),
    }]
    .into_boxed_slice();
    let row = Record::encode(&Value::Map(std::collections::BTreeMap::from([(
        Key::Name("$value".into()),
        Value::Text("exact-id".into()),
    )])))?;
    let request = typed_record([(2, Value::Text("context-operand".into()))])?;
    let observation = typed_observation(&rule)?;
    let home = RegionId::parse("us-east-1")?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    let reference = output.first().ok_or("root reference absent")?;
    assert_eq!(reference.target.value().as_str(), "exact-id");
    assert_eq!(reference.target.mode(), RelationTargetMode::CanonicalId);
    let term = reference
        .target
        .components()
        .first()
        .ok_or("compound term absent")?;
    assert_eq!(term.mode, RelationTargetMode::BoundContextFields);
    assert_eq!(term.path.as_ref(), &[FieldId::new(3)?]);
    assert_eq!(term.value.as_str(), "context-operand");
    let missing = typed_record([])?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &missing,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(output.len(), 0);
    Ok(())
}

#[test]
fn given_parent_rule_when_selected_decoy_differs_then_only_authoritative_parent_scope_is_used()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationPathMode, RelationTargetMode},
        inventory::{Kind, ResourceId},
    };
    let mut rule = typed_rule()?;
    rule.source = RelationSource::Parent;
    rule.source_path = Box::new([]);
    rule.source_path_mode = RelationPathMode::RootText;
    rule.target_mode = RelationTargetMode::CanonicalId;
    rule.target_path = Box::new([]);
    rule.target_kind = Kind::parse("AWS::VpcLattice::Service")?;
    let network = Kind::parse("AWS::VpcLattice::ServiceNetwork")?;
    rule.target_kinds = vec![rule.target_kind.clone(), network.clone()].into_boxed_slice();
    let observation = typed_observation(&rule)?;
    let mut parent = observation.key().clone();
    parent.id = ResourceId::parse("authoritative-parent")?;
    parent.kind = network;
    let row = typed_record([(1, Value::Text("selected-decoy".into()))])?;
    let request = typed_record([])?;
    let home = RegionId::parse("us-east-1")?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: Some(&parent),
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(
        output
            .first()
            .ok_or("parent reference absent")?
            .target
            .value()
            .as_str(),
        "authoritative-parent"
    );
    let target = &output.first().ok_or("parent reference absent")?.target;
    assert_eq!(target.kind(), &parent.kind);
    assert_ne!(target.kind(), &rule.target_kind);
    assert_eq!(target.candidate_kinds(), rule.target_kinds.as_ref());
    assert!(matches!(
        super::relations::Context {
            row: &row,
            request: &request,
            observation: &observation,
            home: &home,
            parent: Some(&parent)
        }
        .append(&rule, &mut output, 1),
        Err(SweepError::Operation(
            "page exceeds the atomic reference bound"
        ))
    ));
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(output.len(), 0);
    Ok(())
}
