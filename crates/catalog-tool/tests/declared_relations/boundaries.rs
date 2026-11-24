use super::*;
use fixture::OTHER_SOURCE;

#[test]
fn given_reference_structure_beyond_the_boundary_when_compiled_then_its_declaration_is_pending()
-> TestResult {
    // Given a referenced structure reached by a ninth member, with a tenth identifier member.
    let (bytes, report) = compile(TargetState::DeepStructureReference, &[])?;
    // When compiling, then the supported declarations survive and the deeper one is reported.
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships().len(), 3);
    assert_eq!((report.declared, report.inactive.len()), (3, 3));
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} Other.Nested.Nested.Nested.Nested.Nested.Nested.Nested.Nested.Id n#Resource - REFERENCES (reference path exceeds eight members)"
        )).map(String::as_str),
        Some("reference path exceeds eight members")
    );
    Ok(())
}

#[test]
fn given_reference_beyond_a_truncated_subtree_when_compiled_then_the_cut_path_names_its_target()
-> TestResult {
    // Given another structure between the depth boundary and the reference-bearing shape.
    let (bytes, report) = compile(TargetState::TruncatedStructureReference, &[])?;
    // Then pruning the subtree preserves an explicit pending entry for its reachable resource.
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships().len(), 3);
    assert_eq!((report.declared, report.inactive.len()), (3, 3));
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} Other.Nested.Nested.Nested.Nested.Nested.Nested.Nested.Nested n#Resource - REFERENCES (reference path exceeds eight members)"
        )).map(String::as_str),
        Some("reference path exceeds eight members")
    );
    Ok(())
}

#[test]
fn given_recursive_reference_structure_when_compiled_then_the_cycle_is_explicitly_pending()
-> TestResult {
    // Given a source whose Other member recursively points to the reference-bearing source.
    let (bytes, report) = compile(TargetState::CyclicReference, &[])?;
    // Then no recursive edge is fabricated and the stopped recursive declarations are reported.
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships().len(), 3);
    assert_eq!(report.declared, 3);
    assert_eq!(
        report
            .inactive
            .get(&format!(
                "pending {SOURCE} Other.IdRef n#Resource - REFERENCES (cyclic reference path)"
            ))
            .map(String::as_str),
        Some("cyclic reference path")
    );
    assert_eq!(
        report
            .inactive
            .values()
            .filter(|reason| reason.as_str() == "cyclic reference path")
            .count(),
        4
    );
    Ok(())
}

#[test]
fn given_reference_bearing_fanout_when_the_walk_budget_is_exhausted_then_the_error_names_the_budget()
-> TestResult {
    // Given shared reference-bearing shapes with four routes per level.
    let error = compile(TargetState::FanoutReference, &[])
        .err()
        .ok_or("expected declaration walk exhaustion")?;
    // Then walk exhaustion is not misreported as malformed capability metadata.
    assert_eq!(
        error.to_string(),
        "model reference walk exceeds 65536 shape visits"
    );
    Ok(())
}

#[test]
fn given_string_reference_with_ids_when_compiled_then_invalid_input_names_the_shape() -> TestResult
{
    // Given ids on a string, where Smithy permits ids only on structures.
    let error = compile(TargetState::InvalidStringIds, &[])
        .err()
        .ok_or("expected invalid string reference ids")?;
    // Then no invented ArnRef.Id selector reaches schema or relation generation.
    assert_eq!(
        error.to_string(),
        "reference declaration on shape ReferenceArn is invalid: ids require a structure"
    );
    Ok(())
}

#[test]
fn given_smithy_string_reference_to_a_key_when_compiled_then_the_reason_does_not_claim_an_arn_trait()
-> TestResult {
    // Given smithy.api#references on a string rather than aws.api#arnReference.
    let (bytes, report) = compile(TargetState::SmithyStringKey, &[])?;
    // Then the unbindable declaration has the generic identity diagnostic.
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships().len(), 2);
    assert_eq!(
        report.inactive.get(&format!(
            "pending {SOURCE} ArnRef {TARGET} - REFERENCES (target identity members cannot be bound)"
        )).map(String::as_str),
        Some("target identity members cannot be bound")
    );
    Ok(())
}

#[test]
fn given_two_trait_failures_on_one_path_when_reported_then_both_pending_reasons_survive()
-> TestResult {
    // Given both traits on the same string, referring to the same key-identified resource.
    let (bytes, report) = compile(TargetState::CollidingPending, &[])?;
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships().len(), 2);
    // Then neither declaration overwrites the other declaration's distinct diagnostic.
    let reasons = report
        .inactive
        .iter()
        .filter(|(key, _)| {
            key.starts_with(&format!("pending {SOURCE} ArnRef {TARGET} - REFERENCES"))
        })
        .map(|(_, reason)| reason.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        reasons,
        [
            "ARN reference requires an ARN identity",
            "target identity members cannot be bound",
        ]
    );
    assert_eq!(report.inactive.len(), 4);
    Ok(())
}

#[test]
fn given_reviewed_noncallable_target_when_compiled_then_the_declared_pending_duplicate_is_suppressed()
-> TestResult {
    // Given an explicit reviewed rule for the same path and deferred target kind.
    let (bytes, report) = compile(TargetState::Deferred, &[reviewed_rule("ArnRef")])?;
    // Then review owns that coverage gap even though neither relationship is live.
    assert_eq!(RuntimeCatalog::from_bytes(&bytes)?.relationships(), &[]);
    assert_eq!(
        (report.reviewed, report.declared, report.inactive.len()),
        (0, 0, 5)
    );
    assert_eq!(
        report.inactive.keys()
            .filter(|key| key.starts_with(&format!("pending {SOURCE} ArnRef {TARGET} - REFERENCES")))
            .collect::<Vec<_>>(),
        Vec::<&String>::new()
    );
    Ok(())
}

#[test]
fn given_review_for_one_source_kind_when_compiled_then_the_other_source_keeps_its_model_edge()
-> TestResult {
    // Given shared item shapes and paths, with review naming only SOURCE, not OTHER_SOURCE.
    let (bytes, report) = compile(TargetState::TwoSources, &[reviewed_rule("ArnRef")])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let mut edges = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| {
            (
                rule.source_kind.as_str(),
                rule.relation.as_str(),
                rule.edge_source,
            )
        })
        .collect::<Vec<_>>();
    edges.sort_unstable_by_key(|(source, _, _)| *source);
    // Then source kind remains part of the suppression key as well as the deduplication key.
    assert_eq!(
        edges,
        [
            (OTHER_SOURCE, "REFERENCES", EdgeSource::Model),
            (SOURCE, "REVIEWED_REFERENCE", EdgeSource::ReviewedRule),
        ]
    );
    assert_eq!((report.reviewed, report.declared), (1, 5));
    Ok(())
}
