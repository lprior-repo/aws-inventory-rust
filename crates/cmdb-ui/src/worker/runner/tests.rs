use super::*;
use crate::worker::tests::isolation::supervised;
use crate::worker::tests::real::fixtures::{
    self, ACCOUNT, Fixture, KIND, NativeGraph, REGION, TestResult, assert_edges, edge, root,
};

fn runtime() -> TestResult<tokio::runtime::Runtime> {
    Ok(Builder::new_current_thread().enable_all().build()?)
}

fn context(capacity: usize) -> (DataWorker, WorkerContext) {
    let (commands, receiver) = tokio::sync::mpsc::channel(capacity);
    let (events, outputs) = mpsc::sync_channel(EVENT_CAPACITY);
    let revisions = Arc::new(Revisions::default());
    let (shutdown, cancellation) = shutdown_channel();
    let context = WorkerContext::new(
        receiver,
        events,
        SignalToUI::default(),
        cancellation,
        Arc::clone(&revisions),
    );
    let worker = DataWorker {
        commands,
        events: Some(outputs),
        revisions,
        shutdown,
        thread: None,
    };
    (worker, context)
}

fn config(fixture: &Fixture, native: &NativeGraph) -> WorkerConfig {
    WorkerConfig {
        database: Some(fixture.path.clone()),
        graph_url: Some(native.endpoint.clone()),
        graph_name: Some(native.name.clone()),
    }
}

fn outputs(worker: &DataWorker) -> TestResult<Vec<Output>> {
    Ok(worker
        .events
        .as_ref()
        .ok_or("worker receiver missing")?
        .try_iter()
        .collect())
}

#[test]
fn given_superseded_refresh_after_admission_when_runner_completes_then_next_expansion_uses_last_committed_session()
-> TestResult {
    supervised(|| {
        // Enter the real process boundary after admission, then accept a newer
        // refresh before completion. No outputs or QuerySession values are injected.
        let runtime = runtime()?;
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        let a = root();
        let b = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = edge(&a, &b, "references", "model");
        let before = runtime.block_on(fixture.commit(
            "before",
            ACCOUNT,
            REGION,
            &[("shared", "before-root"), ("b", "before-neighbor")],
            &[],
            1,
        ))?;
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let result: TestResult = (|| {
            let (mut state, _) =
                runtime.block_on(setup(fixture.path.clone(), config(&fixture, &native)))?;
            runtime.block_on(fixture.commit(
                "after",
                ACCOUNT,
                REGION,
                &[
                    ("shared", "after-root-longer"),
                    ("b", "after-neighbor-longer"),
                    ("extra", "after-only"),
                ],
                &[],
                2,
            ))?;
            let (worker, mut runner) = context(1);
            worker.refresh(Filter::default())?;
            let command = runner.commands.try_recv()?;
            worker.refresh(Filter {
                query: "newer-accepted".into(),
                ..Filter::default()
            })?;
            assert!(matches!(command, Command::Refresh { .. }));
            assert!(runner.process(command, &runtime, &mut state));
            assert_eq!(
                outputs(&worker)?.len(),
                0,
                "superseded producer result must not publish"
            );
            assert!(matches!(
                runner.commands.try_recv()?,
                Command::Refresh { .. }
            ));
            worker.expand(a)?;
            let command = runner.commands.try_recv()?;
            let current = match &command {
                Command::Expand { revision, .. } => *revision,
                _ => return Err("expected real queued expansion".into()),
            };
            assert!(runner.process(command, &runtime, &mut state));
            let output = outputs(&worker)?
                .pop()
                .ok_or("real expansion result missing")?;
            assert_eq!(output.revision, Some(current));
            let WorkerEvent::Graph { graph, error } = output.event else {
                return Err("expected native graph".into());
            };
            assert_eq!(error, None);
            assert_edges(&graph, &[link])?;
            let mut actual = graph.nodes;
            actual.sort_by_key(|row| row.id.clone());
            let mut expected = before;
            expected.sort_by_key(|row| row.id.clone());
            assert_eq!(
                actual, expected,
                "obsolete refresh must not replace the retained SQL session"
            );
            drop(worker);
            Ok(())
        })();
        let cleanup = runtime.block_on(native.delete());
        result?;
        cleanup
    })
}

