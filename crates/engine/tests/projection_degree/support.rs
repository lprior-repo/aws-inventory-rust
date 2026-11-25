use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::inventory::{Relation, ResourceId};
use inventory_graph::{EdgeRow, NodeKey, Provenance};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(super) async fn seed_edges(
    database: &Path,
    owner: &NodeKey,
    count: usize,
) -> TestResult<Vec<EdgeRow>> {
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(database),
    )
    .await?;
    let prefix = format!(
        "degree-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let incident: i64 = sqlx::query_scalar("SELECT count(*) FROM edges WHERE (from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4) OR (to_account=?1 AND to_region=?2 AND to_kind=?3 AND to_id=?4)")
        .bind(owner.account.as_str()).bind(owner.region.as_str()).bind(owner.kind.as_str())
        .bind(owner.id.as_str()).fetch_one(&pool).await?;
    assert_eq!(incident, 0);
    let transaction = pool.begin().await?;
    let result = stream::iter(0..count).map(Ok::<_, Box<dyn std::error::Error>>)
        .try_fold((transaction, Vec::new()), |(mut transaction, mut edges), index| {
            let prefix = &prefix;
            async move {
            let target_id = format!("{prefix}-{index:05}");
            sqlx::query("INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) SELECT account,region,kind,?1,NULL,content_hash,first_seen,last_changed FROM resources WHERE account=?2 AND region=?3 AND kind=?4 AND id=?5")
                .bind(&target_id).bind(owner.account.as_str()).bind(owner.region.as_str())
                .bind(owner.kind.as_str()).bind(owner.id.as_str())
                .execute(&mut *transaction).await?;
            sqlx::query("INSERT INTO edges(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source) VALUES(?1,?2,?3,?4,?1,?2,?3,?5,'attached','fixture')")
                .bind(owner.account.as_str()).bind(owner.region.as_str())
                .bind(owner.kind.as_str()).bind(owner.id.as_str()).bind(&target_id)
                .execute(&mut *transaction).await?;
            let mut target = owner.clone();
            target.id = ResourceId::parse(&target_id)?;
            edges.push(EdgeRow {
                from: owner.clone(), to: target,
                relation: Relation::parse("attached")?, provenance: Provenance::parse("fixture")?,
            });
            Ok((transaction, edges))
        }}).await;
    let (transaction, edges) = result?;
    transaction.commit().await?;
    pool.close().await;
    Ok(edges)
}

pub(super) fn sort_edges(edges: &mut [EdgeRow]) {
    edges.sort_by(|left, right| {
        (&left.from, &left.to, &left.relation, &left.provenance).cmp(&(
            &right.from,
            &right.to,
            &right.relation,
            &right.provenance,
        ))
    });
}
