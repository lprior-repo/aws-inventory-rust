use super::*;
use crate::worker::DEFAULT_GRAPH_LIMITS;

fn graph_status(app: &App, cx: &mut Cx) -> String {
    app.table(cx)
        .borrow::<ResourceTable>()
        .map_or_else(String::new, |table| table.graph_status())
}

#[test]
fn given_successful_empty_database_when_startup_page_commits_then_graph_access_diagnostic_survives()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        for (url, name, diagnostic) in [
            (None, None, "not configured (--graph-url and --graph-name)"),
            (
                Some("redis://127.0.0.1:6379"),
                None,
                "configure both --graph-url and --graph-name",
            ),
            (
                Some("redis://127.0.0.1:6379"),
                Some(""),
                "invalid graph configuration · graph name",
            ),
        ] {
            let (mut cx, mut app) = app()?;
            app.start_worker_with_config(
                &mut cx,
                WorkerConfig {
                    database: Some(fixture.path.to_string_lossy().into_owned()),
                    graph_url: url.map(str::to_owned),
                    graph_name: name.map(str::to_owned),
                    graph_limits: DEFAULT_GRAPH_LIMITS,
                    ..WorkerConfig::default()
                },
            );
            let expected = format!("Graph offline · {diagnostic}");
            wait_for(&mut app, &mut cx, |app, cx| {
                status(app, cx) == "Database opened · no resources yet"
                    && graph_status(app, cx) == expected
            })?;
            assert_eq!(selected(&app, &mut cx), None);
            assert_eq!(graph(&app, &mut cx), Graph::default());
            app.handle_event(&mut cx, &Event::Signal);
            assert_eq!(graph_status(&app, &mut cx), expected);
            shutdown(&mut app, &mut cx);
        }
        Ok(())
    })
}

#[test]
fn given_failed_real_database_when_repaired_and_filter_action_retries_then_closed_worker_is_replaced_at_same_path()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        let expected = runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "recovered")],
            &[],
            1,
        ))?;
        let pool = runtime.block_on(
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&fixture.path)),
        )?;
        runtime.block_on(sqlx::query("PRAGMA user_version=0").execute(&pool))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(
            &mut cx,
            WorkerConfig {
                database: Some(fixture.path.to_string_lossy().into_owned()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
        );
        let failure = "Database error · unsupported schema version 0; expected 15, 16, or 17 (open with the writer to migrate schema 14/15/16)";
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == failure && app.worker.as_ref().is_some_and(DataWorker::is_closed)
        })?;
        assert!(app.retry_worker);
        assert_eq!(selected(&app, &mut cx), None);
        runtime.block_on(sqlx::query("PRAGMA user_version=17").execute(&pool))?;
        text_action(&mut app, &mut cx, ids!(search_input), "recovered");
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "1 matching · 1 loaded"
        })?;
        assert_eq!(selected(&app, &mut cx), expected.first().cloned());
        assert_eq!(app.filter(&mut cx).query, "recovered");
        assert!(!app.retry_worker);
        assert!(
            app.worker
                .as_ref()
                .is_some_and(|worker| !worker.is_closed())
        );
        assert_eq!(
            app.worker_config
                .as_ref()
                .and_then(|config| config.database.as_deref()),
            Some(fixture.path.to_string_lossy().as_ref())
        );
        assert!(!graph_status(&app, &mut cx).contains("database unavailable"));
        shutdown(&mut app, &mut cx);
        runtime.block_on(pool.close());
        Ok(())
    })
}

#[test]
fn given_failed_resource_query_when_filter_removes_selection_then_new_empty_page_does_not_restore_query_failure()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "root"), ("b", "neighbor")],
            &[],
            1,
        ))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(
            &mut cx,
            WorkerConfig {
                database: Some(fixture.path.to_string_lossy().into_owned()),
                graph_url: Some(native.endpoint.clone()),
                graph_name: Some(native.name.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
        );
        wait_for(&mut app, &mut cx, |app, cx| graph(app, cx).nodes.len() == 2)?;
        assert_eq!(app.graph_access_error, None);
        runtime.block_on(native.query(inventory_graph::cypher::diagnostics::CORRUPT_ACCOUNT))?;
        let root = fixtures::digest(&fixtures::root())?;
        graph_action(&mut app, &mut cx, &root);
        wait_for(&mut app, &mut cx, |app, cx| {
            graph_status(app, cx).starts_with("Graph offline · ")
        })?;
        assert_eq!(
            app.graph_access_error, None,
            "resource-query failures are not connection status"
        );
        text_action(
            &mut app,
            &mut cx,
            ids!(search_input),
            "not-present-in-inventory",
        );
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "0 matching · 0 loaded"
        })?;
        assert_eq!(selected(&app, &mut cx), None);
        assert_eq!(
            graph_status(&app, &mut cx),
            "Select a resource to expand its graph neighbors"
        );
        shutdown(&mut app, &mut cx);
        runtime.block_on(native.delete())?;
        Ok(())
    })
}

