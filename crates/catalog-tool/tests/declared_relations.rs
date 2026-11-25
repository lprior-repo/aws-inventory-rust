#[path = "declared_relations/boundaries.rs"]
mod boundaries;
#[path = "support/declared.rs"]
mod fixture;
#[path = "declared_relations/review.rs"]
mod review;

use catalog_tool::kind_map;
use fixture::{SOURCE, TARGET, TargetState, TestResult, compile};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::EdgeSource, record::FieldId};

#[test]
fn given_each_reference_trait_when_compiled_then_paths_and_model_provenance_roundtrip() -> TestResult
{
    let (bytes, report) = compile(TargetState::Callable, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let paths = catalog
        .relationships()
        .iter()
        .map(|relation| {
            assert_eq!(relation.source_kind.as_str(), SOURCE);
            assert_eq!(relation.target_kind.as_str(), TARGET);
            assert_eq!(relation.target_path.as_ref(), &[FieldId::new(3)?]);
            assert_eq!(relation.relation.as_str(), "REFERENCES");
            assert_eq!(relation.edge_source, EdgeSource::Model);
            Ok(relation.source_path.to_vec())
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        paths,
        [
            vec![FieldId::new(4)?],
            vec![FieldId::new(1)?],
            vec![FieldId::new(5)?, FieldId::new(3)?]
        ]
    );
    assert_eq!(
        (report.reviewed, report.declared, report.declared_thin),
        (0, 3, 0)
    );
    assert_eq!(
        report.inactive.into_iter().collect::<Vec<_>>(),
        [
            (
                format!(
                    "pending {SOURCE} ExternalRef external#Resource - REFERENCES (target kind not callable)"
                ),
                "target kind not callable".to_owned()
            ),
            (
                format!(
                    "pending {SOURCE} PendingRef n#Pending - REFERENCES (target kind not callable)"
                ),
                "target kind not callable".to_owned()
            ),
        ]
    );
    Ok(())
}

#[test]
fn given_noncallable_reference_target_when_compiled_then_declarations_remain_pending() -> TestResult
{
    let (bytes, report) = compile(TargetState::Deferred, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships(), &[]);
    assert_eq!(report.declared, 0);
    assert_eq!(
        report.inactive.into_iter().collect::<Vec<_>>(),
        [
            (
                format!("pending {SOURCE} ArnRef {TARGET} - REFERENCES (target kind not callable)"),
                "target kind not callable".to_owned()
            ),
            (
                format!(
                    "pending {SOURCE} ExternalRef external#Resource - REFERENCES (target kind not callable)"
                ),
                "target kind not callable".to_owned()
            ),
            (
                format!("pending {SOURCE} IdRef {TARGET} - REFERENCES (target kind not callable)"),
                "target kind not callable".to_owned()
            ),
            (
                format!(
                    "pending {SOURCE} Nested.Id {TARGET} - REFERENCES (target kind not callable)"
                ),
                "target kind not callable".to_owned()
            ),
            (
                format!(
                    "pending {SOURCE} PendingRef n#Pending - REFERENCES (target kind not callable)"
                ),
                "target kind not callable".to_owned()
            ),
        ]
    );
    Ok(())
}

#[test]
fn given_reviewed_rule_for_same_path_and_kind_when_compiled_then_rule_wins_once() -> TestResult {
    let rule = kind_map::RelationMapping {
        source_kind: SOURCE,
        source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields("ArnRef")),
        target_kind: TARGET,
        target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("Id")),
        relation: "REVIEWED_REFERENCE",
        scope: "account_region",
        guard: None,
    };
    let (bytes, report) = compile(TargetState::Callable, &[rule])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let source_path = [FieldId::new(1)?];
    let relations = catalog
        .relationships()
        .iter()
        .filter(|relation| relation.source_path.as_ref() == source_path)
        .collect::<Vec<_>>();
    assert_eq!(relations.len(), 1);
    let relation = relations.first().ok_or("reviewed relation missing")?;
    assert_eq!(relation.relation.as_str(), "REVIEWED_REFERENCE");
    assert_eq!(relation.edge_source, EdgeSource::ReviewedRule);
    assert_eq!((report.reviewed, report.declared), (1, 2));
    Ok(())
}

