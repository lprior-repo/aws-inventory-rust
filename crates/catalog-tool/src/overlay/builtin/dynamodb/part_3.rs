use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateTableReplicaAutoScaling",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UpdateTimeToLive",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
    ])
}
