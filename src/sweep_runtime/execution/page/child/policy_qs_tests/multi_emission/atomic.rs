use super::*;
use crate::sweep_runtime::{ScopeOutcomes, SweepConfig, SweepSummary, execution::{self, BootstrapContext, RuntimeContext, page}};
use aws_wire::transport::{Client, Config};
use inventory_aws::model_credentials::ProfileCredentials;
use sqlx::Row;

pub(super) struct Driver {
    client: Client,
    credentials: ProfileCredentials,
    config: SweepConfig,
}

impl Driver {
    pub(super) async fn new(home: &RegionId) -> TestResult<Self> {
        // This lazy provider is never resolved: the fixture enters after native wire decoding.
        let credentials = ProfileCredentials::load(Some("inventory-post-response-fixture"), Some(home.as_str())).await?;
        Ok(Self { client: Client::new(Config::default())?, credentials,
            config: SweepConfig { region: Some(home.as_str().to_owned()), max_pages: 10,
                concurrency: 1, resume: false, endpoint: None, ca_bundle: None } })
    }

    pub(super) async fn commit(&self, scenario: &Scenario<'_, '_>, catalog: &RuntimeCatalog<'_>,
        lease: WorkLease, status: &str, keys: &str,
    ) -> TestResult<SweepSummary> {
        assert_eq!(lease.scope().operation().as_str(), "GetDNSSEC");
        let plan = fixture::plan(catalog, "route-53", "GetDNSSEC")?;
        let (record, wire) = scenario.request(plan, &lease)?;
        assert_eq!(wire.method(), http::Method::GET);
        assert!(wire.uri().path().ends_with("/dnssec"));
        let response = call::DecodedResponse {
            record: fixture::decode(plan, format!("<GetDNSSECResponse>{status}{keys}</GetDNSSECResponse>").as_bytes())?,
            headers: http::HeaderMap::new(),
        };
        let request = PageRequest { record, region: routing::request_region(plan,
            lease.scope().region(), &scenario.home, lease.scope().branch())? };
        let ancestry = page::preparation::ancestry(lease.context(), lease.scope().context())?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let context = RuntimeContext {
            bootstrap: BootstrapContext { catalog, client: &self.client, executor: &scenario.executor,
                credentials: &self.credentials, home: &scenario.home, config: &self.config },
            store: &scenario.store, account: scenario.account.clone(), lease_ms: 60_000,
            endpoints: execution::discovery::EndpointCache::default(), consumers: ConsumerIndex::build(catalog)?,
        };
        Ok(page::commit_response(&context, lease, plan, cursor, response, request, &ancestry).await?)
    }
}

pub(super) fn failed() -> SweepSummary {
    SweepSummary { scopes: 1, outcomes: ScopeOutcomes { failed: 1, ..ScopeOutcomes::default() },
        ..SweepSummary::default() }
}

#[derive(Clone, Copy)]
pub(super) enum Checkpoint { Current, AfterFailure }

type Rows = Vec<Vec<String>>;
pub(super) type Snapshot = Vec<(String, Rows)>;

pub(super) async fn snapshot(scenario: &Scenario<'_, '_>, checkpoint: Checkpoint) -> TestResult<Snapshot> {
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(scenario.directory.path().join("inventory.db")).read_only(true)).await?;
    if matches!(checkpoint, Checkpoint::AfterFailure) {
        assert_eq!(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state=1")
            .bind(scenario.sweep.as_str()).fetch_one(&reader).await?, 1,
            "only the actual malformed-page lease may be finalized");
    }
    let mut tables = ["work_queue", "work_tokens", "resources", "resource_facets", "resource_scopes",
        "resource_parents", "unresolved_refs", "edges"]
        .into_iter().map(str::to_owned).collect::<Vec<_>>();
    // Include actual canonical kind records, not only resource IDs or their hashes.
    tables.extend(sqlx::query_scalar::<_, String>(
        "SELECT table_name FROM kinds WHERE kind IN(SELECT kind FROM resources) ORDER BY table_name")
        .fetch_all(&reader).await?);
    let mut snapshot = Vec::new();
    for table in tables {
        let columns = sqlx::query_scalar::<_, String>("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
            .bind(&table).fetch_all(&reader).await?;
        assert!(!columns.is_empty(), "atomicity snapshot table is absent: {table}");
        let finalize = table == "work_queue" && matches!(checkpoint, Checkpoint::AfterFailure);
        let projection = columns.iter().map(|column| match (finalize, column.as_str()) {
            // finish_scope intentionally closes the lease; every other persisted byte must survive.
            (true, "state") => "quote(CASE WHEN sweep_id=?1 AND state=1 THEN 2 ELSE state END)".to_owned(),
            (true, "lease_until") => "quote(CASE WHEN sweep_id=?1 AND state=1 THEN 0 ELSE lease_until END)".to_owned(),
            _ => format!("quote(\"{}\")", column.replace('"', "\"\"")),
        }).collect::<Vec<_>>().join(",");
        let sql = format!("SELECT {projection} FROM \"{}\" ORDER BY {projection}", table.replace('"', "\"\""));
        let query = sqlx::query(&sql);
        let query = if finalize { query.bind(scenario.sweep.as_str()) } else { query };
        let rows = query.fetch_all(&reader).await?.into_iter().map(|row| {
            (0..row.len()).map(|index| row.try_get::<String, _>(index)).collect::<Result<Vec<_>, _>>()
        }).collect::<Result<Rows, _>>()?;
        snapshot.push((table, rows));
    }
    reader.close().await;
    Ok(snapshot)
}

pub(super) async fn assert_finalized(scenario: &Scenario<'_, '_>) -> TestResult {
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!(summary.work_pending, 0);
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.failed_scopes, 1);
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(scenario.directory.path().join("inventory.db")).read_only(true)).await?;
    let outcomes = sqlx::query_scalar::<_, String>(
        "SELECT outcome FROM scope_outcomes WHERE sweep_id=?1 ORDER BY outcome")
        .bind(scenario.sweep.as_str()).fetch_all(&reader).await?;
    assert_eq!(outcomes, vec!["complete".to_owned(), "failed".to_owned()]);
    assert_eq!(sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state=2 AND lease_until=0")
        .bind(scenario.sweep.as_str()).fetch_one(&reader).await?, 2);
    reader.close().await;
    Ok(())
}
