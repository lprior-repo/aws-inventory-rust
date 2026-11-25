use super::*;
use crate::worker::tests::real::fixtures::await_event;

struct EnrichmentFixture {
    postgres: pg::Fixture,
    native: NativeGraph,
    worker: DataWorker,
}

impl EnrichmentFixture {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let postgres = pg::Fixture::new().await?;
        let target = pg::node("111111111111", "us-east-1", 0, "shared");
        let link = pg::edge(&pg::root(), &target, "replica", "catalog");
        let native = NativeGraph::new(std::slice::from_ref(&link)).await?;
        let worker = DataWorker::start(
            WorkerConfig {
                database: Some(postgres.url().to_owned()),
                graph_url: Some(native.endpoint.clone()),
                graph_name: Some(native.name.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
            SignalToUI::default(),
        );
        assert_pg_startup_record(&worker, &postgres).await?;
        worker.expand(pg::root())?;
        let established = graph(&worker).await?;
        assert_edges(&established, std::slice::from_ref(&link))?;
        Ok(Self {
            postgres,
            native,
            worker,
        })
    }

    fn assert_error(&self, event: WorkerEvent, expected: &str) -> TestResult {
        let WorkerEvent::Graph { graph, error } = event else {
            return Err(format!("expected terminal enrichment failure, got {event:?}").into());
        };
        let target = pg::node("111111111111", "us-east-1", 0, "shared");
        assert_edges(
            &graph,
            &[pg::edge(&pg::root(), &target, "replica", "catalog")],
        )?;
        assert_eq!(
            graph
                .nodes
                .iter()
                .map(|row| (
                    row.region.as_str(),
                    row.record_bytes,
                    row.content_hash.as_deref(),
                ))
                .collect::<Vec<_>>(),
            vec![("global", None, None), ("us-east-1", None, None)],
        );
        assert_eq!(error.as_deref(), Some(expected));
        Ok(())
    }

    async fn finish(self) -> TestResult {
        drop(self.worker);
        self.native.delete().await?;
        self.postgres.finish().await
    }
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and FalkorDB"]
async fn postgres_given_missing_payload_when_graph_enriches_then_detail_failure_is_diagnostic()
-> TestResult {
    let fixture = EnrichmentFixture::new().await?;
    sqlx::query("DELETE FROM inventory.kind_0")
        .execute(&fixture.postgres.pool)
        .await?;
    fixture.worker.expand(pg::root())?;
    let event = await_event(&fixture.worker, false).await?;
    let result = fixture.assert_error(
        event,
        "Database snapshot detail query failed · invalid stored resource row",
    );
    fixture.finish().await?;
    result
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and FalkorDB"]
async fn postgres_given_locked_payload_when_graph_enriches_then_detail_timeout_is_diagnostic()
-> TestResult {
    let fixture = EnrichmentFixture::new().await?;
    let mut writer = fixture.postgres.pool.begin().await?;
    sqlx::query("LOCK TABLE inventory.kind_0 IN ACCESS EXCLUSIVE MODE NOWAIT")
        .execute(&mut *writer)
        .await?;
    fixture.worker.expand(pg::root())?;
    let event = await_event(&fixture.worker, false).await;
    writer.rollback().await?;
    let result = fixture.assert_error(
        event?,
        "Database snapshot detail query timed out after 2 seconds",
    );
    fixture.finish().await?;
    result
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and FalkorDB"]
async fn postgres_given_row_locked_payload_when_graph_enriches_then_mvcc_read_is_not_blocked()
-> TestResult {
    let fixture = EnrichmentFixture::new().await?;
    let root = pg::root();
    let mut writer = fixture.postgres.pool.begin().await?;
    let locked: String = sqlx::query_scalar("SELECT id FROM inventory.kind_0 WHERE account=$1 AND region=$2 AND id=$3 FOR UPDATE NOWAIT")
        .bind(&root.account).bind(&root.region).bind(&root.id)
        .fetch_one(&mut *writer).await?;
    assert_eq!(locked, root.id);
    fixture.worker.expand(root)?;
    let neighborhood = graph(&fixture.worker).await;
    writer.rollback().await?;
    let neighborhood = neighborhood?;
    let target = pg::node("111111111111", "us-east-1", 0, "shared");
    assert_edges(
        &neighborhood,
        &[pg::edge(&pg::root(), &target, "replica", "catalog")],
    )?;
    assert_eq!(
        neighborhood
            .nodes
            .iter()
            .filter(|row| row.record_bytes.is_some())
            .count(),
        2
    );
    fixture.finish().await
}
