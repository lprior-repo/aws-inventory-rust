pub(crate) mod catalog;
pub(crate) mod fixtures;
mod tags;

use super::super::*;
use super::isolation::supervised_async;
use fixtures::{
    ACCOUNT, Fixture, KIND, NativeGraph, REGION, TestResult, assert_edges, digest, edge,
    falkor_available, graph, node, root, snapshot,
};

fn sort_rows(rows: &mut [Row]) {
    rows.sort_by_key(|row| {
        (
            row.account.clone(),
            row.region.clone(),
            row.kind.clone(),
            row.id.clone(),
        )
    });
}

#[test]
fn given_divergent_stores_and_concurrent_sweep_when_expanding_then_native_topology_uses_committed_sql_details()
-> TestResult {
    supervised_async(|| async {
        if !falkor_available().await? {
            return Ok(());
        }
        let fixture = Fixture::new().await?;
        let a = root();
        let b = node(ACCOUNT, "us-west-2", KIND, "shared");
        let c = node(ACCOUNT, REGION, "AWS::Other::Thing", "shared");
        let d = node(ACCOUNT, REGION, KIND, "sql-only");
        let e = node("222222222222", REGION, KIND, "shared");
        let common = edge(&a, &b, "references", "reviewed-rule");
        let sql_only = edge(&a, &d, "sql-only", "model");
        let native_edges = vec![
            common.clone(),
            edge(&b, &c, "native-only", "reviewed-rule"),
            edge(&a, &e, "cross-account", "inferred-exact"),
        ];
        let mut expected = fixture
            .commit(
                "before-west",
                ACCOUNT,
                "us-west-2",
                &[("shared", &fixtures::arn(&b))],
                &[],
                1,
            )
            .await?;
        expected.extend(
            fixture
                .commit(
                    "before-account",
                    "222222222222",
                    REGION,
                    &[("shared", "account-before")],
                    &[],
                    1,
                )
                .await?,
        );
        expected.extend(
            fixture
                .commit(
                    "before-root",
                    ACCOUNT,
                    REGION,
                    &[("shared", "root-before"), ("sql-only", "sql-only-before")],
                    &[common.clone(), sql_only.clone()],
                    1,
                )
                .await?,
        );
        sort_rows(&mut expected);
        let native = NativeGraph::new(&native_edges).await?;
        let worker = fixture.worker(Some(&native));
        let result: TestResult = async {
            let (counts, initial, total) = snapshot(&worker).await?;
            assert_eq!(
                counts,
                Counts {
                    accounts: 2,
                    regions: 2,
                    services: 1,
                    resources: 4
                }
            );
            assert_eq!(total, 4);
            assert_eq!(initial, expected);
            let database =
                Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
            let sql = database
                .edges_page(
                    &a,
                    cmdb_view::EdgePageRequest {
                        limit: 20,
                        after: None,
                    },
                )
                .await?;
            let mut sql_expected = vec![common, sql_only];
            sql_expected.sort_by_key(|edge| {
                (
                    edge.to.account.clone(),
                    edge.to.region.clone(),
                    edge.to.kind.clone(),
                    edge.to.id.clone(),
                    edge.relation.clone(),
                    edge.provenance.clone(),
                )
            });
            assert_eq!(sql.edges, sql_expected);
            assert_eq!(sql.total_matching, 2);
            assert_eq!(sql.next, None);

            let mut after = fixture
                .commit(
                    "after-west",
                    ACCOUNT,
                    "us-west-2",
                    &[("shared", "west-after-a-longer-payload")],
                    &[],
                    2,
                )
                .await?;
            after.extend(
                fixture
                    .commit(
                        "after-account",
                        "222222222222",
                        REGION,
                        &[("shared", "account-after-a-longer-payload")],
                        &[],
                        2,
                    )
                    .await?,
            );
            after.extend(
                fixture
                    .commit(
                        "after-root",
                        ACCOUNT,
                        REGION,
                        &[
                            ("shared", "root-after-a-longer-payload"),
                            ("sql-only", "sql-only-after"),
                            ("new", "added-after"),
                        ],
                        &[],
                        2,
                    )
                    .await?,
            );
            sort_rows(&mut after);
            assert_eq!(database.counts().await?.resources, 5);
            worker.expand(a.clone())?;
            let retained = graph(&worker).await?;

            assert_edges(&retained, &native_edges)?;
            assert!(!retained.truncated);
            assert_eq!(retained.nodes.len(), 4);
            for identity in [&a, &b, &e] {
                let key = digest(identity)?;
                assert_eq!(
                    retained.nodes.iter().find(|row| row.key == key),
                    after.iter().find(|row| row.key == key)
                );
            }
            let native_key = digest(&c)?;
            let native_only = retained
                .nodes
                .iter()
                .find(|row| row.key == native_key)
                .ok_or("native-only node missing")?;
            assert_eq!(
                native_only,
                &Row {
                    account: c.account.clone(),
                    region: c.region.clone(),
                    kind: c.kind.clone(),
                    id: c.id.clone(),
                    name: c.id.clone(),
                    key: digest(&c)?,
                    record_bytes: None,
                    content_hash: None,
                    tags: Vec::new(),
                    tags_truncated: false,
                    tag_observation: None
                }
            );
            let sql_only_key = digest(&d)?;
            assert_eq!(
                retained.nodes.iter().find(|row| row.key == sql_only_key),
                None
            );
            let direct = GraphConnection::connect(&native.endpoint, &native.name)?
                .neighbors(&a, DEFAULT_GRAPH_LIMITS)
                .await?;
            assert_edges(&retained, &direct.edges)?;
            assert_eq!(retained.truncated, direct.truncated);

            worker.refresh(Filter::default())?;
            let (counts, refreshed, total) = snapshot(&worker).await?;
            assert_eq!(counts.resources, 5);
            assert_eq!(total, 5);
            assert_eq!(refreshed, after);
            let refreshed_graph = graph(&worker).await?;
            assert_edges(&refreshed_graph, &native_edges)?;
            for identity in [&a, &b, &e] {
                let key = digest(identity)?;
                assert_eq!(
                    refreshed_graph.nodes.iter().find(|row| row.key == key),
                    refreshed.iter().find(|row| row.key == key)
                );
            }
            worker.expand(a.clone())?;
            let updated = graph(&worker).await?;
            assert_edges(&updated, &native_edges)?;
            for identity in [&a, &b, &e] {
                let key = digest(identity)?;
                assert_eq!(
                    updated.nodes.iter().find(|row| row.key == key),
                    refreshed.iter().find(|row| row.key == key)
                );
            }
            worker.expand(c)?;
            let from_native_only = graph(&worker).await?;
            assert_edges(
                &from_native_only,
                native_edges.get(..2).ok_or("native edges missing")?,
            )?;
            Ok(())
        }
        .await;
        drop(worker);
        let cleanup = native.delete().await;
        result?;
        cleanup
    })
}

