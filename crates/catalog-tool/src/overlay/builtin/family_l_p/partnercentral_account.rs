// Pinned Smithy source: .local/aws-models/models/partnercentral-account/service/2025-04-04/partnercentral-account-2025-04-04.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptConnectionInvitation", "AssociateAwsTrainingCertificationEmailDomain", 
            "CancelConnection", "CancelConnectionInvitation", "CancelProfileUpdateTask", 
            "CreateConnectionInvitation", "CreatePartner", 
            "DisassociateAwsTrainingCertificationEmailDomain", "PutAllianceLeadContact", 
            "PutProfileVisibility", "RejectConnectionInvitation", "SendEmailVerificationCode", 
            "StartProfileUpdateTask", "StartQualificationsAssociationTask", 
            "StartQualificationsDisassociationTask", "StartVerification", "TagResource", 
            "UntagResource", "UpdateConnectionPreferences",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAllianceLeadContact", "GetConnection", "GetConnectionInvitation", 
            "GetConnectionPreferences", "GetPartner", "GetProfileUpdateTask", 
            "GetProfileVisibility", "GetQualificationsAssociationDetails", 
            "GetQualificationsAssociationTask", "GetQualificationsDisassociationTask", 
            "GetVerification", "ListConnectionInvitations", "ListConnections", "ListPartners", 
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Partner account listing requires a catalog selection whose accepted value set is not pinned by this model; profile/contact bodies are not infrastructure inventory."),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("partnercentral-account", GROUPS, &[])
}
