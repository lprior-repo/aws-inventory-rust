use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_truncated_search_when_converting_then_absence_is_not_claimed() -> TestResult {
    // Given: the graph search could not finish within its bounds.
    // When: its outcome is converted for view consumers.
    // Then: it must remain distinct from proven absence.
    assert_eq!(
        path::convert(Err(GraphError::Truncated))?,
        PathResult::Truncated
    );
    let error = path::convert(Err(GraphError::Timeout))
        .err()
        .ok_or("expected an error")?;
    assert!(matches!(error, ViewError::GraphTimeout));
    Ok(())
}
