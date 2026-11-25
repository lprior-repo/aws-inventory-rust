use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DeleteConnectionRecordingPreferences",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetConnectionRecordingPreferences",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ClientToken, ConnectionRecordingPreferences. Pinned request/ownership contract: Returns the preferences specified for recording RDP connections in the requesting Amazon Web Services account and Amazon Web Services Region.",
        ),
    ),
    (
        "UpdateConnectionRecordingPreferences",
        Decision::Denied(DenyReason::Mutating),
    ),
];
