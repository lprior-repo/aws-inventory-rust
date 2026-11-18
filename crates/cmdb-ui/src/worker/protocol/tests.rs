use super::*;
use crate::data::{Counts, Graph};

#[test]
fn given_new_page_and_newer_selection_when_draining_then_access_status_uses_page_fence_without_resurrecting_old_graph()
-> Result<(), String> {
    let revisions = Revisions::default();
    let page = revisions.refresh()?;
    let selection = revisions.expand()?;
    assert!(selection > page);
    let events = current_events(
        [
            Output {
                revision: Some(0),
                event: WorkerEvent::GraphStatus {
                    error: Some("obsolete configuration".into()),
                },
            },
            Output {
                revision: Some(page),
                event: WorkerEvent::GraphStatus {
                    error: Some("current configuration".into()),
                },
            },
            Output {
                revision: Some(page),
                event: WorkerEvent::Graph {
                    graph: Graph::default(),
                    error: Some("obsolete selection query".into()),
                },
            },
            Output {
                revision: Some(page),
                event: WorkerEvent::Snapshot {
                    counts: Counts::default(),
                    rows: vec![],
                    total: 0,
                    error: None,
                },
            },
        ]
        .into_iter(),
        &revisions,
    );
    assert_eq!(events.len(), 2);
    assert!(
        matches!(events.first(), Some(WorkerEvent::GraphStatus { error: Some(error) }) if error == "current configuration")
    );
    assert!(matches!(
        events.last(),
        Some(WorkerEvent::Snapshot {
            total: 0,
            error: None,
            ..
        })
    ));
    Ok(())
}
