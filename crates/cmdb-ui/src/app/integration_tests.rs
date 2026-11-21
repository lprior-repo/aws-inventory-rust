//! Headless native widget construction plus real App actions and DataWorker I/O.
//! Rendering/hit testing remains the separate native screenshot acceptance gate.
use super::super::*;
use crate::data::{Graph, Row};
use crate::worker::DEFAULT_GRAPH_LIMITS;
use crate::worker::tests::isolation::supervised;
use crate::worker::tests::real::fixtures::{
    self, ACCOUNT, Fixture, KIND, NativeGraph, REGION, TestResult,
};
use crate::worker::tests::support::{self, BlockedGraph, QueuedWorker};
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};
use std::time::{Duration, Instant};

#[path = "integration_tests/lifecycle.rs"]
mod lifecycle;

fn runtime() -> TestResult<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?)
}

fn app() -> TestResult<(Cx, App)> {
    let mut cx = Cx::new(Box::new(|_, _| {}));
    let app = cx.with_vm(|vm| {
        vm.capture_errors();
        let value = <App as AppMain>::script_mod(vm);
        let mut app = App::script_from_value(vm, value);
        <App as AppMain>::after_new_from_script(vm, &mut app);
        app
    });
    let errors = cx.with_vm(|vm| vm.take_errors());
    assert!(
        errors.is_empty(),
        "real application widget tree: {errors:?}"
    );
    assert!(app.table(&mut cx).borrow::<ResourceTable>().is_some());
    Ok((cx, app))
}

fn status(app: &App, cx: &mut Cx) -> String {
    app.table(cx)
        .borrow::<ResourceTable>()
        .map_or_else(String::new, |table| table.status_line())
}

fn selected(app: &App, cx: &mut Cx) -> Option<Row> {
    app.table(cx)
        .borrow::<ResourceTable>()
        .and_then(|table| table.selected_row())
}

fn graph(app: &App, cx: &mut Cx) -> Graph {
    app.table(cx)
        .borrow::<ResourceTable>()
        .map_or_else(Graph::default, |table| table.graph())
}

fn wait_for(app: &mut App, cx: &mut Cx, predicate: impl Fn(&App, &mut Cx) -> bool) -> TestResult {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .ok_or("deadline overflow")?;
    for _ in 0..20_000_000 {
        app.handle_event(cx, &Event::Signal);
        if predicate(app, cx) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::yield_now();
    }
    Err(format!("App completion deadline: {}", status(app, cx)).into())
}

fn text_action(app: &mut App, cx: &mut Cx, path: &[LiveId], value: &str) {
    let input = app.ui.text_input(cx, path);
    input.set_text(cx, value);
    let actions = cx.capture_actions(|cx| {
        cx.widget_action(input.widget_uid(), TextInputAction::Changed(value.into()))
    });
    app.handle_event(cx, &Event::Actions(actions));
}

fn exact_action(app: &mut App, cx: &mut Cx, active: bool) {
    let input = app.ui.check_box(cx, ids!(tag_value_exact));
    input.set_active(cx, active, Animate::No);
    let actions = cx
        .capture_actions(|cx| cx.widget_action(input.widget_uid(), CheckBoxAction::Change(active)));
    app.handle_event(cx, &Event::Actions(actions));
}

fn graph_action(app: &mut App, cx: &mut Cx, key: &str) {
    let uid = app.ui.widget(cx, ids!(dag)).widget_uid();
    let actions = cx.capture_actions(|cx| {
        cx.widget_action(
            uid,
            crate::dag::widget::DagSelectAction {
                id: Some(key.into()),
            },
        )
    });
    app.handle_event(cx, &Event::Actions(actions));
}

fn shutdown(app: &mut App, cx: &mut Cx) {
    app.handle_event(cx, &Event::Shutdown);
    assert!(app.worker.is_none());
    assert!(app.completion_poll.is_empty());
    assert!(app.shutting_down);
}

