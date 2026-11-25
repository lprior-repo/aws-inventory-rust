use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptQualificationRequest",
            "AssociateQualificationWithWorker",
            "CreateAdditionalAssignmentsForHIT",
            "CreateHIT",
            "CreateHITType",
            "CreateHITWithHITType",
            "CreateQualificationType",
            "CreateWorkerBlock",
            "DeleteHIT",
            "DeleteQualificationType",
            "DeleteWorkerBlock",
            "DisassociateQualificationFromWorker",
            "NotifyWorkers",
            "RejectAssignment",
            "RejectQualificationRequest",
            "SendBonus",
            "SendTestEventNotification",
            "UpdateExpirationForHIT",
            "UpdateHITReviewStatus",
            "UpdateHITTypeOfHIT",
            "UpdateNotificationSettings",
            "UpdateQualificationType",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "ApproveAssignment",
            "GetAccountBalance",
            "GetQualificationScore",
            "GetQualificationType",
            "ListBonusPayments",
            "ListQualificationRequests",
            "ListQualificationTypes",
            "ListReviewPolicyResultsForHIT",
            "ListWorkerBlocks",
            "ListWorkersWithQualificationType",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Worker/assignment/HIT responses contain task questions, answers or worker records; no metadata-only infrastructure enumeration is available.",
        ),
    },
    Group {
        operations: &[
            "GetAssignment",
            "GetFileUploadURL",
            "GetHIT",
            "ListAssignmentsForHIT",
            "ListHITs",
            "ListHITsForQualificationType",
            "ListReviewableHITs",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mturk", GROUPS, &[])
}
