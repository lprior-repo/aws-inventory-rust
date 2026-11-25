use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{Emission, RelationSource, RuntimeCatalogError},
    inventory::AccountId,
    record::{FieldId, Key, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

#[path = "constraint_dispatch_tests/cbor.rs"]
mod cbor;
#[path = "constraint_dispatch_tests/flow.rs"]
mod flow;
#[path = "constraint_dispatch_tests/native.rs"]
mod native;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SERVICE: &str = "service-catalog";
const OPERATION: &str = "ListConstraintsForPortfolio";
const PORTFOLIO: &str = "port-one";
const PORTFOLIO_ARN: &str = "arn:aws:catalog:us-east-1:123456789012:portfolio/port-one";
const MAPPINGS: &[(&str, &str)] = &[
    ("LAUNCH", "AWS::ServiceCatalog::LaunchRoleConstraint"),
    (
        "NOTIFICATION",
        "AWS::ServiceCatalog::LaunchNotificationConstraint",
    ),
    (
        "RESOURCE_UPDATE",
        "AWS::ServiceCatalog::ResourceUpdateConstraint",
    ),
    ("STACKSET", "AWS::ServiceCatalog::StackSetConstraint"),
    ("TEMPLATE", "AWS::ServiceCatalog::LaunchTemplateConstraint"),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("production constraint catalog failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    crate::pinned_model::install(
        &root.path().join("models"),
        SERVICE,
        "2015-12-10",
        &["ListPortfolios", OPERATION],
    )?;
    let embedded = overlay::embedded()?;
    let policy = embedded
        .iter()
        .find(|policy| policy.service() == SERVICE)
        .ok_or("production service absent")?;
    let selected = ["ListPortfolios", OPERATION]
        .into_iter()
        .map(|operation| {
            let key = overlay::OperationName::new(operation)?;
            Ok((
                operation,
                policy
                    .operations()
                    .get(&key)
                    .ok_or("production operation absent")?
                    .clone(),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let policies = [overlay::ServiceOverlay::new(SERVICE, selected)?];
    let checked = overlay::check_policies(
        root.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(
        root.path(),
        &fields,
        "46135f8468cd5cb60d983b97290c9f6661e359c6",
    )?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &endpoints::verified_endpoints(root.path())?,
    )?)
}

fn field(name: &str) -> TestResult<FieldId> {
    smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?
    .id(name)
    .ok_or_else(|| format!("field {name} absent").into())
}

fn request() -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(field("PortfolioId")?),
        Value::Text(PORTFOLIO.to_owned()),
    )])))?)
}

fn dispatched(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    rows: &[Record],
) -> Result<Vec<Observation>, SweepError> {
    let home = RegionId::parse("us-east-1")?;
    let identities = identities(
        catalog,
        record::IdentityContext {
            account: &AccountId::parse("123456789012")?,
            region: &home,
        },
        plan,
        &Region::Regional(home.clone()),
        rows,
        &request().map_err(|_| SweepError::Operation("test request failed"))?,
        &vec![DiscoveryContext::empty(); rows.len()],
    )?;
    observations(
        rows.to_vec(),
        identities,
        plan.hash_exclusions(),
        plan.facet_precedence(),
    )
}

#[test]
fn given_one_production_constraint_operation_when_compiled_then_all_five_canonical_schemas_and_relations_are_authorized()
-> TestResult {
    let catalog = catalog()?;
    let plan = catalog
        .lookup(SERVICE, OPERATION)
        .ok_or("constraint plan absent")?;
    assert_eq!(catalog.plans().len(), 2);
    assert_eq!(plan.emission(), Emission::EmitMetadata);
    assert_eq!(
        plan.kind().as_str(),
        MAPPINGS.first().ok_or("mapping absent")?.1
    );
    assert_eq!(plan.branches().len(), 0);
    let classification = plan.classification().ok_or("classification absent")?;
    assert_eq!(classification.path.as_ref(), &[field("Type")?]);
    assert_eq!(
        classification
            .variants
            .iter()
            .map(|(value, kind)| (value.as_str(), kind.as_str()))
            .collect::<Vec<_>>(),
        MAPPINGS
    );
    let parent = catalog
        .lookup(SERVICE, "ListPortfolios")
        .ok_or("portfolio plan absent")?;
    let mut tables = BTreeSet::new();
    for (_, kind) in MAPPINGS {
        let schema = catalog
            .schemas()
            .iter()
            .find(|schema| schema.kind().as_str() == *kind)
            .ok_or("canonical subtype schema absent")?;
        assert!(tables.insert(schema.table().as_str()));
        assert!(
            catalog
                .coverage()
                .iter()
                .any(|entry| entry.mapped && entry.kind.as_ref() == Some(schema.kind()))
        );
        let relations = catalog
            .relationships()
            .iter()
            .filter(|relation| relation.source_kind.as_str() == *kind)
            .collect::<Vec<_>>();
        assert_eq!(relations.len(), 1, "{kind}");
        let relation = relations.first().ok_or("portfolio relation absent")?;
        assert_eq!(relation.source, RelationSource::Request);
        assert_eq!(relation.source_path.as_ref(), &[field("PortfolioId")?]);
        assert_eq!(
            relation.target_kind.as_str(),
            "AWS::ServiceCatalog::Portfolio"
        );
        assert_eq!(relation.target_path.as_ref(), &[field("Id")?]);
        assert_eq!(relation.relation.as_str(), "FOR_PORTFOLIO");
        assert!(plan.supports_relationship(relation)?);
        assert!(!parent.supports_relationship(relation)?);
    }
    Ok(())
}
