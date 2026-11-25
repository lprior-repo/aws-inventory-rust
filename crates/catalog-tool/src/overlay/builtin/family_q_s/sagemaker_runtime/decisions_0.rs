use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("InvokeEndpoint", Decision::Denied(DenyReason::Paid)),
    ("InvokeEndpointAsync", Decision::Denied(DenyReason::Paid)),
    (
        "InvokeEndpointWithResponseStream",
        Decision::Denied(DenyReason::Paid),
    ),
];
