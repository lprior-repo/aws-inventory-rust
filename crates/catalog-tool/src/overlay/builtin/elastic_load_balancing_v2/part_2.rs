use crate::overlay::{NameError, OperationOverlay, OverlayClass};
use super::super::{empty};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ModifyTargetGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ModifyTargetGroupAttributes", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ModifyTrustStore", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RegisterTargets", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveListenerCertificates", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveTags", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveTrustStoreRevocations", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SetIpAddressType", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SetRulePriorities", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SetSecurityGroups", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SetSubnets", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
