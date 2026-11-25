use super::*;
use crate::worker::DEFAULT_GRAPH_LIMITS;
use crate::worker::tests::isolation::supervised;
use crate::worker::tests::real::fixtures::{
    self, ACCOUNT, Fixture, KIND, NativeGraph, REGION, TestResult, assert_edges, edge,
    falkor_available, root,
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
        pending_test_events: std::cell::RefCell::default(),
    };
    (worker, context)
}

fn config(fixture: &Fixture, native: &NativeGraph) -> WorkerConfig {
    WorkerConfig {
        database: Some(fixture.path.to_string_lossy().into_owned()),
        graph_url: Some(native.endpoint.clone()),
        graph_name: Some(native.name.clone()),
        graph_limits: DEFAULT_GRAPH_LIMITS,
        ..WorkerConfig::default()
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

fn protocol_refresh(worker: &DataWorker, filter: Filter) -> Result<Command, String> {
    Ok(Command::Refresh {
        revision: worker.revisions.refresh()?,
        filter,
    })
}

fn protocol_expansion(worker: &DataWorker, node: GraphNode) -> Result<Command, String> {
    Ok(Command::Expand {
        revision: worker.revisions.expand()?,
        node,
    })
}

#[test]
fn given_superseded_refresh_after_admission_when_runner_completes_then_next_expansion_uses_last_committed_session()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
        let _entered = runtime.enter();
        let fixture = runtime.block_on(Fixture::new())?;
        let a = root();
        let b = fixtures::node(ACCOUNT, REGION, KIND, "b");
        let link = edge(&a, &b, "references", "model");
        runtime.block_on(fixture.commit(
            "before",
            ACCOUNT,
            REGION,
            &[("shared", "before-root"), ("b", "before-neighbor")],
            &[],
            1,
        ))?;
        let native = runtime.block_on(NativeGraph::new(std::slice::from_ref(&link)))?;
        let result: TestResult = (|| {
            let (mut state, _) = runtime.block_on(setup(
                fixture.path.to_string_lossy().into_owned(),
                config(&fixture, &native),
            ))?;
            let after = runtime.block_on(fixture.commit(
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
            let command = protocol_refresh(&worker, Filter::default())?;
            let current = protocol_refresh(
                &worker,
                Filter {
                    query: "newer-accepted".into(),
                    ..Filter::default()
                },
            )?;
            worker.commands.try_send(current)?;
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
            let mut expected: Vec<_> = after
                .into_iter()
                .filter(|row| matches!(row.id.as_str(), "shared" | "b"))
                .collect();
            expected.sort_by_key(|row| row.id.clone());
            assert_eq!(
                actual, expected,
                "obsolete refresh must not replace the last committed SQL session"
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
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
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
            let (mut state, _) = runtime.block_on(setup(
                fixture.path.to_string_lossy().into_owned(),
                config(&fixture, &native),
            ))?;
            let established = runtime.block_on(query_graph(&mut state, Some(a)));
            let WorkerEvent::Graph { graph, error } = established else {
                return Err("expected established graph".into());
            };
            assert_eq!(error, None);
            assert_edges(&graph, std::slice::from_ref(&selected))?;
            let (worker, mut runner) = context(2);
            worker.commands.try_send(protocol_expansion(&worker, b)?)?;
            worker
                .commands
                .try_send(protocol_refresh(&worker, Filter::default())?)?;
            let stop = worker.commands.clone();
            let stopper =
                thread::spawn(move || stop.blocking_send(Command::Stop).map_err(Box::new));
            runner.run_commands(&runtime, state);
            stopper.join().map_err(|_| "stop sender panicked")??;
            let events = worker.drain();
            assert_eq!(events.len(), 3);
            assert!(matches!(
                events.first(),
                Some(WorkerEvent::GraphStatus { error: None })
            ));
            assert!(matches!(
                events.last(),
                Some(WorkerEvent::Snapshot {
                    total: 4,
                    error: None,
                    ..
                })
            ));
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
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
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
            let (mut state, _) = runtime.block_on(setup(
                fixture.path.to_string_lossy().into_owned(),
                config(&fixture, &native),
            ))?;
            let (worker, mut runner) = context(1);
            let command = protocol_expansion(&worker, a)?;
            worker.commands.try_send(protocol_expansion(&worker, b)?)?;
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
            database: Some(fixture.path.to_string_lossy().into_owned()),
            graph_limits: DEFAULT_GRAPH_LIMITS,
            ..WorkerConfig::default()
        };
        let (mut state, _) =
            runtime.block_on(setup(fixture.path.to_string_lossy().into_owned(), config))?;
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
        assert_eq!(page, (expected, 2));
        Ok(())
    })
}

#[test]
fn given_full_queue_when_expansion_is_rejected_then_real_graph_is_delivered_until_deselect()
-> TestResult {
    supervised(|| {
        let runtime = runtime()?;
        if !runtime.block_on(falkor_available())? {
            return Ok(());
        }
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
            let (mut state, _) = runtime.block_on(setup(
                fixture.path.to_string_lossy().into_owned(),
                config(&fixture, &native),
            ))?;
            let (worker, mut runner) = context(1);
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