#[test]
fn given_trait_payloads_when_base_catalog_is_read_then_payloads_and_resource_operations_are_retained()
-> TestResult {
    let (bytes, _) = compile(TargetState::Callable, &[])?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let service = runtime
        .catalog()
        .services()?
        .next()
        .ok_or("service missing")??;
    let shapes = service
        .shapes()?
        .map(|shape| {
            let shape = shape?;
            Ok((shape.name()?, shape))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, inventory_core::catalog::CatalogError>>(
        )?;
    let source = shapes.get("Source").ok_or("source shape missing")?;
    let references: serde_json::Value =
        serde_json::from_str(source.references()?.ok_or("references missing")?)?;
    assert_eq!(
        references,
        serde_json::json!([{"resource":"n#Resource","ids":{"Id":"IdRef"}}])
    );
    let members = source
        .members()?
        .map(|member| {
            let member = member?;
            Ok((member.name()?, member))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, inventory_core::catalog::CatalogError>>(
        )?;
    let arn = members.get("ArnRef").ok_or("ARN member missing")?;
    let payload: serde_json::Value =
        serde_json::from_str(arn.arn_reference()?.ok_or("ARN reference missing")?)?;
    assert_eq!(payload, serde_json::json!({"resource":"n#Resource"}));
    let resource = shapes.get("Resource").ok_or("resource shape missing")?;
    assert_eq!(
        resource
            .resource_operations()?
            .collect::<Result<Vec<_>, _>>()?,
        ["ListTargets", "ListTargets"]
    );
    assert_eq!(
        resource
            .resource_identifiers()?
            .collect::<Result<Vec<_>, _>>()?,
        ["Id"]
    );
    Ok(())
}

#[test]
fn given_compound_identifiers_when_compiled_then_members_remain_one_joint_reference() -> TestResult
{
    let (bytes, report) = compile(TargetState::Compound, &[])?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let paths = runtime
        .relationships()
        .iter()
        .map(|relation| {
            assert_eq!(relation.edge_source, EdgeSource::Model);
            assert_eq!(relation.components.len(), 1);
            let component = relation
                .components
                .first()
                .ok_or("joint component missing")?;
            Ok((
                relation.source_path.to_vec(),
                relation.target_path.to_vec(),
                component.source_path.to_vec(),
                component.target_path.to_vec(),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        paths,
        [
            (
                vec![FieldId::new(4)?],
                vec![FieldId::new(3)?],
                vec![FieldId::new(3)?],
                vec![FieldId::new(4)?]
            ),
            (
                vec![FieldId::new(5)?, FieldId::new(3)?],
                vec![FieldId::new(3)?],
                vec![FieldId::new(5)?, FieldId::new(4)?],
                vec![FieldId::new(4)?]
            ),
        ]
    );
    assert_eq!((report.reviewed, report.declared), (0, 2));
    assert_eq!(
        report
            .inactive
            .get(&format!("pending {SOURCE} ArnRef {TARGET} - REFERENCES (ARN reference requires an ARN identity)"))
            .map(String::as_str),
        Some("ARN reference requires an ARN identity")
    );
    Ok(())
}

fn reviewed_rule(path: &str) -> kind_map::RelationMapping<'_> {
    kind_map::RelationMapping {
        source_kind: SOURCE,
        source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields(path)),
        target_kind: TARGET,
        target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("Id")),
        relation: "REVIEWED_REFERENCE",
        scope: "account_region",
        guard: None,
    }
}

#[test]
fn given_inactive_reviewed_target_group_when_compiled_then_declaration_remains_live() -> TestResult
{
    let (bytes, report) = compile(TargetState::InactiveReview, &[reviewed_rule("ArnRef")])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(1)?];
    let sources = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .map(|rule| rule.edge_source)
        .collect::<Vec<_>>();
    assert_eq!(sources, [EdgeSource::Model]);
    assert_eq!((report.reviewed, report.declared), (0, 3));
    assert_eq!(
        report
            .inactive
            .get(&format!("{SOURCE} -> {TARGET} (inactive source kind)"))
            .map(String::as_str),
        Some("target kind AWS::ApiOnly::MissingReferenceTarget is inactive")
    );
    Ok(())
}

#[test]
fn given_reviewed_secondary_binding_when_compiled_then_joint_primary_declaration_is_preserved()
-> TestResult {
    let (bytes, report) = compile(TargetState::Compound, &[reviewed_rule("Id")])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(4)?];
    let rule = catalog
        .relationships()
        .iter()
        .find(|rule| rule.source_path.as_ref() == path)
        .ok_or("joint primary declaration suppressed")?;
    assert_eq!(rule.edge_source, EdgeSource::Model);
    assert_eq!(rule.components.len(), 1);
    assert_eq!((report.reviewed, report.declared), (1, 2));
    Ok(())
}

#[test]
fn given_container_references_when_compiled_then_each_path_has_an_explicit_pending_reason()
-> TestResult {
    let (bytes, report) = compile(TargetState::Collections, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships().len(), 3);
    assert_eq!((report.declared, report.inactive.len()), (3, 4));
    let pending = report
        .inactive
        .iter()
        .filter(|(_, reason)| {
            reason.as_str() == "reference path traverses a container (requires E2.2 extraction)"
        })
        .map(|(path, reason)| (path.as_str(), reason.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let reason = "reference path traverses a container (requires E2.2 extraction)";
    let list = format!("pending {SOURCE} Collection.Id n#Resource - REFERENCES ({reason})");
    let map = format!("pending {SOURCE} Mapped.Id n#Resource - REFERENCES ({reason})");
    assert_eq!(
        pending,
        std::collections::BTreeMap::from([(list.as_str(), reason), (map.as_str(), reason),])
    );
    Ok(())
}

#[test]
fn given_missing_implicit_identifier_when_compiled_then_invalid_reference_is_pending() -> TestResult
{
    let (bytes, report) = compile(TargetState::MissingMember, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships().len(), 2);
    assert_eq!(report.declared, 2);
    assert_eq!(
        report
            .inactive
            .get(&format!(
                "pending {SOURCE} Nested.Id n#Resource - REFERENCES (reference identifier member is absent)"
            ))
            .map(String::as_str),
        Some("reference identifier member is absent")
    );
    Ok(())
}

#[test]
fn given_ambiguous_nested_identity_names_when_compiled_then_no_identifier_is_reused() -> TestResult
{
    let (bytes, report) = compile(TargetState::RepeatedIdentity, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.relationships(), &[]);
    assert_eq!(report.declared, 0);
    assert_eq!(
        report
            .inactive
            .get(&format!("pending {SOURCE} IdRef {TARGET} - REFERENCES (target identity members cannot be bound)"))
            .map(String::as_str),
        Some("target identity members cannot be bound")
    );
    Ok(())
}

#[test]
fn given_shared_primary_with_distinct_joint_bindings_when_compiled_then_neither_chain_is_lost()
-> TestResult {
    let (bytes, _) = compile(TargetState::DistinctBindings, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = [FieldId::new(4)?];
    let mut paths = catalog
        .relationships()
        .iter()
        .filter(|rule| rule.source_path.as_ref() == path)
        .flat_map(|rule| {
            rule.components
                .iter()
                .map(|component| component.source_path.to_vec())
        })
        .collect::<Vec<_>>();
    paths.sort_unstable();
    assert_eq!(paths, [vec![FieldId::new(1)?], vec![FieldId::new(3)?]]);
    Ok(())
}

#[test]
fn given_thin_service_reference_when_compiled_then_report_contains_nonzero_live_count() -> TestResult
{
    let (bytes, report) = compile(TargetState::Thin, &[])?;
    let text = report.report()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let service = catalog
        .catalog()
        .services()?
        .next()
        .ok_or("thin service missing")??;
    assert_eq!(service.name()?, fixture::THIN_SERVICE);
    assert_eq!(catalog.relationships().len(), 1);
    assert_eq!((report.declared, report.declared_thin), (1, 1));
    assert_eq!(
        text.lines().take(2).collect::<Vec<_>>(),
        [
            "relations live=1 pending=2 declared=1 reviewed=0",
            "declared thin=1",
        ]
    );
    Ok(())
}

#[test]
fn given_same_kind_declaration_when_compiled_then_its_identity_selector_remains_available()
-> TestResult {
    let (bytes, report) = compile(TargetState::SelfReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let relations = catalog
        .relationships()
        .iter()
        .map(|rule| {
            (
                rule.source_kind.as_str(),
                rule.target_kind.as_str(),
                rule.target_path.as_ref(),
                rule.edge_source,
            )
        })
        .collect::<Vec<_>>();
    let path = [FieldId::new(3)?];
    assert_eq!(
        relations,
        [(SOURCE, SOURCE, path.as_slice(), EdgeSource::Model); 3]
    );
    assert_eq!(report.declared, 3);
    Ok(())
}

#[test]
fn given_identity_and_separate_reference_members_when_compiled_then_both_paths_are_retained()
-> TestResult {
    let (bytes, _) = compile(TargetState::IdentityAndSeparateReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let identity = [FieldId::new(3)?];
    let separate = [FieldId::new(4)?];
    for source in [identity.as_slice(), separate.as_slice()] {
        assert!(catalog.relationships().iter().any(|rule| {
            rule.source_kind.as_str() == SOURCE
                && rule.target_kind.as_str() == SOURCE
                && rule.target_path.as_ref() == identity
                && rule.source_path.as_ref() == source
                && rule.edge_source == EdgeSource::Model
        }));
    }
    Ok(())
}

#[test]
fn given_seventeen_shared_shapes_without_references_when_compiled_then_existing_edges_are_unchanged()
-> TestResult {
    let (bytes, report) = compile(TargetState::Branching, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let paths = catalog
        .relationships()
        .iter()
        .map(|rule| rule.source_path.to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            vec![FieldId::new(4)?],
            vec![FieldId::new(1)?],
            vec![FieldId::new(5)?, FieldId::new(3)?],
        ]
    );
    assert_eq!((report.declared, report.inactive.len()), (3, 2));
    Ok(())
}
