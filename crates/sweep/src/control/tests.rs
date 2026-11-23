mod fixture;
mod peer;
mod scenarios;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use super::*;

#[test]
fn control_without_listener_never_constructs_events() {
    let progress = Progress::new(None);
    let constructed = std::cell::Cell::new(false);
    progress.emit(|| {
        constructed.set(true);
        ProgressEvent::Queue {
            pending: 1,
            in_flight: 1,
        }
    });
    assert!(!constructed.get());
    assert_eq!(
        progress.finish(SweepSummary::default()),
        SweepSummary::default()
    );
}

#[test]
fn control_unread_bounded_listener_counts_every_dropped_event() -> TestResult {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let progress = Progress::new(Some(sender));
    let queued = ProgressEvent::Queue {
        pending: 2,
        in_flight: 1,
    };
    progress.emit(|| queued.clone());
    progress.emit(|| ProgressEvent::Page {
        rows: 3,
        resource_writes: 3,
    });
    let summary = progress.finish(SweepSummary::default());
    assert_eq!(summary.metrics.progress_dropped, 2);
    assert_eq!(receiver.try_recv()?, queued);
    assert_eq!(
        receiver.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    );
    Ok(())
}

#[test]
fn control_delivered_finish_matches_returned_summary() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let progress = Progress::new(Some(sender));
    let summary = progress.finish(SweepSummary::default());
    assert_eq!(receiver.try_recv()?, ProgressEvent::Finished { summary });
    Ok(())
}

#[test]
fn control_closed_listener_does_not_count_as_full() {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    drop(receiver);
    let progress = Progress::new(Some(sender));
    assert_eq!(
        progress
            .finish(SweepSummary::default())
            .metrics
            .progress_dropped,
        0
    );
}
