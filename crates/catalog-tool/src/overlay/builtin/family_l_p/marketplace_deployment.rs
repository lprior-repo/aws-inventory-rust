// Pinned Smithy source: .local/aws-models/models/marketplace-deployment/service/2023-01-25/marketplace-deployment-2023-01-25.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Deployment creates registration-token access and tag reads require externally supplied resource ARN; no resource discovery operation exists.",
        ),
    },
    Group {
        operations: &["PutDeploymentParameter", "TagResource", "UntagResource"],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-deployment", GROUPS, &[])
}
