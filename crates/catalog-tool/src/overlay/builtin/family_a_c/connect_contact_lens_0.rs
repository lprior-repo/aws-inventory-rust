use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![(
        "ListRealtimeContactAnalysisSegments",
        denied(DenyReason::Content),
    )])
}
