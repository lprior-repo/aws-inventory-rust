use super::*;
use crate::declared_fixture::{self, TargetState};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{EdgeSource, UnresolvedReference},
};

type ResultOf<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TARGET_ARN: &str = "arn:aws:refs:us-east-1:123456789012:target";
const SOURCE_ARN: &str = "arn:aws:refs:us-east-1:123456789012:source";

#[test]
fn given_model_rule_when_sweep_extracts_reference_then_model_provenance_is_preserved() -> ResultOf {
    // Given a model rule and a source row carrying its reference value.
    let mut rule = typed_rule()?;
    rule.edge_source = EdgeSource::Model;
    let observation = typed_observation(&rule)?;
    let row = typed_record([(1, Value::Text("target".into()))])?;
    let request = typed_record([])?;
    let home = RegionId::parse("us-east-1")?;
    let mut output = Vec::new();
    // When extracting through the production sweep context.
    super::super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    // Then the reference carries the rule's provenance, not an unconditional reviewed label.
    assert_eq!(output.len(), 1);
    let reference = output.first().ok_or("model reference missing")?;
    assert_eq!(reference.source, EdgeSource::Model);
    assert_eq!(reference.target.value().as_str(), "target");
    Ok(())
}

#[tokio::test]
async fn given_declared_catalog_when_sweep_commits_to_engine_then_late_target_resolves_model_edge()
-> ResultOf {
    // Given a catalog compiled from actual reference traits and a real SQLite store.
    let (bytes, _) = declared_fixture::compile(TargetState::Callable, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("declared.sqlite"), registry).await?;
    let row = typed_record([
        (3, Value::Text("source".into())),
        (4, Value::Text(TARGET_ARN.into())),
    ])?;
    let source = observation(&catalog, "ListSources", row.clone())?;
    let key = source.key().clone();
    let request = typed_record([])?;
    let home = RegionId::parse("us-east-1")?;
    let plan = catalog
        .lookup("refs", "ListSources")
        .ok_or("source plan missing")?;
    let mut references = Vec::new();
    // When the real sweep extractor submits its emitted references to the checked engine.
    super::super::relations::Context {
        row: &row,
        request: &request,
        observation: &source,
        home: &home,
        parent: None,
    }
    .append_plan(&catalog, plan, &mut references, 10)?;
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.source)
            .collect::<Vec<_>>(),
        [EdgeSource::Model]
    );
    commit(&store, "ListSources", vec![source], references, 1000).await?;
    assert_eq!(store.projection_edges_for(&key, None, 10).await?.edges, []);
    let target = observation(
        &catalog,
        "ListTargets",
        typed_record([(3, Value::Text(TARGET_ARN.into()))])?,
    )?;
    let target_key = target.key().clone();
    commit(&store, "ListTargets", vec![target], Vec::new(), 1002).await?;
    // Then the persisted edge resolves exactly, with model provenance intact.
    let edges = store.projection_edges_for(&key, None, 10).await?;
    assert_eq!(edges.edges.len(), 1);
    let edge = edges.edges.first().ok_or("declared edge missing")?;
    assert_eq!(edge.from, key);
    assert_eq!(edge.to, target_key);
    assert_eq!(edge.source, "model");
    assert_eq!(edge.relation.as_str(), "REFERENCES");
    Ok(())
}

#[tokio::test]
async fn given_declared_self_reference_when_two_sweeps_commit_then_exactly_one_self_edge_persists()
-> ResultOf {
    let (bytes, _) = declared_fixture::compile(TargetState::SelfReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("refs", "ListSources")
        .ok_or("source plan missing")?;
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("self.sqlite"), registry).await?;
    for time in [1000, 1002] {
        let row = typed_record([
            (3, Value::Text(SOURCE_ARN.into())),
            (4, Value::Text(SOURCE_ARN.into())),
        ])?;
        let source = observation(&catalog, "ListSources", row.clone())?;
        let key = source.key().clone();
        let references = super::super::references(
            &catalog,
            plan,
            &RegionId::parse("us-east-1")?,
            &[row],
            std::slice::from_ref(&source),
            &typed_record([])?,
            None,
        )?;
        assert_eq!(references.len(), 1);
        commit(&store, "ListSources", vec![source], references, time).await?;
        let actual = store.projection_edges_for(&key, None, 10).await?;
        assert_eq!(actual.edges.len(), 1);
        let edge = actual.edges.first().ok_or("declared self edge absent")?;
        assert_eq!(edge.from, key);
        assert_eq!(edge.to, key);
        assert_eq!(edge.source, "model");
        assert_eq!(edge.relation.as_str(), "REFERENCES");
    }
    Ok(())
}

fn observation(
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    row: Record,
) -> ResultOf<Observation> {
    let plan = catalog
        .lookup("refs", operation)
        .ok_or("observation plan missing")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    Ok(super::super::observation(
        catalog,
        record::IdentityContext {
            account: &account,
            region: &home,
        },
        plan,
        &Region::Regional(home.clone()),
        row,
        &typed_record([])?,
        &inventory_core::inventory::DiscoveryContext::empty(),
    )?)
}

async fn commit(
    store: &BinaryStore,
    operation: &str,
    observations: Vec<Observation>,
    unresolved: Vec<UnresolvedReference>,
    time: i64,
) -> ResultOf {
    let sweep = SweepId::new(format!("declared-reference-{time}"))?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("refs")?,
        OperationId::parse(operation)?,
        None,
    )?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            typed_record([])?.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, time, 1000)
        .await?
        .ok_or("work lease missing")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            unresolved,
            edges: Vec::new(),
            tags: Vec::new(),
            children: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: time,
        })
        .await?;
    Ok(())
}
