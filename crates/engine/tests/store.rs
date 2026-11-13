use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::{GraphExportCheckpoint, MAX_QUERY_ROWS, Store, StoreError};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult = Result<(), StoreError>;

fn scope(collector: &str, source: Source) -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        collector: collector.to_owned(),
        parent_id: None,
        source,
    }
}

fn resource(id: &str, source: Source, observed_at: i64) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "aws_ec2_instance".to_owned(),
        name: Some("worker".to_owned()),
        properties: json!({"instance_id": id, "instance_type": "m7i.large", "state": "running"}),
        tags: BTreeMap::from([("env".to_owned(), "production".to_owned())]),
        source,
        observed_at,
    }
}

fn page(collector: &str, source: Source, resources: Vec<Resource>) -> CollectionPage {
    CollectionPage {
        scope: scope(collector, source),
        page_number: 1,
        next_token: None,
        resources,
        edges: Vec::new(),
    }
}

fn relationship() -> Edge {
    Edge {
        from_id: "i-1".to_owned(),
        to_id: "vpc-unresolved".to_owned(),
        relation: "IN_VPC".to_owned(),
        source: Source::Direct,
        observed_at: 10,
        properties: json!({}),
    }
}

#[tokio::test]
async fn exact_page_replay_is_idempotent_even_after_finalization() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges.push(relationship());
    store.apply_page("one", input.clone()).await?;
    store.apply_page("one", input.clone()).await?;
    let summary = store.finish_scan("one", ScanStatus::Complete, 20).await?;
    assert_eq!(summary.resources, 1);
    assert_eq!(summary.edges, 1);
    assert_eq!(summary.completed_scopes, 1);
    store.apply_page("one", input.clone()).await?;
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM collection_pages", 10)
            .await?,
        json!([{"total": 1}])
    );
    assert_eq!(store.summary("one").await?, summary);
    Ok(())
}

#[tokio::test]
async fn changed_replay_cannot_overwrite_committed_observation() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    store.apply_page("one", input.clone()).await?;
    let changed = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 11)],
    );
    assert!(matches!(
        store.apply_page("one", changed.clone()).await,
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(store.resources("one").await?, input.resources);
    Ok(())
}

#[tokio::test]
async fn observation_failure_rolls_back_page_checkpoint_edges_and_other_observations() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let duplicate = resource("i-1", Source::Direct, 10);
    let mut broken = page("ec2", Source::Direct, vec![duplicate.clone(), duplicate]);
    broken.edges.push(relationship());
    assert!(store.apply_page("one", broken.clone()).await.is_err());
    assert!(store.checkpoint("one", &broken.scope).await?.is_none());
    assert_eq!(store.summary("one").await?.resources, 0);
    assert_eq!(store.summary("one").await?.edges, 0);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM collection_pages", 10)
            .await?,
        json!([{"total": 0}])
    );
    broken.resources.truncate(1);
    store.apply_page("one", broken.clone()).await?;
    assert_eq!(store.summary("one").await?.resources, 1);
    assert_eq!(store.summary("one").await?.edges, 1);
    Ok(())
}

#[tokio::test]
async fn edge_failure_also_rolls_back_resources_and_checkpoint() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut broken = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    broken.edges = vec![relationship(), relationship()];
    assert!(store.apply_page("one", broken.clone()).await.is_err());
    assert_eq!(store.summary("one").await?.resources, 0);
    assert!(store.checkpoint("one", &broken.scope).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn pages_require_contiguous_order_and_reject_continuation_cycles() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut input = page("ec2", Source::Direct, Vec::new());
    input.page_number = 2;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.page_number = 1;
    input.next_token = Some("first".to_owned());
    store.apply_page("one", input.clone()).await?;
    input.page_number = 3;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.page_number = 2;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.next_token = Some("second".to_owned());
    store.apply_page("one", input.clone()).await?;
    input.page_number = 3;
    input.next_token = Some("first".to_owned());
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.next_token = None;
    store.apply_page("one", input.clone()).await?;
    let checkpoint = store
        .checkpoint("one", &input.scope)
        .await?
        .ok_or(StoreError::Conflict("missing expected checkpoint"))?;
    assert_eq!(checkpoint.last_page, 3);
    assert!(checkpoint.complete);
    Ok(())
}

