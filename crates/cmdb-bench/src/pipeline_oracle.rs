use super::{
    pipeline::{BenchResult, Phase, Workload},
    pipeline_fixture,
};
use futures_util::TryStreamExt;
use inventory_core::{
    inventory::SqlName,
    record::{Exclusions, ValueRef, scalar_record},
};
use inventory_engine::inventory::{InventoryStore, PgStore, SweepId};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};

async fn connect(url: &str) -> BenchResult<PgPool> {
    Ok(PgPoolOptions::new().max_connections(1).connect(url).await?)
}

pub(super) async fn revision(url: &str) -> BenchResult<i64> {
    let pool = connect(url).await?;
    let revision = revision_of(&pool).await?;
    pool.close().await;
    Ok(revision)
}

async fn revision_of(pool: &PgPool) -> BenchResult<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(pg_sequence_last_value('inventory.projection_revision_seq'),0)::bigint",
    )
    .fetch_one(pool)
    .await?)
}

async fn count(pool: &PgPool, statement: &str) -> BenchResult<u64> {
    let value: i64 = sqlx::query_scalar(AssertSqlSafe(statement))
        .fetch_one(pool)
        .await?;
    Ok(u64::try_from(value)?)
}

struct ProjectionState {
    outbox: u64,
    rebuilds: u64,
    revision_after: i64,
}

async fn validate_projection(
    pool: &PgPool,
    store: &PgStore,
    phase: Phase,
    revision_before: i64,
) -> BenchResult<ProjectionState> {
    let outbox = count(pool, "SELECT COUNT(*) FROM inventory.projection_outbox").await?;
    let state = store.projection_state().await?;
    if outbox != state.pending || outbox > 100_000 {
        return Err("durable outbox count/backpressure mismatch".into());
    }
    let rebuilds = count(
        pool,
        "SELECT COUNT(*) FROM inventory.projection_outbox WHERE full_rebuild",
    )
    .await?;
    if rebuilds != u64::from(state.rebuild_required) {
        return Err("outbox rebuild marker mismatch".into());
    }
    let invalid_outbox = count(
        pool,
        "SELECT COUNT(*) FROM inventory.projection_outbox WHERE deleted OR revision<1",
    )
    .await?;
    if invalid_outbox != 0 {
        return Err("outbox contains unexpected deletion or invalid revision".into());
    }
    let revision_after = revision_of(pool).await?;
    if phase == Phase::Replay && revision_after != revision_before {
        return Err("duplicate replay unexpectedly published new projection revisions".into());
    }
    Ok(ProjectionState {
        outbox,
        rebuilds,
        revision_after,
    })
}

pub(super) async fn check(
    workload: &Workload,
    store: &PgStore,
    sweep: &SweepId,
    phase: Phase,
    revision_before: i64,
) -> BenchResult {
    let pool = connect(workload.database.dsn()).await?;
    let total = u64::try_from(workload.resources()?)?;
    let resources = count(&pool, "SELECT COUNT(*) FROM inventory.resources").await?;
    let scopes = count(&pool, "SELECT COUNT(*) FROM inventory.resource_scopes").await?;
    let tags = count(&pool, "SELECT COUNT(*) FROM inventory.tags").await?;
    let expected_tags = pipeline_fixture::expected_tags(workload.rows_per_scope()?, phase)?
        .checked_mul(workload.concurrency)
        .ok_or("tag count overflow")?;
    if resources != total || scopes != total || tags != u64::try_from(expected_tags)? {
        return Err(format!(
            "{} PostgreSQL count mismatch: resources={resources} scopes={scopes} tags={tags}",
            phase.name()
        )
        .into());
    }
    check_records(&pool, workload).await?;
    if count(
        &pool,
        "SELECT (SELECT COUNT(*) FROM inventory.edges)+(SELECT COUNT(*) FROM inventory.unresolved_refs)+(SELECT COUNT(*) FROM inventory.resource_parents)",
    )
    .await?
        != 0
    {
        return Err("table-only fixture unexpectedly produced dependency edges or parent relationships".into());
    }
    check_tags(&pool, phase).await?;
    check_coverage(&pool, workload, phase).await?;
    check_work(&pool, workload, sweep).await?;
    let proj = validate_projection(&pool, store, phase, revision_before).await?;
    println!(
        "pipeline_state={} resources={resources} resource_scopes={scopes} tags={tags} complete_scopes={} pending_outbox={outbox} rebuild_markers={rebuilds} projection_revision={revision_after} exact_postgres_state=true",
        phase.name(),
        workload.concurrency,
        outbox = proj.outbox,
        rebuilds = proj.rebuilds,
        revision_after = proj.revision_after
    );
    pool.close().await;
    Ok(())
}

