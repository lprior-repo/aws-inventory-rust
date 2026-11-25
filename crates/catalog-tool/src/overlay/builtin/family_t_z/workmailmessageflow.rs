use super::super::service;
use super::deny;
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workmailmessageflow",
        vec![
            ("GetRawMessageContent", deny(DenyReason::Content)),
            ("PutRawMessageContent", deny(DenyReason::Mutating)),
        ],
    )
}
