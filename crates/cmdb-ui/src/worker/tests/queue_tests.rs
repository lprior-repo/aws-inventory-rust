use super::*;
use std::thread;
use std::time::Duration;

#[test]
fn given_closed_intake_when_command_is_submitted_then_stopped_error_is_explicit() {
    let (worker, commands, _events, _cancellation) = harness(1);
    drop(commands);
    assert_eq!(
        worker.refresh(Filter::default()),
        Err("inventory worker stopped".to_owned())
    );
}

#[tokio::test]
async fn given_active_query_when_shutdown_requested_then_pending_io_is_cancelled() -> TestResult {
    let (shutdown, mut cancellation) = crate::worker::shutdown_channel();
    let (started, ready) = tokio::sync::oneshot::channel();
    let operation = cancellation.run(async {
        started
            .send(())
            .map_err(|()| "query handshake disconnected")?;
        std::future::pending::<Result<(), String>>().await
    });
    let stop = async {
        ready.await?;
        shutdown.stop();
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    let (outcome, stopped) = tokio::join!(operation, stop);
    stopped?;
    assert!(outcome.is_none());
    Ok(())
}

#[test]
fn given_full_result_queue_when_worker_dropped_then_blocked_producer_is_released() -> TestResult {
    let (mut worker, _commands, events, _cancellation) = harness(1);
    (0..crate::worker::EVENT_CAPACITY)
        .try_for_each(|_| events.send(page(0, 1)).map_err(Box::new))?;
    let (started, ready) = mpsc::sync_channel(1);
    let (outcome, result) = mpsc::sync_channel(1);
    worker.thread = Some(thread::spawn(move || {
        if started.send(()).is_err() {
            return;
        }
        assert!(outcome.send(events.send(page(0, 2)).is_err()).is_ok());
    }));
    ready.recv_timeout(Duration::from_secs(1))?;
    let (finished, completion) = mpsc::sync_channel(1);
    let dropper = thread::spawn(move || {
        drop(worker);
        assert!(finished.send(()).is_ok());
    });
    completion.recv_timeout(Duration::from_secs(1))?;
    assert!(dropper.join().is_ok());
    assert!(result.recv_timeout(Duration::from_secs(1))?);
    Ok(())
}

#[test]
fn given_full_results_when_ui_drains_then_critical_output_is_preserved() -> TestResult {
    let (worker, _commands, events, _cancellation) = harness(1);
    (0..crate::worker::EVENT_CAPACITY)
        .try_for_each(|_| events.send(page(0, 1)).map_err(Box::new))?;
    let (started, ready) = mpsc::sync_channel(1);
    let producer = thread::spawn(move || {
        if started.send(()).is_err() {
            return;
        }
        assert!(events.send(graph(0, "critical graph failure")).is_ok());
    });
    ready.recv_timeout(Duration::from_secs(1))?;
    assert!(matches!(
        worker.drain().last(),
        Some(WorkerEvent::Snapshot { total: 1, .. })
    ));
    assert!(producer.join().is_ok());
    assert!(
        matches!(worker.drain().first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "critical graph failure")
    );
    Ok(())
}

#[test]
fn given_saturated_queue_when_terminal_graph_failure_is_blocked_then_it_is_delivered_exactly_once()
-> TestResult {
    let (worker, _commands, events, _cancellation) = harness(1);
    (0..crate::worker::EVENT_CAPACITY)
        .try_for_each(|_| events.send(page(0, 1)).map_err(Box::new))?;
    let (started, ready) = mpsc::sync_channel(1);
    let producer = thread::spawn(move || {
        if started.send(()).is_err() {
            return;
        }
        assert!(events.send(graph(0, "terminal graph failure")).is_ok());
    });
    ready.recv_timeout(Duration::from_secs(1))?;
    let first = worker.drain();
    assert!(producer.join().is_ok());
    let second = worker.drain();
    let third = worker.drain();
    let terminal = |events: &[WorkerEvent]| {
        events
            .iter()
            .filter(|event| {
                matches!(event, WorkerEvent::Graph { error: Some(error), .. } if error == "terminal graph failure")
            })
            .count()
    };
    assert_eq!(
        terminal(&first) + terminal(&second) + terminal(&third),
        1,
        "terminal failure must survive saturation exactly once"
    );
    assert!(third.is_empty(), "delivered terminal state must not replay");
    Ok(())
}
