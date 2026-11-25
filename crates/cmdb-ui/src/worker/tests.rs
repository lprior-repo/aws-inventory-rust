pub(crate) mod config_tests;
pub(crate) mod graph_tests;
pub(crate) mod isolation;
pub(crate) mod no_socket;
pub(crate) mod queue_tests;
pub(crate) mod real;
mod shutdown;
pub(crate) mod support;

use super::*;
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

fn harness(
    capacity: usize,
) -> (
    DataWorker,
    CommandReceiver<Command>,
    SyncSender<Output>,
    Cancellation,
) {
    let (commands, receiver) = tokio::sync::mpsc::channel(capacity);
    let (events, outputs) = mpsc::sync_channel(EVENT_CAPACITY);
    let (shutdown, cancellation) = shutdown_channel();
    let worker = DataWorker {
        commands,
        events: Some(outputs),
        revisions: Arc::new(Revisions::default()),
        shutdown,
        thread: None,
    };
    (worker, receiver, events, cancellation)
}

fn page(revision: u64, total: u64) -> Output {
    Output {
        revision: Some(revision),
        event: WorkerEvent::Snapshot {
            counts: Counts {
                resources: total,
                ..Counts::default()
            },
            rows: Vec::new(),
            total,
            error: None,
        },
    }
}

fn node(id: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::EC2::VPC".to_owned(),
        id: id.to_owned(),
    }
}

fn graph(revision: u64, error: &str) -> Output {
    Output {
        revision: Some(revision),
        event: graph_failure(error.to_owned()),
    }
}

fn revision(command: Command) -> Result<u64, String> {
    match command {
        Command::Refresh { revision, .. } | Command::Expand { revision, .. } => Ok(revision),
        Command::Stop => Err("unexpected stop command".to_owned()),
    }
}

#[test]
fn given_new_filter_when_old_page_finishes_last_then_only_latest_page_is_delivered() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.refresh(Filter {
        query: "old".to_owned(),
        ..Filter::default()
    })?;
    let old = revision(commands.try_recv()?)?;
    worker.refresh(Filter {
        query: "new".to_owned(),
        ..Filter::default()
    })?;
    let new = revision(commands.try_recv()?)?;
    events.send(page(new, 7))?;
    events.send(page(old, 99))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(matches!(
        drained.first(),
        Some(WorkerEvent::Snapshot {
            total: 7,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_full_queue_when_new_filter_is_rejected_then_accepted_page_is_preserved() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    let rejection = worker.refresh(Filter {
        query: "rejected".to_owned(),
        ..Filter::default()
    });
    assert_eq!(rejection, Err("inventory worker queue is full".to_owned()));
    events.send(page(revision(commands.try_recv()?)?, 3))?;
    assert!(matches!(
        worker.drain().first(),
        Some(WorkerEvent::Snapshot {
            total: 3,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_new_selection_when_old_expansion_finishes_then_only_new_graph_is_delivered() -> TestResult
{
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.expand(node("vpc-old"))?;
    let old = revision(commands.try_recv()?)?;
    worker.expand(node("vpc-new"))?;
    let new = revision(commands.try_recv()?)?;
    events.send(graph(new, "new selection"))?;
    events.send(graph(old, "old selection"))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(
        matches!(drained.first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "new selection")
    );
    Ok(())
}

#[test]
fn given_filter_change_when_prior_expansion_finishes_then_graph_is_not_applied() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.expand(node("vpc-old"))?;
    let old = revision(commands.try_recv()?)?;
    worker.refresh(Filter::default())?;
    let new = revision(commands.try_recv()?)?;
    events.send(graph(old, "old selection"))?;
    events.send(page(new, 4))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(matches!(
        drained.first(),
        Some(WorkerEvent::Snapshot { total: 4, .. })
    ));
    Ok(())
}

#[test]
fn given_pending_page_when_selection_changes_then_page_still_finishes_loading() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    worker.expand(node("vpc-new"))?;
    events.send(page(request, 5))?;
    assert!(matches!(
        worker.drain().first(),
        Some(WorkerEvent::Snapshot { total: 5, .. })
    ));
    Ok(())
}

#[test]
fn given_offline_graph_when_table_arrives_then_graph_status_precedes_page_selection() -> TestResult
{
    let (worker, mut commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    events.send(page(request, 8))?;
    events.send(graph(request, "connection refused"))?;
    let drained = worker.drain();
    assert!(
        matches!(drained.first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "connection refused")
    );
    assert!(matches!(
        drained.last(),
        Some(WorkerEvent::Snapshot {
            total: 8,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_startup_failure_after_filter_change_then_terminal_error_is_not_filtered() -> TestResult {
    let (worker, _commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    events.send(Output {
        revision: None,
        event: database_error("schema version 13".to_owned()),
    })?;
    assert!(
        matches!(worker.drain().first(), Some(WorkerEvent::Snapshot { error: Some(error), .. }) if error == "schema version 13")
    );
    Ok(())
}