#[test]
fn given_depth_boundary_chain_when_real_worker_expands_then_truncation_warning_propagates_and_clears()
-> TestResult {
    supervised_async(|| async {
        if !falkor_available().await? {
            return Ok(());
        }
        let fixture = Fixture::new().await?;
        fixture
            .commit(
                "chain",
                ACCOUNT,
                REGION,
                &[
                    ("shared", "root"),
                    ("b", "second"),
                    ("c", "third"),
                    ("d", "fourth"),
                    ("x", "small-root"),
                    ("y", "small-leaf"),
                ],
                &[],
                1,
            )
            .await?;
        let a = root();
        let b = node(ACCOUNT, REGION, KIND, "b");
        let c = node(ACCOUNT, REGION, KIND, "c");
        let d = node(ACCOUNT, REGION, KIND, "d");
        let x = node(ACCOUNT, REGION, KIND, "x");
        let y = node(ACCOUNT, REGION, KIND, "y");
        let edges = vec![
            edge(&a, &b, "first", "model"),
            edge(&b, &c, "second", "reviewed-rule"),
            edge(&c, &d, "beyond-depth", "inferred-exact"),
            edge(&x, &y, "complete", "model"),
        ];
        let native = NativeGraph::new(&edges).await?;
        let worker = fixture.worker(Some(&native));
        let result: TestResult = async {
            let (_, rows, total) = snapshot(&worker).await?;
            assert_eq!(rows.len(), 6);
            assert_eq!(total, 6);
            worker.expand(a)?;
            let truncated = graph(&worker).await?;
            assert_edges(&truncated, edges.get(..2).ok_or("edges missing")?)?;
            assert_eq!(truncated.nodes.len(), 3);
            assert!(truncated.truncated);
            let mut data = crate::data::Data::default();
            data.set_graph(truncated, None);
            assert!(
                data.graph_status()
                    .contains("neighborhood truncated by query limits")
            );
            worker.expand(x)?;
            let complete = graph(&worker).await?;
            assert_edges(&complete, edges.get(3..).ok_or("edges missing")?)?;
            assert!(!complete.truncated);
            data.set_graph(complete, None);
            assert!(
                !data
                    .graph_status()
                    .contains("neighborhood truncated by query limits")
            );
            Ok(())
        }
        .await;
        drop(worker);
        let cleanup = native.delete().await;
        result?;
        cleanup
    })
}

