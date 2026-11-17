use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{Emission, RelationSource, RuntimeCatalogError},
    inventory::AccountId,
    record::{FieldId, Key, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId};
use serde_json::json;
use std::{collections::{BTreeMap, BTreeSet}, sync::LazyLock};

#[path = "constraint_dispatch_tests/cbor.rs"]
mod cbor;
#[path = "constraint_dispatch_tests/flow.rs"]
mod flow;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SERVICE: &str = "service-catalog";
const OPERATION: &str = "ListConstraintsForPortfolio";
const PORTFOLIO: &str = "port-one";
const PORTFOLIO_ARN: &str = "arn:aws:catalog:us-east-1:123456789012:portfolio/port-one";
const MAPPINGS: &[(&str, &str)] = &[
    ("LAUNCH", "AWS::ServiceCatalog::LaunchRoleConstraint"),
    ("NOTIFICATION", "AWS::ServiceCatalog::LaunchNotificationConstraint"),
    ("RESOURCE_UPDATE", "AWS::ServiceCatalog::ResourceUpdateConstraint"),
    ("STACKSET", "AWS::ServiceCatalog::StackSetConstraint"),
    ("TEMPLATE", "AWS::ServiceCatalog::LaunchTemplateConstraint"),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("production constraint catalog failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    crate::sweep_runtime::pinned_model::install(&root.path().join("models"), SERVICE, "2015-12-10", &["ListPortfolios", OPERATION])?;
    let embedded = overlay::embedded()?;
    let policy = embedded.iter().find(|policy| policy.service() == SERVICE).ok_or("production service absent")?;
    let selected = ["ListPortfolios", OPERATION].into_iter().map(|operation| {
        let key = overlay::OperationName::new(operation)?;
        Ok((operation, policy.operations().get(&key).ok_or("production operation absent")?.clone()))
    }).collect::<TestResult<Vec<_>>>()?;
    let policies = [overlay::ServiceOverlay::new(SERVICE, selected)?];
    let checked = overlay::check_policies(root.path(), &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(root.path(), &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(root.path())?)?)
}

fn field(name: &str) -> TestResult<FieldId> {
    smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?
        .id(name).ok_or_else(|| format!("field {name} absent").into())
}

fn request() -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(field("PortfolioId")?), Value::Text(PORTFOLIO.to_owned())),
    ])))?)
}


fn dispatched(catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>, rows: &[Record]) -> Result<Vec<Observation>, SweepError> {
    let home = RegionId::parse("us-east-1")?;
    observations(catalog, record::IdentityContext { account: &AccountId::parse("123456789012")?, region: &home },
        plan, &Region::Regional(home.clone()), rows, &request().map_err(|_| SweepError::Operation("test request failed"))?,
        &vec![DiscoveryContext::empty(); rows.len()])
}


#[test]
fn given_one_production_constraint_operation_when_compiled_then_all_five_canonical_schemas_and_relations_are_authorized() -> TestResult {
    // Given the actual pinned Smithy model, embedded policies and production kind mappings.
    let catalog = catalog()?;
    let plan = catalog.lookup(SERVICE, OPERATION).ok_or("constraint plan absent")?;
    // When the checked runtime catalog is decoded, there is one producer, not five filtered calls.
    assert_eq!(catalog.plans().len(), 2);
    assert_eq!(plan.emission(), Emission::EmitMetadata);
    assert_eq!(plan.kind().as_str(), MAPPINGS.first().ok_or("mapping absent")?.1);
    assert_eq!(plan.branches().len(), 0);
    let classification = plan.classification().ok_or("classification absent")?;
    assert_eq!(classification.path.as_ref(), &[field("Type")?]);
    assert_eq!(classification.variants.iter().map(|(value, kind)| (value.as_str(), kind.as_str())).collect::<Vec<_>>(), MAPPINGS);
    let parent = catalog.lookup(SERVICE, "ListPortfolios").ok_or("portfolio plan absent")?;
    // Then every selected subtype owns a canonical schema and exactly its checked portfolio relation.
    let mut tables = BTreeSet::new();
    for (_, kind) in MAPPINGS {
        let schema = catalog.schemas().iter().find(|schema| schema.kind().as_str() == *kind).ok_or("canonical subtype schema absent")?;
        assert!(tables.insert(schema.table().as_str()));
        assert!(catalog.coverage().iter().any(|entry| entry.mapped && entry.kind.as_ref() == Some(schema.kind())));
        let relations = catalog.relationships().iter().filter(|relation| relation.source_kind.as_str() == *kind).collect::<Vec<_>>();
        assert_eq!(relations.len(), 1, "{kind}");
        let relation = relations.first().ok_or("portfolio relation absent")?;
        assert_eq!(relation.source, RelationSource::Request);
        assert_eq!(relation.source_path.as_ref(), &[field("PortfolioId")?]);
        assert_eq!(relation.target_kind.as_str(), "AWS::ServiceCatalog::Portfolio");
        assert_eq!(relation.target_path.as_ref(), &[field("Id")?]);
        assert_eq!(relation.relation.as_str(), "FOR_PORTFOLIO");
        assert_eq!(plan.supports_relationship(relation)?, true);
        assert_eq!(parent.supports_relationship(relation)?, false);
    }
    Ok(())
}