#[tokio::test]
async fn empty_terminal_page_proves_empty_scope_but_unfinished_page_does_not() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("empty", 1).await?;
    store
        .apply_page("empty", page("ec2", Source::Direct, Vec::new()))
        .await?;
    assert_eq!(
        store
            .finish_scan("empty", ScanStatus::Complete, 2)
            .await?
            .completed_scopes,
        1
    );
    assert_eq!(store.scan_status("empty").await?, ScanStatus::Complete);
    store.begin_scan("unfinished", 3).await?;
    let mut unfinished = page("ec2", Source::Direct, Vec::new());
    unfinished.next_token = Some("more".to_owned());
    store.apply_page("unfinished", unfinished.clone()).await?;
    store
        .finish_scan("unfinished", ScanStatus::Complete, 4)
        .await?;
    assert_eq!(store.scan_status("unfinished").await?, ScanStatus::Partial);
    store.begin_scan("no-scopes", 5).await?;
    store
        .finish_scan("no-scopes", ScanStatus::Complete, 6)
        .await?;
    assert_eq!(store.scan_status("no-scopes").await?, ScanStatus::Partial);
    Ok(())
}

#[tokio::test]
async fn failed_scopes_are_durable_idempotent_and_prevent_complete_status() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    store
        .apply_page(
            "one",
            page(
                "ec2",
                Source::Direct,
                vec![resource("i-1", Source::Direct, 10)],
            ),
        )
        .await?;
    let failure = CollectionFailure {
        scope: scope("lambda", Source::Direct),
        category: "access_denied".to_owned(),
        message: "denied '; DROP TABLE scans; --".to_owned(),
    };
    store.record_failure("one", &failure).await?;
    store.record_failure("one", &failure).await?;
    let summary = store.finish_scan("one", ScanStatus::Complete, 20).await?;
    assert_eq!(summary.failed_scopes, 1);
    assert_eq!(summary.completed_scopes, 1);
    assert_eq!(store.scan_status("one").await?, ScanStatus::Partial);
    assert_eq!(
        store
            .query("SELECT category, message FROM collection_failures", 10)
            .await?,
        json!([{"category": "access_denied", "message": failure.message}])
    );
    Ok(())
}

