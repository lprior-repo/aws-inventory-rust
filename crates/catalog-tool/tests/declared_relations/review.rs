use super::*;
use fixture::OTHER_SOURCE;

#[test]
fn given_arn_string_and_key_target_when_compiled_then_the_declaration_is_pending() -> TestResult {
    let (bytes, report) = compile(TargetState::ArnKey, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships().len(), 2);
    assert_eq!(report.declared, 2);
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} ArnRef {TARGET} - REFERENCES (ARN reference requires an ARN identity)"
        )),
        Some(&"ARN reference requires an ARN identity".to_owned())
    );
    Ok(())
}

#[test]
fn given_arn_string_and_arn_target_when_compiled_then_the_declaration_retains_model_provenance()
-> TestResult {
    let (bytes, report) = compile(TargetState::ArnString, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let relations = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| {
            (
                rule.target_kind.as_str(),
                rule.target_path.to_vec(),
                rule.edge_source,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        relations,
        [(TARGET, vec![FieldId::new(3)?], EdgeSource::Model)]
    );
    assert_eq!(report.declared, 3);
    Ok(())
}

#[test]
fn given_reviewed_rule_and_unbindable_declaration_when_compiled_then_no_pending_duplicate_is_reported()
-> TestResult {
    let (bytes, report) = compile(TargetState::Compound, &[reviewed_rule("ArnRef")])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let sources = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| rule.edge_source)
        .collect::<Vec<_>>();
    assert_eq!(sources, [EdgeSource::ReviewedRule]);
    assert_eq!((report.reviewed, report.declared), (1, 2));
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} ArnRef {TARGET} - REFERENCES (ARN reference requires an ARN identity)"
        )),
        None
    );
    Ok(())
}

#[test]
fn given_reviewed_rule_for_a_different_target_when_compiled_then_the_model_edge_remains()
-> TestResult {
    let mut rule = reviewed_rule("ArnRef");
    rule.target_kind = SOURCE;
    let (bytes, report) = compile(TargetState::Callable, &[rule])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let targets = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| {
            (
                rule.target_kind.as_str(),
                rule.relation.as_str(),
                rule.edge_source,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        targets,
        [
            (SOURCE, "REVIEWED_REFERENCE", EdgeSource::ReviewedRule),
            (TARGET, "REFERENCES", EdgeSource::Model),
        ]
    );
    assert_eq!((report.reviewed, report.declared), (1, 3));
    Ok(())
}

#[test]
fn given_nine_member_reference_path_when_compiled_then_the_path_has_a_named_pending_reason()
-> TestResult {
    let (bytes, report) = compile(TargetState::DeepReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships().len(), 3);
    assert_eq!((report.declared, report.inactive.len()), (3, 3));
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} Other.Nested.Nested.Nested.Nested.Nested.Nested.Nested.Id n#Resource - REFERENCES (reference path exceeds eight members)"
        )).map(String::as_str),
        Some("reference path exceeds eight members")
    );
    Ok(())
}

#[test]
fn given_two_source_kinds_with_identical_declarations_when_compiled_then_both_sources_are_retained()
-> TestResult {
    let (bytes, report) = compile(TargetState::TwoSources, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let mut sources = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| {
            (
                rule.source_kind.as_str(),
                rule.target_kind.as_str(),
                rule.edge_source,
            )
        })
        .collect::<Vec<_>>();
    sources.sort_unstable_by_key(|(source, target, _)| (*source, *target));
    assert_eq!(
        sources,
        [
            (OTHER_SOURCE, TARGET, EdgeSource::Model),
            (SOURCE, TARGET, EdgeSource::Model),
        ]
    );
    assert_eq!(report.declared, 6);
    Ok(())
}

#[test]
fn given_eight_member_reference_path_when_compiled_then_the_boundary_path_remains_live()
-> TestResult {
    let (bytes, report) = compile(TargetState::BoundaryReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let paths = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.len() == 8)
        .map(|rule| (rule.source_path.to_vec(), rule.edge_source))
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [(
            vec![
                FieldId::new(11)?,
                FieldId::new(5)?,
                FieldId::new(5)?,
                FieldId::new(5)?,
                FieldId::new(5)?,
                FieldId::new(5)?,
                FieldId::new(5)?,
                FieldId::new(3)?,
            ],
            EdgeSource::Model,
        )]
    );
    assert_eq!((report.declared, report.inactive.len()), (4, 2));
    Ok(())
}
