use super::*;
use inventory_core::inventory::{Edge, EdgeSource, KindSchema, ReferenceTarget, Relation, UnresolvedReference};

const SOURCE: &str = "arn:aws:example:us-east-1:123456789012:setting/source";
const TARGET: &str = "arn:aws:example:us-east-1:123456789012:setting/target";
const VALUE: &str = "target-value";

fn row(id: &str, value: &str) -> TestResult<Observation> {
    Ok(Observation::new(support::resource(id)?, scalar_record(ValueRef::Text(value))?, &Exclusions::none(), None)?)
}

fn reference(schema: &KindSchema) -> TestResult<UnresolvedReference> {
    let source = support::resource(SOURCE)?;
    Ok(UnresolvedReference {
        target: ReferenceTarget::root_text(source.account.clone(), source.region.clone(), schema, ResourceId::parse(VALUE)?)?,
        from: source,
        relation: Relation::parse("references")?,
        source: EdgeSource::ReviewedRule,
    })
}

async fn relationship_lease(store: &BinaryStore, sweep: &str, at: i64) -> TestResult<WorkLease> {
    store.seed_work(support::relationship_work(sweep, "us-east-1", Some(VALUE), None)?).await?;
    claim(store, sweep, at).await
}

async fn edges(store: &BinaryStore) -> TestResult<Vec<inventory_engine::inventory::ProjectionEdgeHeader>> {
    let mut snapshot = store.begin_projection_snapshot().await?;
    let page = snapshot.edges(None, 1000).await?;
    assert!(page.next.is_none());
    snapshot.finish().await?;
    Ok(page.edges)
}

async fn exact_edge(store: &BinaryStore) -> TestResult {
    let edges = edges(store).await?;
    assert_eq!(edges.len(), 1);
    let edge = edges.first().ok_or("scalar relationship edge absent")?;
    assert_eq!(edge.from, support::resource(SOURCE)?);
    assert_eq!(edge.to, support::resource(TARGET)?);
    assert_eq!(edge.relation, Relation::parse("references")?);
    assert_eq!(edge.source, EdgeSource::ReviewedRule.as_str());
    Ok(())
}

#[tokio::test]
async fn given_text_root_reference_when_arriving_in_either_order_then_reopen_and_target_changes_preserve_exact_evidence() -> TestResult {
    let bytes = support::scalar_catalog_bytes(ScalarType::Text)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let schema = catalog.schemas().first().ok_or("scalar schema absent")?;
    for source_first in [true, false] {
        let (directory, store) = create_store().await?;
        for (index, is_source) in [source_first, !source_first].into_iter().enumerate() {
            let sweep = format!("root-{index}");
            let lease = relationship_lease(&store, &sweep, i64::try_from(index)? + 1).await?;
            let observations = if is_source { vec![row(SOURCE, "source-value")?] } else { vec![row(TARGET, VALUE)?] };
            let mut page = support::page(lease, observations, true, None, i64::try_from(index)? + 1);
            if is_source { page.unresolved.push(reference(schema)?); }
            store.commit_page(page).await?;
            if index == 0 { assert!(edges(&store).await?.is_empty()); }
        }
        exact_edge(&store).await?;
        drop(store);
        let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        exact_edge(&store).await?;
        let lease = relationship_lease(&store, "root-change", 10).await?;
        store.commit_page(support::page(lease, vec![row(TARGET, "different-value")?], false, Some(b"next".to_vec()), 10)).await?;
        assert!(edges(&store).await?.is_empty());
        let pool = read_pool(&directory).await?;
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence").fetch_one(&pool).await?, 1);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resources WHERE id=?1").bind(SOURCE).fetch_one(&pool).await?, 1);
        assert_eq!(sqlx::query_scalar::<_, String>("SELECT resolution FROM unresolved_refs WHERE from_id=?1").bind(SOURCE).fetch_one(&pool).await?, "unresolved");
        let continuation = claim(&store, "root-change", 11).await?;
        let mut page = support::page(continuation, vec![row(SOURCE, "source-value")?, row(TARGET, VALUE)?], true, None, 11);
        page.unresolved.push(reference(schema)?);
        store.commit_page(page).await?;
        exact_edge(&store).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_same_page_text_roots_when_direct_edge_is_checked_then_pin_survives_codec_and_retracts_on_value_change() -> TestResult {
    let (directory, store) = create_store().await?;
    let lease = relationship_lease(&store, "root-direct", 1).await?;
    let mut page = support::page(lease, vec![row(SOURCE, "source-value")?, row(TARGET, VALUE)?], true, None, 1);
    page.edges.push(Edge { from: support::resource(SOURCE)?, to: support::resource(TARGET)?,
        relation: Relation::parse("references")?, source: EdgeSource::ReviewedRule });
    store.commit_page(page).await?;
    exact_edge(&store).await?;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, String>("SELECT target_id FROM reviewed_reference_evidence").fetch_one(&pool).await?, TARGET);
    exact_edge(&store).await?;
    let lease = relationship_lease(&store, "root-direct-change", 2).await?;
    store.commit_page(support::page(lease, vec![row(TARGET, "different-value")?], false, Some(b"next".to_vec()), 2)).await?;
    assert!(edges(&store).await?.is_empty());
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence").fetch_one(&pool).await?, 1);
    Ok(())
}
