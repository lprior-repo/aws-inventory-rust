use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdatePrimaryRegion",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "Decapsulate",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "Decrypt",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "DeriveSharedSecret",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "Encapsulate",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "GenerateDataKey",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "GenerateDataKeyPair",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "GenerateRandom",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Secret)),
                ..empty()
            },
        ),
        (
            "Encrypt",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "GenerateDataKeyPairWithoutPlaintext",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "GenerateDataKeyWithoutPlaintext",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "GenerateMac",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "ReEncrypt",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "Sign",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "Verify",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
        (
            "VerifyMac",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Paid)),
                ..empty()
            },
        ),
    ])
}