#[tokio::test]
async fn preferred_source_and_latest_observation_within_source_are_deterministic() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, source, time) in [
        ("explorer", Source::ResourceExplorer, 100),
        ("direct-a", Source::Direct, 10),
        ("config", Source::AwsConfig, 200),
        ("direct-b", Source::Direct, 20),
    ] {
        store
            .apply_page(
                "one",
                page(collector, source, vec![resource("i-1", source, time)]),
            )
            .await?;
    }
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM latest_resources", 10)
            .await?,
        json!([{"source":"direct", "observed_at":20}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_resource_sources", 10)
            .await?,
        json!([{"total":3}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM resource_observations", 10)
            .await?,
        json!([{"total":4}])
    );
    assert_eq!(store.summary("one").await?.resources, 1);
    Ok(())
}

#[tokio::test]
async fn equal_timestamp_source_ties_ignore_page_arrival_order() -> TestResult {
    let first = Store::open(":memory:").await?;
    let second = Store::open(":memory:").await?;
    first.begin_scan("one", 1).await?;
    second.begin_scan("one", 1).await?;
    let mut a = resource("i-1", Source::Direct, 10);
    a.name = Some("alpha".to_owned());
    let mut b = a.clone();
    b.name = Some("beta".to_owned());
    let alpha = page("a", Source::Direct, vec![a]);
    let beta = page("b", Source::Direct, vec![b]);
    first.apply_page("one", alpha.clone()).await?;
    first.apply_page("one", beta.clone()).await?;
    second.apply_page("one", beta.clone()).await?;
    second.apply_page("one", alpha.clone()).await?;
    assert_eq!(
        first.resources("one").await?,
        second.resources("one").await?
    );
    assert_eq!(
        first.query("SELECT name FROM latest_resources", 10).await?,
        json!([{"name":"alpha"}])
    );
    Ok(())
}

#[tokio::test]
async fn partial_snapshot_never_erases_previous_resources_or_edges() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("old", 1).await?;
    let mut old = page(
        "ec2",
        Source::Direct,
        vec![
            resource("i-1", Source::Direct, 10),
            resource("i-2", Source::Direct, 10),
        ],
    );
    old.edges.push(relationship());
    store.apply_page("old", old.clone()).await?;
    store.finish_scan("old", ScanStatus::Complete, 20).await?;
    store.begin_scan("new", 30).await?;
    store
        .apply_page(
            "new",
            page(
                "ec2",
                Source::Direct,
                vec![resource("i-1", Source::Direct, 40)],
            ),
        )
        .await?;
    store
        .record_failure(
            "new",
            &CollectionFailure {
                scope: scope("lambda", Source::Direct),
                category: "timeout".to_owned(),
                message: "late".to_owned(),
            },
        )
        .await?;
    store.finish_scan("new", ScanStatus::Complete, 50).await?;
    assert_eq!(store.scan_status("new").await?, ScanStatus::Partial);
    assert_eq!(store.resources("new").await?.len(), 1);
    assert_eq!(
        store
            .query("SELECT id FROM latest_resources ORDER BY id", 10)
            .await?,
        json!([{"id":"i-1"},{"id":"i-2"}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_edges", 10)
            .await?,
        json!([{"total":1}])
    );
    Ok(())
}

#[tokio::test]
async fn query_denies_writes_pragma_transactions_and_external_side_effects() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for forbidden in [
        "DELETE FROM scans",
        "UPDATE scans SET status='complete'",
        "DROP TABLE scans",
        "CREATE TABLE injected(x)",
        "CREATE TEMP TABLE injected(x)",
        "BEGIN",
        "COMMIT",
        "PRAGMA query_only=OFF",
        "PRAGMA user_version=9",
        "ATTACH DATABASE ':memory:' AS other",
        "VACUUM",
        "SELECT load_extension('untrusted')",
        "SELECT writefile('/tmp/untrusted', 'x')",
        "WITH n AS (SELECT 1) DELETE FROM scans RETURNING scan_id",
        "SELECT * FROM pragma_query_only('OFF')",
    ] {
        assert!(
            store.query(forbidden, 10).await.is_err(),
            "accepted forbidden SQL: {forbidden}"
        );
    }
    assert_eq!(store.scan_status("one").await?, ScanStatus::Running);
    store.begin_scan("after-denials", 2).await?;
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM scans", 10)
            .await?,
        json!([{"total":2}])
    );
    Ok(())
}

#[tokio::test]
async fn query_single_statement_check_understands_quotes_and_comments() -> TestResult {
    let store = Store::open(":memory:").await?;
    for forbidden in [
        "SELECT 1; DELETE FROM scans",
        "SELECT 1; /* hidden */ SELECT 2",
        "SELECT 1; -- hidden\nSELECT 2",
        "SELECT ';'; SELECT 2",
        "SELECT 1;;",
        "SELECT 1 /* unterminated",
        "SELECT 'unterminated",
    ] {
        assert!(
            store.query(forbidden, 10).await.is_err(),
            "accepted SQL: {forbidden}"
        );
    }
    assert_eq!(
        store
            .query(
                "SELECT '; DROP TABLE scans; --' AS value; /* harmless */",
                10
            )
            .await?,
        json!([{"value":"; DROP TABLE scans; --"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT 'it''s; quoted' AS \"column;name\" -- comment;\n",
                10
            )
            .await?,
        json!([{"column;name":"it's; quoted"}])
    );
    assert_eq!(
        store
            .query("SELECT 1 AS [bracket;name], 2 AS `backtick;name`", 10)
            .await?,
        json!([{"bracket;name":1, "backtick;name":2}])
    );
    Ok(())
}

#[tokio::test]
async fn quoted_identifiers_and_values_are_bound_in_ingestion_paths() -> TestResult {
    let store = Store::open(":memory:").await?;
    let scan = "scan'); DROP TABLE scans; --";
    store.begin_scan(scan, 1).await?;
    let input = page(
        "ec2'",
        Source::Direct,
        vec![resource("i-'quoted", Source::Direct, 10)],
    );
    store.apply_page(scan, input.clone()).await?;
    assert_eq!(store.resources(scan).await?, input.resources);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM scans", 10)
            .await?,
        json!([{"total":1}])
    );
    Ok(())
}

#[tokio::test]
async fn query_caps_rows_columns_output_and_instruction_work() -> TestResult {
    let store = Store::open(":memory:").await?;
    assert!(matches!(
        store.query("SELECT 1 UNION ALL SELECT 2", 1).await,
        Err(StoreError::Limit(_))
    ));
    assert!(store.query("SELECT 1", 0).await.is_err());
    assert!(
        store
            .query("SELECT 1", MAX_QUERY_ROWS.saturating_add(1))
            .await
            .is_err()
    );
    assert_eq!(
        store
            .query("SELECT printf('%9000000s','a') AS large", 1)
            .await?,
        json!([{"large": null}])
    );
    assert!(matches!(
        store
            .query(
                "SELECT printf('%4500000s','a') AS large UNION ALL SELECT printf('%4500000s','b')",
                2
            )
            .await,
        Err(StoreError::Limit(_))
    ));
    assert!(store.query("SELECT 1 AS same,2 AS same", 10).await.is_err());
    assert!(store.query(
        "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n) SELECT sum(x) FROM n", 1,
    ).await.is_err());
    store.begin_scan("after-interruption", 1).await?;
    assert_eq!(
        store.query("SELECT upper('ok') AS value", 10).await?,
        json!([{"value":"OK"}])
    );
    Ok(())
}

#[tokio::test]
async fn sql_views_expose_selected_configuration_and_provenance() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut lambda = resource(
        "arn:aws:lambda:us-east-1:111111111111:function:worker",
        Source::Direct,
        10,
    );
    lambda.kind = "aws_lambda_function".to_owned();
    lambda.properties =
        json!({"function_name":"worker", "runtime":"provided.al2023", "memory_size":256});
    store
        .apply_page("one", page("lambda", Source::Direct, vec![lambda]))
        .await?;
    assert_eq!(
        store.query(
            "SELECT function_name,runtime,memory_size,source,observed_at FROM aws_lambda_function",
            10
        ).await?,
        json!([{"function_name":"worker", "runtime":"provided.al2023", "memory_size":256,
            "source":"direct", "observed_at":10}])
    );
    for view in [
        "aws_vpc",
        "aws_vpc_subnet",
        "aws_vpc_security_group",
        "aws_dynamodb_table",
        "aws_ecs_cluster",
        "aws_ecs_service",
        "aws_ecs_task",
        "aws_eks_cluster",
        "aws_eks_node_group",
    ] {
        assert_eq!(
            store.query(&format!("SELECT * FROM {view}"), 10).await?,
            json!([])
        );
    }
    Ok(())
}

#[tokio::test]
async fn reopen_preserves_atomic_page_and_resume_token() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges.push(relationship());
    input.next_token = Some("continuation".to_owned());
    {
        let store = Store::open(&path).await?;
        store.begin_scan("one", 1).await?;
        store.apply_page("one", input.clone()).await?;
        store.close().await;
    }
    let reopened = Store::open(&path).await?;
    assert_eq!(reopened.resources("one").await?, input.resources);
    assert_eq!(reopened.edges("one").await?, input.edges);
    let saved = reopened
        .checkpoint("one", &input.scope)
        .await?
        .ok_or(StoreError::Conflict("missing persisted checkpoint"))?;
    assert_eq!(saved.last_page, 1);
    assert_eq!(saved.next_token.as_deref(), Some("continuation"));
    assert!(!saved.complete);
    reopened.apply_page("one", input.clone()).await?;
    let mut connection = <sqlx::SqliteConnection as sqlx::Connection>::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new().filename(&path),
    )
    .await?;
    let mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(mode, "wal");
    sqlx::Connection::close(connection).await?;
    reopened.close().await;
    Ok(())
}