const RESOURCE_INDEX: &str =
    "(CASE WHEN name ~ '^bench-[0-9]{12}$' THEN substr(name,7)::bigint ELSE -1 END)";

async fn check_records(pool: &PgPool, workload: &Workload) -> BenchResult {
    let statement = format!(
        "SELECT COUNT(*) FROM inventory.resources WHERE region<>'us-east-1' OR kind<>'AWS::DynamoDB::Table' OR id<>'arn:aws:dynamodb:us-east-1:'||account||':table/'||name OR first_seen<>1000 OR last_changed<>1000 OR octet_length(content_hash)<>32 OR name IS NULL OR name !~ '^bench-[0-9]{{12}}$' OR {RESOURCE_INDEX}<0 OR {RESOURCE_INDEX}>=$1 OR account !~ '^[0-9]{{12}}$' OR (CASE WHEN account ~ '^[0-9]{{12}}$' THEN account::bigint ELSE 0 END) NOT BETWEEN 1 AND $2"
    );
    let invalid: i64 = sqlx::query_scalar(AssertSqlSafe(statement.as_str()))
        .bind(i64::try_from(workload.rows_per_scope()?)?)
        .bind(i64::try_from(workload.concurrency)?)
        .fetch_one(pool)
        .await?;
    if invalid != 0 {
        return Err("resource identity, immutable hash timestamp, or scope is incorrect".into());
    }
    check_detail_records(pool, workload).await?;
    let missing_counters = count(
        pool,
        "SELECT COUNT(*) FROM inventory.resource_scopes WHERE missing_complete<>0",
    )
    .await?;
    if missing_counters != 0 {
        return Err("a fully observed resource has a missing counter".into());
    }
    Ok(())
}

async fn check_detail_records(pool: &PgPool, workload: &Workload) -> BenchResult {
    let table: String = sqlx::query_scalar(
        "SELECT table_name FROM inventory.kinds WHERE kind='AWS::DynamoDB::Table'",
    )
    .fetch_one(pool)
    .await?;
    let table = SqlName::parse(&table)?;
    let detail_count = count(
        pool,
        &format!("SELECT COUNT(*) FROM inventory.\"{}\"", table.as_str()),
    )
    .await?;
    if detail_count != u64::try_from(workload.resources()?)? {
        return Err("typed detail table has missing or extra records".into());
    }
    let statement = format!(
        "SELECT r.name,d.record,r.content_hash,d.content_hash,d.first_seen,d.last_changed FROM inventory.\"{}\" d JOIN inventory.resources r ON r.account=d.account AND r.region=d.region AND r.id=d.id AND r.kind='AWS::DynamoDB::Table'",
        table.as_str()
    );
    let checked = sqlx::query_as::<_, (String, Vec<u8>, Vec<u8>, Vec<u8>, i64, i64)>(
        AssertSqlSafe(statement.as_str()),
    )
    .fetch(pool)
    .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })
    .try_fold(
        0_u64,
        |count, (name, record, summary_hash, detail_hash, first_seen, changed)| async move {
            let expected = scalar_record(ValueRef::Text(&name))?;
            let expected_hash = expected.content_hash(&Exclusions::none())?;
            if record.as_slice() != expected.as_bytes()
                || summary_hash.as_slice() != expected_hash.as_bytes()
                || detail_hash != summary_hash
                || first_seen != 1000
                || changed != 1000
            {
                return Err(
                    "typed record bytes/hash disagree with independently generated fixture".into(),
                );
            }
            count
                .checked_add(1)
                .ok_or_else(|| "detail counter overflow".into())
        },
    )
    .await?;
    if checked != u64::try_from(workload.resources()?)? {
        return Err("typed detail/summary exact-key join count mismatch".into());
    }
    Ok(())
}