#[test]
fn given_tag_input_actions_when_real_app_filters_then_exact_empty_and_key_only_have_distinct_results()
-> TestResult {
    supervised(|| {
        // Given real committed tags and the complete production widget tree.
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        let tagged = |id, value| -> TestResult<TagObservation> {
            Ok(TagObservation::new(
                fixtures::key(&fixtures::node(ACCOUNT, REGION, KIND, id))?,
                TagCoverage::Complete,
                vec![Tag::new("owner", value)?],
            )?)
        };
        let expected = runtime.block_on(fixture.commit_tagged(
            "seed",
            (ACCOUNT, REGION),
            &[
                ("shared", "empty-owner"),
                ("b", "nonempty-owner"),
                ("c", "untagged"),
            ],
            &[],
            vec![tagged("shared", "")?, tagged("b", "team")?],
            1,
        ))?;
        let (mut cx, mut app) = app()?;
        app.worker = Some(fixture.worker(None));
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "3 matching · 3 loaded"
        })?;
        // When key and checkbox actions pass through AppMain/handle_tag_filter.
        text_action(&mut app, &mut cx, ids!(tag_key_input), "owner");
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        assert_eq!(
            app.filter(&mut cx).tag,
            Some(cmdb_view::TagFilter {
                key: "owner".into(),
                value: None
            })
        );
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "2 matching · 2 loaded"
        })?;
        exact_action(&mut app, &mut cx, true);
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "1 matching · 1 loaded"
        })?;
        // Then both stored metadata and visible control state retain exact empty.
        assert_eq!(
            app.filter(&mut cx).tag,
            Some(cmdb_view::TagFilter {
                key: "owner".into(),
                value: Some(String::new())
            })
        );
        assert_eq!(
            selected(&app, &mut cx),
            expected.iter().find(|row| row.id == "shared").cloned()
        );
        assert_eq!(app.ui.text_input(&cx, ids!(tag_key_input)).text(), "owner");
        assert_eq!(app.ui.text_input(&cx, ids!(tag_value_input)).text(), "");
        assert!(app.ui.check_box(&cx, ids!(tag_value_exact)).active(&cx));
        assert_eq!(
            app.ui.label(&cx, ids!(table_status)).text(),
            "1 matching · 1 loaded"
        );
        text_action(&mut app, &mut cx, ids!(tag_value_input), "team");
        wait_for(&mut app, &mut cx, |app, cx| {
            selected(app, cx).is_some_and(|row| row.id == "b")
                && status(app, cx) == "1 matching · 1 loaded"
        })?;
        assert_eq!(
            selected(&app, &mut cx),
            expected.iter().find(|row| row.id == "b").cloned()
        );
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}

#[test]
fn given_real_native_graph_when_app_receives_resource_action_then_selected_root_and_metadata_change()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        let expected = runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "root"), ("b", "neighbor")],
            &[],
            1,
        ))?;
        let neighbor = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let edge = fixtures::edge(&fixtures::root(), &neighbor, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&edge)))?;
        let result: TestResult = (|| {
            // Given an initialized App consuming its real worker's native graph.
            let (mut cx, mut app) = app()?;
            app.worker = Some(fixture.worker(Some(&native)));
            wait_for(&mut app, &mut cx, |app, cx| graph(app, cx).edges.len() == 1)?;
            let root_key = fixtures::digest(&fixtures::root())?;
            // When the actual graph action crosses App's enqueue boundary.
            graph_action(&mut app, &mut cx, &root_key);
            wait_for(&mut app, &mut cx, |app, cx| {
                graph(app, cx).tag_root.as_deref() == Some(root_key.as_str())
            })?;
            assert_eq!(
                selected(&app, &mut cx),
                expected.iter().find(|row| row.id == "shared").cloned()
            );
            let neighbor_key = fixtures::digest(&neighbor)?;
            graph_action(&mut app, &mut cx, &neighbor_key);
            wait_for(&mut app, &mut cx, |app, cx| {
                graph(app, cx).tag_root.as_deref() == Some(neighbor_key.as_str())
            })?;
            // Then the real expansion, native relationship, and SQL row agree.
            assert_eq!(
                selected(&app, &mut cx),
                expected.iter().find(|row| row.id == "b").cloned()
            );
            fixtures::assert_edges(&graph(&app, &mut cx), &[edge])?;
            shutdown(&mut app, &mut cx);
            Ok(())
        })();
        let cleanup = runtime.block_on(native.delete());
        result?;
        cleanup
    })
}

