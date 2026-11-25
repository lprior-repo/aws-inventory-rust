mod enrichment;
#[path = "../../../../cmdb-view/tests/postgres_support/mod.rs"]
mod pg;

use super::super::*;
use super::real::fixtures::{NativeGraph, assert_edges, graph, snapshot};
use pg::TestResult;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_swept_rows_when_worker_refreshes_then_table_counts_and_metadata_are_live()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    let worker = DataWorker::start(
        WorkerConfig {
            database: Some(fixture.url().to_owned()),
            ..WorkerConfig::default()
        },
        SignalToUI::default(),
    );
    let result: TestResult = async {
        assert_pg_startup(&worker).await?;
        fixture
            .commit(
                "refresh",
                "111111111111",
                "global",
                0,
                &["new", "shared"],
                50,
            )
            .await?;
        worker.refresh(Filter {
            account: Some("111111111111".into()),
            ..Filter::default()
        })?;
        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!((counts.resources, total), (5, 3));
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["new", "shared", "shared"]
        );
        assert_eq!(
            rows.first().and_then(|row| row.record_bytes),
            Some(pg::record("new", 50)?.as_bytes().len())
        );
        Ok(())
    }
    .await;
    drop(worker);
    fixture.finish().await?;
    result
}

async fn assert_pg_startup(worker: &DataWorker) -> TestResult {
    let (counts, rows, total) = snapshot(worker).await?;
    assert_eq!(
        counts,
        Counts {
            accounts: 2,
            regions: 4,
            services: 1,
            resources: 4
        }
    );
    assert_eq!(total, 4);
    assert_eq!(
        rows.iter()
            .map(|row| (
                row.account.as_str(),
                row.region.as_str(),
                row.kind.as_str(),
                row.id.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("111111111111", "global", "AWS::Example::Kind0", "shared"),
            ("111111111111", "us-east-1", "AWS::Example::Kind0", "shared"),
            ("222222222222", "eu-west-1", "AWS::Example::Kind0", "shared"),
            ("222222222222", "us-west-2", "AWS::Example::Kind1", "shared")
        ]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and FalkorDB"]
async fn postgres_given_native_graph_when_worker_expands_then_details_come_from_postgres()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    let root = pg::root();
    let target = pg::node("111111111111", "us-east-1", 0, "shared");
    let edges = vec![pg::edge(&root, &target, "replica", "catalog")];
    let native = NativeGraph::new(&edges).await?;
    let worker = DataWorker::start(
        WorkerConfig {
            database: Some(fixture.url().to_owned()),
            graph_url: Some(native.endpoint.clone()),
            graph_name: Some(native.name.clone()),
            ..WorkerConfig::default()
        },
        SignalToUI::default(),
    );
    let result: TestResult = async {
        assert_pg_startup_record(&worker, &fixture).await?;
        fixture
            .commit("expand", "111111111111", "global", 0, &["shared"], 70)
            .await?;
        worker
            .expand(root)
            .map_err(|error| format!("expand rejected: {error}"))?;
        let expanded = graph(&worker).await?;
        assert_edges(&expanded, &edges)?;
        let reader = fixture.reader().await?;
        let actual = expanded
            .nodes
            .iter()
            .find(|row| row.region == "global")
            .ok_or("missing expanded root")?;
        let stored = reader
            .detail(&pg::root())
            .await?
            .ok_or("missing PostgreSQL root")?;
        assert_eq!(actual, &projection::row_from_resource(&stored)?);
        assert_eq!(stored.record, pg::record("shared", 70)?);
        assert_eq!(expanded.nodes.len(), 2);
        Ok(())
    }
    .await;
    drop(worker);
    native.delete().await?;
    fixture.finish().await?;
    result
}

async fn assert_pg_startup_record(worker: &DataWorker, fixture: &pg::Fixture) -> TestResult {
    let (counts, rows, total) = snapshot(worker).await?;
    assert_eq!((counts.resources, total), (4, 4));
    let actual = rows
        .iter()
        .find(|row| row.region == "global")
        .ok_or("missing startup root")?;
    let reader = fixture.reader().await?;
    let stored = reader
        .detail(&pg::root())
        .await?
        .ok_or("missing PostgreSQL startup root")?;
    assert_eq!(stored.record, pg::record("shared", 10)?);
    assert_eq!(actual, &projection::row_from_resource(&stored)?);
    Ok(())
}