#[test]
fn given_real_query_failure_when_storage_is_repaired_then_worker_clears_error_and_publishes_new_snapshot()
-> TestResult {
    supervised_async(|| async {
        let fixture = Fixture::new().await?;
        let before = fixture
            .commit("before", ACCOUNT, REGION, &[("shared", "initial")], &[], 1)
            .await?;
        let worker = fixture.worker(None);
        let (_, rows, total) = snapshot(&worker).await?;
        assert_eq!(rows, before);
        assert_eq!(total, 1);
        let writer = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&fixture.path))
            .await?;
        sqlx::query("UPDATE resources SET content_hash=zeroblob(1) WHERE account=? AND region=? AND kind=? AND id=?")
        .bind(ACCOUNT).bind(REGION).bind(KIND).bind("shared").execute(&writer).await?;
        worker.refresh(Filter::default())?;
        match fixtures::await_event(&worker, true).await? {
            WorkerEvent::Snapshot {
                counts,
                rows,
                total,
                error,
            } => {
                assert_eq!(error, Some("invalid stored resource row".to_owned()));
                assert_eq!(counts, Counts::default());
                assert_eq!(rows, Vec::<Row>::new());
                assert_eq!(total, 0);
            }
            other => return Err(format!("expected failed real snapshot, got {other:?}").into()),
        }
        sqlx::query("UPDATE resources SET content_hash=(SELECT detail.content_hash FROM thing_records AS detail WHERE detail.account=resources.account AND detail.region=resources.region AND detail.id=resources.id) WHERE kind=?")
        .bind(KIND).execute(&writer).await?;
        let mut expected = fixture
            .commit(
                "repair",
                ACCOUNT,
                REGION,
                &[("shared", "recovered"), ("added", "new-count")],
                &[],
                2,
            )
            .await?;
        sort_rows(&mut expected);
        worker.refresh(Filter::default())?;
        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!(
            counts,
            Counts {
                accounts: 1,
                regions: 1,
                services: 1,
                resources: 2
            }
        );
        assert_eq!(rows, expected);
        assert_eq!(total, 2);
        drop(worker);
        writer.close().await;
        Ok(())
    })
}

#[test]
fn given_hostile_text_and_scope_filters_when_actual_worker_refreshes_then_rows_and_counts_match_public_sql_contract()
-> TestResult {
    supervised_async(|| async {
        let fixture = Fixture::new().await?;
        let expected = fixture
            .commit(
                "filter",
                ACCOUNT,
                REGION,
                &[
                    ("shared", "needle ' OR 1=1 --"),
                    ("ordinary", "ordinary"),
                    ("wildcard", "%_tail"),
                ],
                &[],
                1,
            )
            .await?;
        let worker = fixture.worker(None);
        let (counts, _, total) = snapshot(&worker).await?;
        assert_eq!(counts.resources, 3);
        assert_eq!(total, 3);
        let database =
            Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
        for (query, expected_id) in [("' OR 1=1 --", "shared"), ("%_tail", "wildcard")] {
            worker.refresh(Filter {
                query: query.into(),
                account: Some(ACCOUNT.into()),
                region: Some(REGION.into()),
                kind: Some(KIND.into()),
                ..Filter::default()
            })?;
            let (counts, rows, total) = snapshot(&worker).await?;
            let sql = database
                .page(
                    &ResourceFilter {
                        text: Some(query.into()),
                        account: Some(ACCOUNT.into()),
                        region: Some(REGION.into()),
                        kind: Some(KIND.into()),
                        ..ResourceFilter::default()
                    },
                    PageRequest {
                        limit: PAGE_LIMIT,
                        after: None,
                    },
                )
                .await?;
            assert_eq!(total, 1);
            assert_eq!(counts.resources, 3);
            assert_eq!(
                rows,
                expected
                    .iter()
                    .filter(|row| row.id == expected_id)
                    .cloned()
                    .collect::<Vec<_>>()
            );
            assert_eq!(sql.total_matching, total);
            assert_eq!(
                sql.resources
                    .iter()
                    .map(|resource| resource.key.id.as_str())
                    .collect::<Vec<_>>(),
                vec![expected_id]
            );
        }
        worker.refresh(Filter {
            account: Some("222222222222".into()),
            ..Filter::default()
        })?;
        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!(rows, Vec::<Row>::new());
        assert_eq!(total, 0);
        assert_eq!(counts.resources, 3);
        drop(worker);
        Ok(())
    })
}