#[test]
fn given_obsolete_selection_queued_before_latest_refresh_when_real_runner_drains_then_last_selection_is_not_mutated()
-> TestResult {
    supervised(|| {
        // Admission filtering has observable state semantics even though emit and
        // drain also reject stale results: a skipped selection must not become the
        // selection re-expanded by the following current refresh.
        let runtime = runtime()?;
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[
                ("shared", "selected-root"),
                ("a-leaf", "selected-leaf"),
                ("b-root", "obsolete-root"),
                ("b-leaf", "obsolete-leaf"),
            ],
            &[],
            1,
        ))?;
        let a = root();
        let a_leaf = fixtures::node(ACCOUNT, REGION, KIND, "a-leaf");
        let b = fixtures::node(ACCOUNT, REGION, KIND, "b-root");
        let b_leaf = fixtures::node(ACCOUNT, REGION, KIND, "b-leaf");
        let selected = edge(&a, &a_leaf, "selected", "model");
        let obsolete = edge(&b, &b_leaf, "obsolete", "reviewed-rule");
        let native = runtime.block_on(NativeGraph::new(&[selected.clone(), obsolete]))?;
        let result: TestResult = (|| {
            let (mut state, _) =
                runtime.block_on(setup(fixture.path.clone(), config(&fixture, &native)))?;
            let established = runtime.block_on(query_graph(&mut state, Some(a)));
            let WorkerEvent::Graph { graph, error } = established else {
                return Err("expected established graph".into());
            };
            assert_eq!(error, None);
            assert_edges(&graph, std::slice::from_ref(&selected))?;
            let (worker, mut runner) = context(2);
            worker.expand(b)?;
            worker.refresh(Filter::default())?;
            let stop = worker.commands.clone();
            let stopper =
                thread::spawn(move || stop.blocking_send(Command::Stop).map_err(Box::new));
            runner.run_commands(&runtime, state);
            stopper.join().map_err(|_| "stop sender panicked")??;
            let events = worker.drain();
            assert_eq!(events.len(), 2);
            let graph = events
                .into_iter()
                .find_map(|event| match event {
                    WorkerEvent::Graph { graph, error: None } => Some(graph),
                    _ => None,
                })
                .ok_or("latest graph refresh missing")?;
            assert_edges(&graph, &[selected])?;
            assert_eq!(
                graph
                    .nodes
                    .iter()
                    .map(|row| row.name.as_str())
                    .collect::<std::collections::BTreeSet<_>>(),
                std::collections::BTreeSet::from(["selected-root", "selected-leaf"])
            );
            drop(worker);
            Ok(())
        })();
        let cleanup = runtime.block_on(native.delete());
        result?;
        cleanup
    })
}

#[test]
fn given_selection_superseded_after_admission_when_native_query_completes_then_producer_does_not_publish_it()
-> TestResult {
    supervised(|| {
        // A producer-boundary test, NOT drain filtering: all observations below are
        // real process/query_graph/native transport outputs before current_events.
        let runtime = runtime()?;
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "root"), ("b", "neighbor")],
            &[],
            1,
        ))?;
        let a = root();
        let b = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = edge(&a, &b, "references", "model");
        let native = runtime.block_on(NativeGraph::new(&[link]))?;
        let result: TestResult = (|| {
            let (mut state, _) =
                runtime.block_on(setup(fixture.path.clone(), config(&fixture, &native)))?;
            let (worker, mut runner) = context(1);
            worker.expand(a)?;
            let command = runner.commands.try_recv()?;
            worker.expand(b)?;
            assert!(runner.process(command, &runtime, &mut state));
            assert_eq!(
                outputs(&worker)?.len(),
                0,
                "emit guard must reject obsolete real query completion before the UI drain"
            );
            drop(worker);
            Ok(())
        })();
        let cleanup = runtime.block_on(native.delete());
        result?;
        cleanup
    })
}

