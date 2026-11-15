use super::{deny};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("workmailmessageflow", vec![
        ("GetRawMessageContent", deny(DenyReason::Content)),
        ("PutRawMessageContent", deny(DenyReason::Mutating)),
    ])
}
