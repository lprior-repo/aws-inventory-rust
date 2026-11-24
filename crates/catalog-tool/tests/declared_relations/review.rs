use super::*;
use fixture::OTHER_SOURCE;

#[test]
fn given_arn_string_and_key_target_when_compiled_then_the_declaration_is_pending() -> TestResult {
    // Given an ARN-reference string whose resource is identified by a key instead of its ARN.
    let (bytes, report) = compile(TargetState::ArnKey, &[])?;
    // When compiling, then the ARN cannot be used as the target's key identity.
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
    // Given the same trait on a named string and an ARN-identified callable resource.
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
    // Then the exact identity selector and source are preserved by the runtime reader.
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
    // Given review of the ARN path and a declared target with a compound key identity.
    let (bytes, report) = compile(TargetState::Compound, &[reviewed_rule("ArnRef")])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let sources = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| rule.edge_source)
        .collect::<Vec<_>>();
    // Then review wins before declaration binding can create a spurious coverage gap.
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
    // Given the same source path, but a reviewed rule that names another active target kind.
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
    // Then suppression is keyed by the source path AND target kind.
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
    // Given eight nested structures followed by their identifier member.
    let (bytes, report) = compile(TargetState::DeepReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    // Then the ninth member is pending, while the ordinary fixture references remain live.
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
    // Given two callable kinds sharing the same item shape, target kind and member paths.
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
    // Then the source kind remains part of the deduplication key.
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
    // Given a reference at the maximum supported member-path depth.
    let (bytes, report) = compile(TargetState::BoundaryReference, &[])?;
    // When reading the catalog, then all eight members form one live model selector.
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
