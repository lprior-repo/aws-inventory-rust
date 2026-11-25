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
    let mut rule = typed_rule()?;
    rule.edge_source = EdgeSource::Model;
    let observation = typed_observation(&rule)?;
    let row = typed_record([(1, Value::Text("target".into()))])?;
    let request = typed_record([])?;
    let home = RegionId::parse("us-east-1")?;
    let mut output = Vec::new();
    super::super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(output.len(), 1);
    let reference = output.first().ok_or("model reference missing")?;
    assert_eq!(reference.source, EdgeSource::Model);
    assert_eq!(reference.target.value().as_str(), "target");
    Ok(())
}

#[tokio::test]
async fn given_declared_catalog_when_sweep_commits_to_engine_then_late_target_resolves_model_edge()
-> ResultOf {
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

#[tokio::test]
async fn given_identity_and_separate_reference_fields_when_swept_then_only_the_separate_reference_is_an_edge()
-> ResultOf {
    let (bytes, _) = declared_fixture::compile(TargetState::IdentityAndSeparateReference, &[])?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("refs", "ListSources")
        .ok_or("source plan missing")?;
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("identity.sqlite"), registry).await?;
    let request = typed_record([])?;
    let region = RegionId::parse("us-east-1")?;
    for (at, separate_reference) in [(1000, false), (1002, true)] {
        let row = typed_record(
            [(3, Value::Text(SOURCE_ARN.into()))]
                .into_iter()
                .chain(separate_reference.then(|| (4, Value::Text(SOURCE_ARN.into())))),
        )?;
        let source = observation(&catalog, "ListSources", row.clone())?;
        let key = source.key().clone();
        let references = super::super::references(
            &catalog,
            plan,
            &region,
            &[row],
            std::slice::from_ref(&source),
            &request,
            None,
        )?;
        assert_eq!(references.len(), usize::from(separate_reference));
        commit(&store, "ListSources", vec![source], references, at).await?;
        let actual = store.projection_edges_for(&key, None, 10).await?;
        if separate_reference {
            let [edge] = actual.edges.as_slice() else {
                return Err("expected exactly one separate-reference edge".into());
            };
            assert_eq!(edge.from, key);
            assert_eq!(edge.to, key);
            assert_eq!(edge.source, "model");
            assert_eq!(edge.relation.as_str(), "REFERENCES");
        } else {
            assert!(actual.edges.is_empty());
        }
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
