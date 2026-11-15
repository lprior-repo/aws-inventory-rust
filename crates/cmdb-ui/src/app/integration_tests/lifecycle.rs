use super::*;

fn graph_status(app: &App, cx: &mut Cx) -> String {
    app.table(cx).borrow::<ResourceTable>().map_or_else(String::new, |table| table.graph_status())
}

#[test]
fn given_successful_empty_database_when_startup_page_commits_then_graph_access_diagnostic_survives() -> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        for (url, name, diagnostic) in [
            (None, None, "not configured (--graph-url and --graph-name)"),
            (Some("redis://127.0.0.1:6379"), None, "configure both --graph-url and --graph-name"),
            (Some("redis://127.0.0.1:6379"), Some(""), "invalid graph configuration · graph name"),
        ] {
            let (mut cx, mut app) = app()?;
            app.start_worker_with_config(&mut cx, WorkerConfig { database: Some(fixture.path.clone()),
                graph_url: url.map(str::to_owned), graph_name: name.map(str::to_owned), argument_error: None });
            let expected = format!("Graph offline · {diagnostic}");
            wait_for(&mut app, &mut cx, |app, cx|
                status(app, cx) == "Database opened · no resources yet" && graph_status(app, cx) == expected)?;
            assert_eq!(selected(&app, &mut cx), None);
            assert_eq!(graph(&app, &mut cx), Graph::default());
            // A later ordinary drain must not erase the persistent access state.
            app.handle_event(&mut cx, &Event::Signal);
            assert_eq!(graph_status(&app, &mut cx), expected);
            shutdown(&mut app, &mut cx);
        }
        Ok(())
    })
}

#[test]
fn given_failed_real_database_when_repaired_and_filter_action_retries_then_closed_worker_is_replaced_at_same_path() -> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        let expected = runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "recovered")], &[], 1))?;
        let pool = runtime.block_on(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&fixture.path)))?;
        runtime.block_on(sqlx::query("PRAGMA user_version=0").execute(&pool))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(&mut cx, WorkerConfig { database: Some(fixture.path.clone()), ..WorkerConfig::default() });
        let failure = "Database error · unsupported schema version 0; expected 15 or 16 (open with the writer to migrate schema 14)";
        wait_for(&mut app, &mut cx, |app, cx| status(app, cx) == failure
            && app.worker.as_ref().is_some_and(DataWorker::is_closed))?;
        assert!(app.retry_worker);
        assert_eq!(selected(&app, &mut cx), None);
        // Repair the actual file, without invoking any App lifecycle method.
        runtime.block_on(sqlx::query("PRAGMA user_version=16").execute(&pool))?;
        text_action(&mut app, &mut cx, ids!(search_input), "recovered");
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        wait_for(&mut app, &mut cx, |app, cx| status(app, cx) == "1 matching · 1 loaded")?;
        assert_eq!(selected(&app, &mut cx), expected.first().cloned());
        assert_eq!(app.filter(&mut cx).query, "recovered");
        assert!(!app.retry_worker);
        assert!(app.worker.as_ref().is_some_and(|worker| !worker.is_closed()));
        assert_eq!(app.worker_config.as_ref().and_then(|config| config.database.as_ref()), Some(&fixture.path));
        assert!(!graph_status(&app, &mut cx).contains("database unavailable"));
        shutdown(&mut app, &mut cx);
        runtime.block_on(pool.close());
        Ok(())
    })
}

#[test]
fn given_failed_resource_query_when_filter_removes_selection_then_new_empty_page_does_not_restore_query_failure() -> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root"), ("b", "neighbor")], &[], 1))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(&mut cx, WorkerConfig { database: Some(fixture.path.clone()),
            graph_url: Some(native.endpoint.clone()), graph_name: Some(native.name.clone()), argument_error: None });
        wait_for(&mut app, &mut cx, |app, cx| graph(app, cx).nodes.len() == 2)?;
        assert_eq!(app.graph_access_error, None);
        // Corrupt real native identity metadata, not a synthetic WorkerEvent.
        runtime.block_on(native.query("MATCH (n:Resource) SET n.account='invalid'"))?;
        let root = fixtures::digest(&fixtures::root())?;
        graph_action(&mut app, &mut cx, &root);
        wait_for(&mut app, &mut cx, |app, cx| graph_status(app, cx).starts_with("Graph offline · "))?;
        assert_eq!(app.graph_access_error, None, "resource-query failures are not connection status");
        text_action(&mut app, &mut cx, ids!(search_input), "not-present-in-inventory");
        wait_for(&mut app, &mut cx, |app, cx| status(app, cx) == "0 matching · 0 loaded")?;
        assert_eq!(selected(&app, &mut cx), None);
        assert_eq!(graph_status(&app, &mut cx), "Select a resource to expand its graph neighbors");
        shutdown(&mut app, &mut cx);
        runtime.block_on(native.delete())?;
        Ok(())
    })
}

#[test]
fn given_cached_access_failure_and_current_selection_when_stale_success_arrives_then_rejected_graph_cannot_clear_access_status() -> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "current")], &[], 1))?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(&mut cx, WorkerConfig { database: Some(fixture.path.clone()), ..WorkerConfig::default() });
        let access = "not configured (--graph-url and --graph-name)";
        let offline = format!("Graph offline · {access}");
        wait_for(&mut app, &mut cx, |app, cx| selected(app, cx).is_some() && graph_status(app, cx) == offline)?;
        let previous = graph(&app, &mut cx);
        let selection = selected(&app, &mut cx);
        // Exercise the defensive App consumer directly: even a successful
        // output must not clear access state if the table rejects its root.
        app.apply_worker_event(&mut cx, WorkerEvent::Graph {
            graph: Graph { tag_root: Some("different-resource-root".into()), ..Graph::default() },
            error: None,
        });
        assert_eq!(graph(&app, &mut cx), previous);
        assert_eq!(selected(&app, &mut cx), selection);
        assert_eq!(app.graph_access_error.as_deref(), Some(access));
        assert_eq!(graph_status(&app, &mut cx), offline);
        app.apply_worker_event(&mut cx, WorkerEvent::Snapshot {
            counts: crate::data::Counts { resources: 1, ..crate::data::Counts::default() },
            rows: vec![], total: 0, error: None,
        });
        assert_eq!(selected(&app, &mut cx), None);
        assert_eq!(status(&app, &mut cx), "0 matching · 0 loaded");
        assert_eq!(app.graph_access_error.as_deref(), Some(access));
        assert_eq!(graph_status(&app, &mut cx), offline);
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}
