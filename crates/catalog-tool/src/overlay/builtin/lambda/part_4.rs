use crate::overlay::{NameError, OperationOverlay, OverlayClass};
use super::super::{empty};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("UpdateAlias", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateCapacityProvider", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateCodeSigningConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateEventSourceMapping", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateFunctionCode", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateFunctionConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateFunctionEventInvokeConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("UpdateFunctionUrlConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
