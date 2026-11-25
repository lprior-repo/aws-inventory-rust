use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AmendBenefitApplication",
            "AssociateBenefitApplicationResource",
            "CancelBenefitApplication",
            "CreateBenefitApplication",
            "DisassociateBenefitApplicationResource",
            "RecallBenefitApplication",
            "SubmitBenefitApplication",
            "TagResource",
            "UntagResource",
            "UpdateBenefitApplication",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetBenefit",
            "GetBenefitAllocation",
            "GetBenefitApplication",
            "ListBenefitAllocations",
            "ListBenefitApplications",
            "ListBenefits",
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Benefit listing requires catalog selection not enumerated by the pinned model; application details contain customer business content.",
        ),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("partnercentral-benefits", GROUPS, &[])
}
