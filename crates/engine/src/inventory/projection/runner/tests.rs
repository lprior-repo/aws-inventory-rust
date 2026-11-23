use super::{ProjectionError, ProjectionIdentity, report, validate};

#[test]
fn graph_names_and_batches_are_strictly_bounded() {
    assert!(validate("inventory-graph_1", "redis://localhost", 1).is_ok());
    assert!(matches!(
        validate("inventory graph", "redis://localhost", 1),
        Err(ProjectionError::InvalidConfiguration)
    ));
    assert!(matches!(
        validate("inventory", "redis://localhost", 0),
        Err(ProjectionError::InvalidConfiguration)
    ));
    assert!(matches!(
        validate("inventory", "redis://localhost", 1001),
        Err(ProjectionError::InvalidConfiguration)
    ));
}

#[test]
fn report_retains_pending_when_delivery_is_offline_or_rebuild_is_partial() {
    let offline = report(3, 0, true, false);
    let partial = report(2, 1, false, true);
    assert_eq!(offline.pending, 3);
    assert!(offline.offline);
    assert_eq!(partial.pending, 2);
    assert!(partial.rebuild_incomplete);
}

#[test]
fn identity_differs_on_endpoint_change() {
    let stored = ProjectionIdentity {
        endpoint: "redis://localhost:6379".to_owned(),
        graph: "inventory".to_owned(),
    };
    assert!(!stored.differs("redis://localhost:6379", "inventory"));
    assert!(stored.differs("redis://localhost:7000", "inventory"));
}

#[test]
fn identity_differs_on_graph_name_change() {
    let stored = ProjectionIdentity {
        endpoint: "redis://localhost:6379".to_owned(),
        graph: "inventory".to_owned(),
    };
    assert!(stored.differs("redis://localhost:6379", "other_graph"));
}

#[test]
fn stale_graph_identity_requires_rebuild() {
    let stale = report(0, 0, false, true);
    assert!(stale.rebuild_incomplete);
    assert_eq!(stale.pending, 0);
    assert_eq!(stale.acknowledged, 0);
}

#[test]
fn response_errors_are_fatal_but_transport_errors_are_offline() {
    use super::super::client::GraphError;
    use super::{EntryError, entry_graph_error};
    let response = redis::RedisError::from((
        redis::ErrorKind::Server(redis::ServerErrorKind::ResponseError),
        "bad query",
    ));
    assert!(matches!(
        entry_graph_error(GraphError::Redis(response)),
        EntryError::Projection(ProjectionError::Query)
    ));
    let transport = redis::RedisError::from(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "offline",
    ));
    assert!(matches!(
        entry_graph_error(GraphError::Redis(transport)),
        EntryError::Offline
    ));
}
