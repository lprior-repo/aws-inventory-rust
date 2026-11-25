use super::{ProjectionError, validate};

#[test]
fn graph_names_and_batches_are_strictly_bounded() -> Result<(), Box<dyn std::error::Error>> {
    validate("inventory-graph_1", "redis://localhost", 1)?;
    [
        ("inventory graph", 1),
        ("inventory", 0),
        ("inventory", 1001),
    ]
    .into_iter()
    .try_for_each(|(graph, batch)| {
        let error = validate(graph, "redis://localhost", batch)
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "projection graph name or batch limit is invalid"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn response_errors_are_fatal_but_transport_errors_are_offline()
-> Result<(), Box<dyn std::error::Error>> {
    use super::{EntryError, entry_graph_error};
    use inventory_graph::GraphError;
    let response = redis::RedisError::from((
        redis::ErrorKind::Server(redis::ServerErrorKind::ResponseError),
        "bad query",
    ));
    assert!(matches!(
        entry_graph_error(GraphError::from(response)),
        EntryError::Projection(ProjectionError::Query)
    ));
    let transport = redis::RedisError::from(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "offline",
    ));
    assert!(matches!(
        entry_graph_error(GraphError::from(transport)),
        EntryError::Offline
    ));
    Ok(())
}

#[test]
fn graph_and_endpoint_byte_limits_accept_the_boundary_and_reject_the_next_byte()
-> Result<(), Box<dyn std::error::Error>> {
    validate(&"g".repeat(128), &"e".repeat(4096), 1)?;
    [
        ("g".repeat(129), "e".repeat(4096)),
        ("g".repeat(128), "e".repeat(4097)),
    ]
    .into_iter()
    .try_for_each(|(graph, endpoint)| {
        let error = validate(&graph, &endpoint, 1)
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "projection graph name or batch limit is invalid"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
