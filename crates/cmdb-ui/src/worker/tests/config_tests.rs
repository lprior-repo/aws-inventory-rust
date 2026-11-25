use super::*;

#[test]
fn given_full_or_disconnected_intake_when_checking_worker_then_only_disconnection_is_closed()
-> TestResult {
    let (worker, commands, _events, _cancellation) = harness(1);
    assert!(!worker.is_closed());
    worker.refresh(Filter::default())?;
    assert_eq!(
        worker.refresh(Filter::default()),
        Err("inventory worker queue is full".to_owned())
    );
    assert!(!worker.is_closed());
    drop(commands);
    assert!(worker.is_closed());
    Ok(())
}

#[test]
fn given_valid_lod_config_string_then_parse_returns_custom_limits() -> TestResult {
    let limits = parse_lod_config("3:500:1000")?;
    assert_eq!(limits.depth, 3);
    assert_eq!(limits.nodes, 500);
    assert_eq!(limits.edges, 1000);
    Ok(())
}

#[test]
fn given_invalid_lod_format_when_parsing_then_error_is_returned() -> TestResult {
    let Err(error) = parse_lod_config("invalid") else {
        return Err("invalid lod format must fail".into());
    };
    assert!(error.contains("depth:nodes:edges"));
    Ok(())
}

#[test]
fn given_lod_with_partial_numbers_when_parsing_then_error_is_returned() -> TestResult {
    let Err(error) = parse_lod_config("1:2:abc") else {
        return Err("partial lod numbers must fail".into());
    };
    assert!(error.contains("invalid edges"));
    Ok(())
}

#[test]
fn given_default_config_then_default_lod_limits_are_used() {
    let config = WorkerConfig::default();
    assert_eq!(config.graph_limits, DEFAULT_GRAPH_LIMITS);
    assert_eq!(config.graph_limits.depth, 2);
    assert_eq!(config.graph_limits.nodes, 200);
    assert_eq!(config.graph_limits.edges, 500);
}

#[test]
fn given_lod_with_wrong_field_count_when_parsing_then_error_is_returned() {
    assert!(parse_lod_config("1:2").is_err());
    assert!(parse_lod_config("1:2:3:4").is_err());
}
