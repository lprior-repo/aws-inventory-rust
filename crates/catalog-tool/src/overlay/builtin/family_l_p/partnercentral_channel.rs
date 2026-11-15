// Pinned Smithy source: .local/aws-models/models/partnercentral-channel/service/2024-03-18/partnercentral-channel-2024-03-18.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptChannelHandshake", "CancelChannelHandshake", "CreateChannelHandshake", 
            "CreateProgramManagementAccount", "CreateRelationship", 
            "DeleteProgramManagementAccount", "DeleteRelationship", "RejectChannelHandshake", 
            "TagResource", "UntagResource", "UpdateProgramManagementAccount", 
            "UpdateRelationship",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetRelationship", "ListChannelHandshakes", "ListProgramManagementAccounts", 
            "ListRelationships", "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Relationship listing requires catalog selection not enumerated by the pinned model; do not guess a production or sandbox catalog."),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("partnercentral-channel", GROUPS, &[])
}