async fn check_coverage(pool: &PgPool, workload: &Workload, phase: Phase) -> BenchResult {
    let rows = count(pool, "SELECT COUNT(*) FROM inventory.tag_observations").await?;
    if rows != u64::try_from(workload.resources()?)? {
        return Err("tag coverage row count mismatch".into());
    }
    let invalid: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.tag_observations t JOIN inventory.resources r USING(account,region,kind,id) WHERE t.observed_at<>$1::bigint OR t.coverage<>CASE WHEN $2 AND CAST(substr(r.name,7) AS bigint)%4=2 THEN 'partial' ELSE 'complete' END OR t.last_complete_at IS NULL OR t.last_complete_at<>CASE WHEN $2 AND CAST(substr(r.name,7) AS bigint)%4=2 THEN 1000 ELSE $1::bigint END",
    )
    .bind(phase.timestamp())
    .bind(phase != Phase::Initial)
    .fetch_one(pool)
    .await?;
    if invalid != 0 {
        return Err("durable complete/partial tag coverage mismatch".into());
    }
    Ok(())
}

async fn check_tags(pool: &PgPool, phase: Phase) -> BenchResult {
    let statement = if phase == Phase::Initial {
        "SELECT COUNT(*) FROM inventory.tags t JOIN inventory.resources r USING(account,region,kind,id) WHERE NOT ((t.tag_key='environment' AND t.tag_value='bench') OR (t.tag_key='owner' AND t.tag_value='inventory'))"
    } else {
        "SELECT COUNT(*) FROM inventory.tags t JOIN inventory.resources r USING(account,region,kind,id) WHERE CAST(substr(r.name,7) AS bigint)%4=0 OR NOT ((t.tag_key='owner' AND t.tag_value='inventory') OR (t.tag_key='environment' AND t.tag_value=CASE WHEN CAST(substr(r.name,7) AS bigint)%4=1 THEN 'bench-updated' ELSE 'bench' END))"
    };
    if count(pool, statement).await? != 0 {
        return Err("tag update/empty/partial snapshot exact-state mismatch".into());
    }
    Ok(())
}

async fn check_work(pool: &PgPool, workload: &Workload, sweep: &SweepId) -> BenchResult {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),COUNT(*) FILTER (WHERE state<>2 OR token IS NOT NULL OR lease_until<>0),COUNT(*) FILTER (WHERE generation<>$2) FROM inventory.work_queue WHERE sweep_id=$1",
    )
    .bind(sweep.as_str())
    .bind(i64::try_from(workload.pages)?)
    .fetch_one(pool)
    .await?;
    if row != (i64::try_from(workload.concurrency)?, 0, 0) {
        return Err("work completion or generation checkpoint mismatch".into());
    }
    let tokens: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM inventory.work_tokens WHERE sweep_id=$1")
            .bind(sweep.as_str())
            .fetch_one(pool)
            .await?;
    let expected = workload
        .pages
        .saturating_sub(1)
        .checked_mul(workload.concurrency)
        .ok_or("token count overflow")?;
    if tokens != i64::try_from(expected)? {
        return Err("durable page continuation count mismatch".into());
    }
    let outcomes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.scope_outcomes WHERE sweep_id=$1 AND outcome='complete'",
    )
    .bind(sweep.as_str())
    .fetch_one(pool)
    .await?;
    if outcomes != i64::try_from(workload.concurrency)? {
        return Err("complete scope count mismatch".into());
    }
    let observed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.resource_scopes WHERE observed_sweep=$1",
    )
    .bind(sweep.as_str())
    .fetch_one(pool)
    .await?;
    if observed != i64::try_from(workload.resources()?)? {
        return Err("observed resource checkpoint count mismatch".into());
    }
    Ok(())
}