#[test]
fn given_full_queue_when_refresh_is_rejected_then_real_runner_commits_accepted_snapshot()
-> TestResult {
    supervised(|| {
        // Given a retained session before a real writer update and one queue slot.
        let runtime = runtime()?;
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "before",
            ACCOUNT,
            REGION,
            &[("shared", "before")],
            &[],
            1,
        ))?;
        let config = WorkerConfig {
            database: Some(fixture.path.clone()),
            ..WorkerConfig::default()
        };
        let (mut state, _) = runtime.block_on(setup(fixture.path.clone(), config))?;
        let expected = runtime.block_on(fixture.commit(
            "after",
            ACCOUNT,
            REGION,
            &[("new", "new"), ("shared", "after")],
            &[],
            2,
        ))?;
        let (worker, mut runner) = context(1);
        worker.refresh(Filter::default())?;
        // When a second refresh is rejected, only the accepted command is processed.
        assert_eq!(
            worker.refresh(Filter {
                query: "rejected".into(),
                ..Filter::default()
            }),
            Err("inventory worker queue is full".to_owned())
        );
        worker.invalidate_graph()?;
        let command = runner.commands.try_recv()?;
        assert!(runner.process(command, &runtime, &mut state));
        let page = worker
            .drain()
            .into_iter()
            .find_map(|event| match event {
                WorkerEvent::Snapshot {
                    rows,
                    total,
                    error: None,
                    ..
                } => Some((rows, total)),
                _ => None,
            })
            .ok_or("accepted real snapshot lost after rejected refresh")?;
        // Then the UI receives the accepted writer values, not an empty drain.
        assert_eq!(page, (expected, 2));
        Ok(())
    })
}

#[test]
fn given_full_queue_when_expansion_is_rejected_then_real_graph_is_delivered_until_deselect()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        runtime.block_on(fixture.commit(
            "seed",
            ACCOUNT,
            REGION,
            &[("shared", "root"), ("b", "neighbor")],
            &[],
            1,
        ))?;
        let a = root();
        let b = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = edge(&a, &b, "references", "model");
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let result: TestResult = (|| {
            let (mut state, _) =
                runtime.block_on(setup(fixture.path.clone(), config(&fixture, &native)))?;
            let (worker, mut runner) = context(1);
            // Given an accepted expansion, rejection must not revoke its revision.
            worker.expand(a.clone())?;
            assert_eq!(
                worker.expand(b),
                Err("inventory worker queue is full".to_owned())
            );
            let command = runner.commands.try_recv()?;
            assert!(runner.process(command, &runtime, &mut state));
            let delivered = worker.drain().pop().ok_or("accepted native graph lost")?;
            let WorkerEvent::Graph { graph, error: None } = delivered else {
                return Err("native graph failed".into());
            };
            assert_edges(&graph, &[link])?;
            worker.expand(a.clone())?;
            let queued = runner.commands.try_recv()?;
            assert!(runner.process(queued, &runtime, &mut state));
            worker.invalidate_graph()?;
            assert!(
                worker.drain().is_empty(),
                "deselect must reject already queued native output"
            );
            // When deselect happens after admission, the real producer is suppressed.
            worker.expand(a)?;
            let command = runner.commands.try_recv()?;
            worker.invalidate_graph()?;
            assert!(runner.process(command, &runtime, &mut state));
            assert!(
                outputs(&worker)?.is_empty(),
                "post-deselect native result must never publish"
            );
            assert!(worker.drain().is_empty());
            Ok(())
        })();
        let cleanup = runtime.block_on(native.delete());
        result?;
        cleanup
    })
}
