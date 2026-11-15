// Pinned operation purposes: .local/aws-models/models/ssm-guiconnect/service/2021-05-01/ssm-guiconnect-2021-05-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the preferences for recording RDP connections.
    ("DeleteConnectionRecordingPreferences", Decision::Denied(DenyReason::Mutating)),
    ("GetConnectionRecordingPreferences", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ClientToken, ConnectionRecordingPreferences. Pinned request/ownership contract: Returns the preferences specified for recording RDP connections in the requesting Amazon Web Services account and Amazon Web Services Region.")),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the preferences for recording RDP connections.
    ("UpdateConnectionRecordingPreferences", Decision::Denied(DenyReason::Mutating)),
];