#[tokio::test]
async fn given_five_native_constraint_types_in_one_page_when_dispatched_then_each_exact_canonical_row_and_portfolio_edge_persists() -> TestResult {
    // Given a real root lease whose assembled portfolio page discovers one bound constraint child.
    let catalog = catalog()?;
    let plan = catalog.lookup(SERVICE, OPERATION).ok_or("constraint plan absent")?;
    let (root, store, lease) = flow::rooted(&catalog).await?;
    let sweep = lease.sweep().clone();
    let details = MAPPINGS.iter().enumerate().map(|(index, (value, _))| json!({
        "ConstraintId": format!("cons-{index}"), "Type": value, "Description": format!("metadata-{index}"),
    })).collect::<Vec<_>>();
    let body = json!({"ConstraintDetails":details});
    let native = flow::decode(plan, body.clone())?;
    let rows = record::item_records(&native.record, plan.items())?;
    // When the actual claimed child request is encoded and the complete native response is assembled.
    let parts = flow::prepare(&catalog, &lease, body).await?;
    assert_eq!(parts.observations.len(), 5);
    assert_eq!(parts.references.len(), 5);
    assert!(parts.children.is_empty());
    flow::commit(&store, lease, parts).await?;
    let store = flow::reopen(store, &root, &catalog).await?;
    let reader = flow::reader(&root).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 5);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets").fetch_one(&reader).await?, 6);
    // Then reopening retains exact canonical IDs/records and actual resolved portfolio graph edges.
    for (index, ((_, kind), row)) in MAPPINGS.iter().zip(&rows).enumerate() {
        let id = format!("8:port-one|6:cons-{index}");
        let schema = catalog.schemas().iter().find(|schema| schema.kind().as_str() == *kind).ok_or("schema absent")?;
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT id,record FROM \"");
        query.push(schema.table().as_str()).push("\"");
        let saved: Vec<(String, Vec<u8>)> = query.build_query_as().fetch_all(&reader).await?;
        assert_eq!(saved, [(id.clone(), row.as_bytes().to_vec())], "{kind}");
        let key = ResourceKey { account: AccountId::parse("123456789012")?, region: Region::parse("us-east-1")?,
            kind: inventory_core::inventory::Kind::parse(kind)?, id: ResourceId::parse(&id)? };
        let edges = store.projection_edges_for(&key, None, 10).await?;
        assert_eq!(edges.edges.len(), 1, "{kind}");
        let edge = edges.edges.first().ok_or("portfolio edge absent")?;
        assert_eq!(edge.from, key);
        assert_eq!(edge.relation.as_str(), "FOR_PORTFOLIO");
        assert_eq!(edge.to.kind.as_str(), "AWS::ServiceCatalog::Portfolio");
        assert_eq!(edge.to.id.as_str(), PORTFOLIO_ARN);
        assert_eq!(edge.to.account, AccountId::parse("123456789012")?);
        assert_eq!(edge.to.region, Region::parse("us-east-1")?);
    }
    let summary = store.sweep_summary(&sweep).await?;
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.work_pending, 0);
    reader.close().await;
    Ok(())
}