#[tokio::test]
async fn streaming_exports_propagate_consumer_failure() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    store
        .apply_page(
            "one",
            page(
                "ec2",
                Source::Direct,
                vec![
                    resource("i-a", Source::Direct, 10),
                    resource("i-b", Source::Direct, 10),
                ],
            ),
        )
        .await?;
    let mut visits = 0_u32;
    let error = store
        .visit_resources("one", |_| {
            visits = visits.saturating_add(1);
            Err(StoreError::Io(std::io::Error::other("consumer stopped")))
        })
        .await;
    assert!(matches!(error, Err(StoreError::Io(_))));
    assert_eq!(visits, 1);
    assert_eq!(store.resources("one").await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn graph_checkpoints_are_monotonic_and_require_a_frozen_scan() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut checkpoint = GraphExportCheckpoint {
        scan_id: "one".to_owned(),
        target: "graph".to_owned(),
        snapshot_id: "snapshot-a".to_owned(),
        resources_exported: 1,
        edges_exported: 0,
        complete: false,
    };
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    store.finish_scan("one", ScanStatus::Partial, 2).await?;
    store.save_graph_checkpoint(&checkpoint).await?;
    store.save_graph_checkpoint(&checkpoint).await?;
    assert_eq!(
        store.graph_checkpoint("one", "graph", "snapshot-a").await?,
        Some(checkpoint.clone())
    );
    checkpoint.resources_exported = 0;
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    checkpoint.resources_exported = 2;
    checkpoint.complete = true;
    store.save_graph_checkpoint(&checkpoint).await?;
    checkpoint.complete = false;
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    Ok(())
}

