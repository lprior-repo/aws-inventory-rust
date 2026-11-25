use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[(
    "InvokeEndpointWithBidirectionalStream",
    Decision::Denied(DenyReason::Paid),
)];