#[tokio::test]
async fn given_unknown_constraint_after_known_row_when_dispatched_then_incomplete_coverage_is_explicit_and_no_generic_resource_persists() -> TestResult {
    // Given a persisted portfolio and its real discovered child lease, not a fabricated parentless scope.
    let catalog = catalog()?;
    let (root, store, lease) = flow::rooted(&catalog).await?;
    // When native request encoding and complete production assembly encounter an unknown second row.
    let error = flow::prepare(&catalog, &lease, json!({"ConstraintDetails":[
        {"ConstraintId":"known","Type":"LAUNCH"}, {"ConstraintId":"future","Type":"FUTURE_CONSTRAINT"}
    ]})).await.err().ok_or("unknown discriminator unexpectedly assembled")?;
    assert_eq!(error.to_string(), "runtime catalog error: incomplete resource kind coverage: unknown discriminator \"FUTURE_CONSTRAINT\"");
    assert!(matches!(error.downcast_ref::<SweepError>(),
        Some(SweepError::Catalog(RuntimeCatalogError::UnclassifiedKind { discriminator })) if discriminator == "FUTURE_CONSTRAINT"));
    // Then reopening shows only the committed root; even the preceding known child row is absent.
    let _store = flow::reopen(store, &root, &catalog).await?;
    let reader = flow::reader(&root).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets").fetch_one(&reader).await?, 1);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 0);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs").fetch_one(&reader).await?, 0);
    reader.close().await;
    Ok(())
}

#[test]
fn given_missing_null_or_nontext_typed_discriminator_when_dispatched_then_malformed_input_is_not_unknown_coverage() -> TestResult {
    // Given already typed rows, including forms the native JSON decoder may reject earlier.
    let catalog = catalog()?;
    let plan = catalog.lookup(SERVICE, OPERATION).ok_or("constraint plan absent")?;
    for value in [None, Some(Value::Null), Some(Value::Bool(true)), Some(Value::Int(1))] {
        let mut fields = BTreeMap::from([(Key::Field(field("ConstraintId")?), Value::Text("cons-one".to_owned()))]);
        if let Some(value) = value { fields.insert(Key::Field(field("Type")?), value); }
        let row = Record::encode(&Value::Map(fields))?;
        // When classification sees invalid modeled discriminator data, it fails distinctly.
        let result = dispatched(&catalog, plan, &[row]);
        // Then it never falls back to LaunchRoleConstraint or calls five APIs.
        assert!(matches!(result, Err(SweepError::Catalog(RuntimeCatalogError::Capability))));
    }
    Ok(())
}

#[tokio::test]
async fn given_each_classified_kind_with_wrong_request_evidence_when_committed_then_checked_authorization_rejects_atomically() -> TestResult {
    let catalog = catalog()?;
    for (value, _) in MAPPINGS {
        // Given an actually discovered child lease and references from the complete production assembly.
        let (root, store, lease) = flow::rooted(&catalog).await?;
        let mut parts = flow::prepare(&catalog, &lease,
            json!({"ConstraintDetails":[{"ConstraintId":"cons-one","Type":value}]})).await?;
        assert_eq!(parts.references.len(), 1, "{value}");
        let reference = parts.references.first_mut().ok_or("assembled portfolio reference absent")?;
        assert_eq!(reference.target.value().as_str(), PORTFOLIO);
        // When a caller forges relationship evidence inconsistent with the unchanged real lease request.
        reference.target = inventory_core::inventory::ReferenceTarget::new(
            AccountId::parse("123456789012")?, Region::parse("us-east-1")?,
            inventory_core::inventory::Kind::parse("AWS::ServiceCatalog::Portfolio")?,
            vec![field("Id")?], ResourceId::parse("port-other")?)?;
        let error = flow::commit(&store, lease, parts).await.err().ok_or("forged request evidence committed")?;
        // Then the checked store rejects atomically, preserving only the previously committed root.
        assert!(matches!(error.downcast_ref::<inventory_engine::inventory::StoreError>(),
            Some(inventory_engine::inventory::StoreError::Invalid(
                "relationship evidence is not permitted by the checked catalog"))), "{value}");
        let _store = flow::reopen(store, &root, &catalog).await?;
        let reader = flow::reader(&root).await?;
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets").fetch_one(&reader).await?, 1);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 0);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs").fetch_one(&reader).await?, 0);
        reader.close().await;
    }
    Ok(())
}