#[test]
fn given_saturated_real_intake_when_app_rejects_actions_then_inputs_selection_and_accepted_output_survive()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        let expected = runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "accepted-root"), ("b", "accepted-neighbor")],
            &[],
            1,
        ))?;
        let (mut cx, mut app) = app()?;
        app.worker = Some(fixture.worker(None));
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "2 matching · 2 loaded"
        })?;
        let previous = selected(&app, &mut cx).ok_or("initial selection missing")?;
        let previous_graph = graph(&app, &mut cx);
        let QueuedWorker { worker, start } = support::queued(
            WorkerConfig {
                database: Some(fixture.path.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
            1,
            64,
        );
        app.worker = Some(worker);
        // Given one real accepted filter occupies the entire command budget.
        text_action(&mut app, &mut cx, ids!(search_input), "accepted");
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        let accepted = app.filter(&mut cx);
        // When the real filter and selection calls are rejected, inputs roll back.
        text_action(&mut app, &mut cx, ids!(tag_key_input), "rejected-key");
        assert_eq!(app.filter(&mut cx), accepted);
        assert_eq!(app.ui.text_input(&cx, ids!(tag_key_input)).text(), "");
        assert_eq!(
            app.ui.text_input(&cx, ids!(search_input)).text(),
            "accepted"
        );
        assert!(!app.ui.check_box(&cx, ids!(tag_value_exact)).active(&cx));
        assert_eq!(
            status(&app, &mut cx),
            "Database error · inventory worker queue is full"
        );
        let rejected = expected
            .iter()
            .find(|row| row.key != previous.key)
            .ok_or("second row missing")?;
        graph_action(&mut app, &mut cx, &rejected.key);
        assert_eq!(selected(&app, &mut cx), Some(previous));
        assert_eq!(graph(&app, &mut cx), previous_graph);
        // Then releasing the real runner delivers the still-accepted revision.
        start.send(())?;
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "2 matching · 2 loaded"
        })?;
        assert_eq!(app.filter(&mut cx), accepted);
        assert_eq!(
            app.table(&mut cx)
                .borrow::<ResourceTable>()
                .map(|table| table.visible_len()),
            Some(2)
        );
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}

#[test]
fn given_failed_startup_with_accepted_filter_when_app_drains_then_unrevisioned_error_is_terminal()
-> TestResult {
    supervised(|| {
        let (mut cx, mut app) = app()?;
        let QueuedWorker { worker, start } = support::queued(WorkerConfig::default(), 2, 64);
        app.worker = Some(worker);
        // Given an actual admitted action before startup failure is produced.
        text_action(&mut app, &mut cx, ids!(search_input), "recovered");
        assert_eq!(status(&app, &mut cx), "Loading inventory…");
        start.send(())?;
        // When unrevisioned real producer errors reach App's normal drain path.
        wait_for(&mut app, &mut cx, |app, cx| {
            status(app, cx) == "Database error · missing --database PATH"
        })?;
        assert!(app.retry_worker);
        assert_eq!(selected(&app, &mut cx), None);
        assert!(graph(&app, &mut cx).nodes.is_empty());
        assert_eq!(
            app.table(&mut cx)
                .borrow::<ResourceTable>()
                .map(|table| table.graph_status()),
            Some("Graph offline · database unavailable · missing --database PATH".into())
        );
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}

#[test]
fn given_live_app_blocked_query_when_shutdown_event_arrives_then_worker_joins_and_timer_stops_without_app_drop()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root")], &[], 1))?;
        let peer = BlockedGraph::start()?;
        let (mut cx, mut app) = app()?;
        app.start_worker_with_config(&mut cx, peer.config(fixture.path.clone()));
        assert!(!app.completion_poll.is_empty());
        wait_for(&mut app, &mut cx, |app, cx| selected(app, cx).is_some())?;
        peer.await_query()?;
        // When shutdown dispatch occurs while App and Cx remain strongly alive.
        let started = Instant::now();
        shutdown(&mut app, &mut cx);
        assert!(started.elapsed() < Duration::from_secs(2));
        peer.finish()?;
        // Then delayed completion/actions/startup cannot resurrect its owners.
        app.handle_event(&mut cx, &Event::Signal);
        app.handle_event(&mut cx, &Event::Startup);
        text_action(&mut app, &mut cx, ids!(search_input), "after close");
        assert!(app.worker.is_none());
        assert!(app.completion_poll.is_empty());
        shutdown(&mut app, &mut cx);
        Ok(())
    })
}

#[test]
fn given_live_app_with_blocked_output_when_shutdown_event_arrives_then_timer_and_real_worker_close()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit("seed", ACCOUNT, REGION, &[("shared", "root")], &[], 1))?;
        let (mut cx, mut app) = app()?;
        let worker = support::queued(
            WorkerConfig {
                database: Some(fixture.path.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
            1,
            0,
        )
        .release()?;
        // The real runner has delivered its graph and cannot deliver its
        // following snapshot through this unconsumed rendezvous receiver.
        support::receive_startup_graph(&worker)?;
        app.worker = Some(worker);
        app.completion_poll = cx.start_interval(0.05);
        assert!(!app.completion_poll.is_empty());
        let started = Instant::now();
        shutdown(&mut app, &mut cx);
        assert!(started.elapsed() < Duration::from_secs(2));
        app.handle_event(&mut cx, &Event::Signal);
        app.handle_event(&mut cx, &Event::Startup);
        assert!(app.worker.is_none());
        assert!(app.completion_poll.is_empty());
        Ok(())
    })
}