#[test]
fn given_cached_access_failure_and_current_selection_when_stale_success_arrives_then_rejected_graph_cannot_clear_access_status()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "current")],
            &[],
            1,
        ))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(
            &mut cx,
            WorkerConfig {
                database: Some(fixture.path.to_string_lossy().into_owned()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
        );
        let access = "not configured (--graph-url and --graph-name)";
        let offline = format!("Graph offline · {access}");
        wait_for(&mut app, &mut cx, |app, cx| {
            selected(app, cx).is_some() && graph_status(app, cx) == offline
        })?;
        let previous = graph(&app, &mut cx);
        let selection = selected(&app, &mut cx);
        app.apply_worker_event(
            &mut cx,
            WorkerEvent::Graph {
                graph: Graph {
                    tag_root: Some("different-resource-root".into()),
                    ..Graph::default()
                },
                error: None,
            },
        );
        assert_eq!(graph(&app, &mut cx), previous);
        assert_eq!(selected(&app, &mut cx), selection);
        assert_eq!(app.graph_access_error.as_deref(), Some(access));
        assert_eq!(graph_status(&app, &mut cx), offline);
        app.apply_worker_event(
            &mut cx,
            WorkerEvent::Snapshot {
                counts: crate::data::Counts {
                    resources: 1,
                    ..crate::data::Counts::default()
                },
                rows: vec![],
                total: 0,
                error: None,
            },
        );
        assert_eq!(selected(&app, &mut cx), None);
        assert_eq!(status(&app, &mut cx), "0 matching · 0 loaded");
        assert_eq!(app.graph_access_error.as_deref(), Some(access));
        assert_eq!(graph_status(&app, &mut cx), offline);
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}

fn online_app(fixture: &Fixture, native: &NativeGraph, endpoint: &str) -> TestResult<(Cx, App)> {
    let (mut cx, mut app) = app()?;
    app.start_worker_with_config(
        &mut cx,
        WorkerConfig {
            database: Some(fixture.path.to_string_lossy().into_owned()),
            graph_url: Some(endpoint.to_owned()),
            graph_name: Some(native.name.clone()),
            graph_limits: DEFAULT_GRAPH_LIMITS,
            ..WorkerConfig::default()
        },
    );
    wait_for(&mut app, &mut cx, |app, cx| graph(app, cx).nodes.len() == 2)?;
    assert_eq!(app.graph_access_error, None);
    Ok((cx, app))
}

#[test]
fn given_lost_graph_endpoint_when_snapshot_follows_failure_then_offline_survives_until_restored_refresh()
-> TestResult {
    supervised(|| {
        use crate::worker::tests::support::transport::{FaultableEndpoint, refresh_batch};
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root")], &[], 1))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "native-neighbor");
        let link = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let endpoint = FaultableEndpoint::start(&native.endpoint)?;
        let (mut cx, mut app) = online_app(&fixture, &native, &endpoint.endpoint)?;
        let online = graph_status(&app, &mut cx);
        endpoint.set_online(false);
        graph_action(&mut app, &mut cx, &fixtures::digest(&fixtures::root())?);
        wait_for(&mut app, &mut cx, |app, cx| {
            graph_status(app, cx).starts_with("Graph offline · connection/query unavailable · ")
        })?;
        let offline = graph_status(&app, &mut cx);
        let worker = app.worker.as_ref().ok_or("missing graph worker")?;
        worker.refresh(crate::data::Filter {
            query: "absent".to_owned(),
            ..crate::data::Filter::default()
        })?;
        let batch = refresh_batch(worker)?;
        let terminal = batch
            .iter()
            .filter(|event| !matches!(event, WorkerEvent::GraphStatus { .. }));
        assert_eq!(
            terminal
                .map(|event| matches!(event, WorkerEvent::Snapshot { .. }))
                .collect::<Vec<_>>(),
            vec![false, true]
        );
        for event in batch {
            app.apply_worker_event(&mut cx, event);
        }
        assert_eq!(selected(&app, &mut cx), None);
        assert_eq!(graph_status(&app, &mut cx), offline);
        endpoint.set_online(true);
        app.worker
            .as_ref()
            .ok_or("missing graph worker")?
            .refresh(crate::data::Filter::default())?;
        wait_for(&mut app, &mut cx, |app, cx| {
            app.graph_access_error.is_none() && graph_status(app, cx) == online
        })?;
        assert_eq!(graph_status(&app, &mut cx), online);
        assert_eq!(app.graph_access_error, None);
        shutdown(&mut app, &mut cx);
        endpoint.finish()?;
        runtime.block_on(native.delete())?;
        Ok(())
    })
}

#[test]
fn given_restored_graph_when_refresh_snapshot_removes_selection_then_success_is_not_fenced()
-> TestResult {
    supervised(|| {
        use crate::worker::tests::support::transport::{
            FaultableEndpoint, first_refresh_snapshot, refresh_graph,
        };
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root")], &[], 1))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "native-neighbor");
        let link = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let endpoint = FaultableEndpoint::start(&native.endpoint)?;
        let (mut cx, mut app) = online_app(&fixture, &native, &endpoint.endpoint)?;
        endpoint.set_online(false);
        graph_action(&mut app, &mut cx, &fixtures::digest(&fixtures::root())?);
        wait_for(&mut app, &mut cx, |app, cx| {
            graph_status(app, cx).starts_with("Graph offline · connection/query unavailable · ")
        })?;
        endpoint.set_online(true);
        let worker = app.worker.as_ref().ok_or("missing graph worker")?;
        worker.refresh(crate::data::Filter {
            query: "absent".to_owned(),
            ..crate::data::Filter::default()
        })?;
        let snapshot = first_refresh_snapshot(worker)?;
        for event in snapshot {
            app.apply_worker_event(&mut cx, event);
        }
        assert_eq!(selected(&app, &mut cx), None);
        let worker = app.worker.as_ref().ok_or("missing graph worker")?;
        for event in refresh_graph(worker, None)? {
            app.apply_worker_event(&mut cx, event);
        }
        assert_eq!(app.graph_access_error, None);
        assert_eq!(
            graph_status(&app, &mut cx),
            "Select a resource to expand its graph neighbors"
        );
        shutdown(&mut app, &mut cx);
        endpoint.finish()?;
        runtime.block_on(native.delete())?;
        Ok(())
    })
}

#[test]
fn given_invalid_native_resource_when_refresh_removes_selection_then_query_failure_stays_scoped()
-> TestResult {
    supervised(|| {
        use crate::worker::tests::support::transport::{first_refresh_snapshot, refresh_graph};
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root")], &[], 1))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "native-neighbor");
        let link = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let (mut cx, mut app) = online_app(&fixture, &native, &native.endpoint)?;
        runtime.block_on(native.query("MATCH (n:Resource) SET n.account='invalid'"))?;
        let worker = app.worker.as_ref().ok_or("missing graph worker")?;
        worker.refresh(crate::data::Filter {
            query: "absent".to_owned(),
            ..crate::data::Filter::default()
        })?;
        for event in first_refresh_snapshot(worker)? {
            app.apply_worker_event(&mut cx, event);
        }
        assert_eq!(selected(&app, &mut cx), None);
        let worker = app.worker.as_ref().ok_or("missing graph worker")?;
        for event in refresh_graph(worker, Some("invalid graph response"))? {
            app.apply_worker_event(&mut cx, event);
        }
        assert_eq!(app.graph_access_error, None);
        assert_eq!(
            graph_status(&app, &mut cx),
            "Select a resource to expand its graph neighbors"
        );
        shutdown(&mut app, &mut cx);
        runtime.block_on(native.delete())?;
        Ok(())
    })
}