#[tokio::test]
async fn invalid_scan_lifecycle_does_not_reopen_terminal_scans() -> TestResult {
    let store = Store::open(":memory:").await?;
    assert!(matches!(
        store.summary("unknown").await,
        Err(StoreError::MissingScan(_))
    ));
    assert!(store.begin_scan("one", -1).await.is_err());
    store.begin_scan("one", 10).await?;
    store.begin_scan("one", 10).await?;
    assert!(store.begin_scan("one", 11).await.is_err());
    assert!(
        store
            .finish_scan("one", ScanStatus::Running, 20)
            .await
            .is_err()
    );
    assert!(
        store
            .finish_scan("one", ScanStatus::Cancelled, 9)
            .await
            .is_err()
    );
    store.finish_scan("one", ScanStatus::Cancelled, 20).await?;
    assert_eq!(store.scan_status("one").await?, ScanStatus::Cancelled);
    assert!(store.begin_scan("one", 10).await.is_err());
    assert!(
        store
            .apply_page("one", page("ec2", Source::Direct, Vec::new()))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn cancelled_query_keeps_runtime_responsive_and_reader_reusable() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("cancel.sqlite")).await?;
    let query = store.query(
        "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n) SELECT sum(x) FROM n",
        1,
    );
    let cancelled = tokio::time::timeout(std::time::Duration::from_millis(5), query).await;
    assert!(cancelled.is_err());
    store.begin_scan("writer-still-works", 1).await?;
    let response = store.query("SELECT sqlite_version() AS version", 1).await?;
    assert!(
        response
            .get(0)
            .and_then(|row| row.get("version"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|version| !version.is_empty())
    );
    assert_eq!(
        store.scan_status("writer-still-works").await?,
        ScanStatus::Running
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn file_reader_denies_cte_writes_and_preserves_bound_values() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("readonly.sqlite")).await?;
    store.begin_scan("quoted ' value", 1).await?;
    assert!(
        store
            .query(
                "WITH x AS (SELECT 1) DELETE FROM scans RETURNING scan_id",
                10
            )
            .await
            .is_err()
    );
    assert!(
        store
            .query("SELECT 1; ATTACH ':memory:' AS injected", 10)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .query("/* prefix */ SELECT scan_id FROM scans; -- suffix", 10)
            .await?,
        json!([{"scan_id":"quoted ' value"}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn separate_memory_stores_do_not_share_data() -> TestResult {
    let first = Store::open(":memory:").await?;
    let second = Store::open(":memory:").await?;
    first.begin_scan("private", 1).await?;
    assert!(matches!(
        second.summary("private").await,
        Err(StoreError::MissingScan(_))
    ));
    first.close().await;
    second.close().await;
    Ok(())
}

#[tokio::test]
async fn query_bounds_native_columns_and_decoded_cells() -> TestResult {
    let store = Store::open(":memory:").await?;
    let columns = (0..257)
        .map(|index| format!("1 AS c{index}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(store.query(&format!("SELECT {columns}"), 1).await.is_err());
    // Exercise the cell budget with fewer SQLx row handoffs so this boundary
    // test does not need to approach the separate native query deadline.
    let cell_columns = (0..100)
        .map(|index| format!("x AS c{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let cell_query = |rows| {
        format!(
            "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<{rows})
             SELECT {cell_columns} FROM n ORDER BY x"
        )
    };
    let at_limit = store.query(&cell_query(1_000), 1_000).await?;
    let rows = at_limit
        .as_array()
        .ok_or(StoreError::InvalidInput("query result must be an array"))?;
    assert_eq!(rows.len(), 1_000);
    assert!(
        rows.iter()
            .all(|row| row.as_object().is_some_and(|columns| columns.len() == 100))
    );
    let over_limit = store.query(&cell_query(1_001), 1_001).await;
    assert!(
        matches!(&over_limit, Err(StoreError::Limit("query cells"))),
        "expected the query-cell limit, received {over_limit:?}"
    );
    for query in [
        "SELECT x'ff' AS blob",
        "SELECT CAST(x'ff' AS TEXT) AS invalid_utf8",
        "SELECT 1e999 AS nonfinite",
    ] {
        assert!(store.query(query, 1).await.is_err());
    }
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn empty_resource_name_preserves_valid_aws_tag_value() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("empty-name", 1).await?;
    let mut instance = resource("i-empty-name", Source::Direct, 10);
    instance.name = Some(String::new());
    instance.tags.insert("Name".to_owned(), String::new());
    store
        .apply_page(
            "empty-name",
            page("ec2", Source::Direct, vec![instance.clone()]),
        )
        .await?;
    assert_eq!(store.resources("empty-name").await?, vec![instance]);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn duplicate_resource_after_first_sql_batch_rolls_back_entire_page() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("batch-failure", 1).await?;
    let mut resources: Vec<_> = (0..128_u32)
        .map(|index| resource(&format!("i-{index}"), Source::Direct, 10))
        .collect();
    resources.push(resource("i-0", Source::Direct, 10));
    let input = page("ec2", Source::Direct, resources);
    let scope = input.scope.clone();
    assert!(store.apply_page("batch-failure", input).await.is_err());
    assert_eq!(store.summary("batch-failure").await?.resources, 0);
    assert!(store.checkpoint("batch-failure", &scope).await?.is_none());
    assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await?,
        json!([{"pages":0}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn duplicate_edge_after_first_sql_batch_rolls_back_resources_too() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("edge-batch-failure", 1).await?;
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges = (0..128_u32)
        .map(|index| Edge {
            from_id: format!("i-{index}"),
            ..relationship()
        })
        .collect();
    input.edges.push(Edge {
        from_id: "i-0".to_owned(),
        ..relationship()
    });
    let scope = input.scope.clone();
    assert!(store.apply_page("edge-batch-failure", input).await.is_err());
    let summary = store.summary("edge-batch-failure").await?;
    assert_eq!(summary.resources, 0);
    assert_eq!(summary.edges, 0);
    assert!(
        store
            .checkpoint("edge-batch-failure", &scope)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await?,
        json!([{"pages":0}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn successful_page_roundtrips_all_rows_across_multiple_sql_batches() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("many-rows", 1).await?;
    let resources: Vec<_> = (0..257_u32)
        .map(|index| resource(&format!("i-{index:03}"), Source::Direct, 10))
        .collect();
    let edges: Vec<_> = resources
        .iter()
        .map(|resource| Edge {
            from_id: resource.id.clone(),
            ..relationship()
        })
        .collect();
    let mut input = page("ec2", Source::Direct, resources.clone());
    input.edges = edges.clone();
    store.apply_page("many-rows", input.clone()).await?;
    store.apply_page("many-rows", input).await?;
    assert_eq!(store.resources("many-rows").await?, resources);
    assert_eq!(store.edges("many-rows").await?, edges);
    let summary = store
        .finish_scan("many-rows", ScanStatus::Complete, 20)
        .await?;
    assert_eq!(summary.resources, 257);
    assert_eq!(summary.edges, 257);
    assert_eq!(summary.completed_scopes, 1);
    store.close().await;
    Ok(())
}
