// Pinned source: .local/aws-models/models/connect-contact-lens/service/2020-08-21/connect-contact-lens-2020-08-21.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Provides a list of analysis segments for a real-time analysis session for voice. Voice data is retained for 24 hours. You must invoke this API during that time.
        (
            "ListRealtimeContactAnalysisSegments",
            denied(DenyReason::Content),
        ),
    ])
}
