use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeConversionTasks",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "DescribeInstanceTypes",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
                ..empty()
            },
        ),
        (
            "DescribeVerifiedAccessTrustProviders",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "GetPasswordData",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
    ])
}
