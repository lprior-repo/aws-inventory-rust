use super::super::{TestResult, collect, multi_kind_registry, run_with, support};
use super::status;
use inventory_core::inventory::{
    AccountId, EdgeSource, Kind, Observation, ReferenceTarget, Region, Relation, ResourceId,
    ResourceKey, UnresolvedReference,
};
use inventory_core::record::{Exclusions, FieldId, Key, Record, Value};
use inventory_engine::inventory::{DatabaseUrl, PgStore};
use sqlx::PgPool;
use std::collections::BTreeMap;

const THING: &str = "AWS::Example::Thing";
const OTHER: &str = "AWS::Example::Other";

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_only_second_kind_matches_when_reference_spans_kinds_then_edge_reaches_second_kind()
-> TestResult {
    run_with(multi_kind_registry()?, only_second_kind_matches).await
}

async fn only_second_kind_matches(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    commit(
        store,
        "only-second",
        vec![kinded(OTHER, "other-id", "target-name")?],
    )
    .await?;
    assert_eq!(
        edge_endpoints(pool).await?,
        vec![(OTHER.to_owned(), "other-id".to_owned())]
    );
    assert_eq!(status(pool).await?, "resolved");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_first_kind_matches_and_second_kind_decoys_when_reference_spans_kinds_then_only_first_kind_edge_survives()
-> TestResult {
    run_with(multi_kind_registry()?, partial_match_with_decoy).await
}

async fn partial_match_with_decoy(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    commit(
        store,
        "partial",
        vec![
            kinded(THING, "thing-id", "target-name")?,
            kinded(OTHER, "other-decoy", "other-name")?,
        ],
    )
    .await?;
    assert_eq!(
        edge_endpoints(pool).await?,
        vec![(THING.to_owned(), "thing-id".to_owned())]
    );
    assert_eq!(status(pool).await?, "resolved");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_both_kinds_match_when_reference_spans_kinds_then_reference_is_ambiguous_and_unlinked()
-> TestResult {
    run_with(multi_kind_registry()?, both_kinds_match).await
}

async fn both_kinds_match(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    commit(
        store,
        "both",
        vec![
            kinded(THING, "thing-id", "target-name")?,
            kinded(OTHER, "other-id", "target-name")?,
        ],
    )
    .await?;
    assert!(edge_endpoints(pool).await?.is_empty());
    assert_eq!(status(pool).await?, "ambiguous");
    Ok(())
}

async fn commit(store: &PgStore, sweep: &str, targets: Vec<Observation>) -> TestResult {
    let mut observations = vec![support::observation("source", "source-name")?];
    observations.extend(targets);
    collect(store, sweep, observations, vec![multi_kind_reference()?]).await?;
    Ok(())
}

fn kinded(kind: &str, id: &str, value: &str) -> TestResult<Observation> {
    let resource = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse(kind)?,
        id: ResourceId::parse(id)?,
    };
    let fields = BTreeMap::from([(Key::Field(FieldId::new(1)?), Value::Text(value.to_owned()))]);
    Ok(Observation::new(
        resource,
        Record::encode(&Value::Map(fields))?,
        &Exclusions::none(),
        None,
    )?)
}

fn multi_kind_reference() -> TestResult<UnresolvedReference> {
    let from = support::resource("source")?;
    Ok(UnresolvedReference {
        target: ReferenceTarget::new(
            from.account.clone(),
            from.region.clone(),
            from.kind.clone(),
            vec![FieldId::new(1)?],
            ResourceId::parse("target-name")?,
        )?
        .with_candidate_kinds(vec![Kind::parse(THING)?, Kind::parse(OTHER)?])?,
        from,
        relation: Relation::parse("references")?,
        source: EdgeSource::ReviewedRule,
    })
}

async fn edge_endpoints(pool: &PgPool) -> TestResult<Vec<(String, String)>> {
    Ok(sqlx::query_as(
        "SELECT to_kind, to_id FROM inventory.edges WHERE from_id='source' ORDER BY to_kind,to_id",
    )
    .fetch_all(pool)
    .await?)
}
