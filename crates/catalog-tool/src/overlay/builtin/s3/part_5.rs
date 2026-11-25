use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutObjectRetention",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutObjectTagging",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutPublicAccessBlock",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "RenameObject",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "RestoreObject",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UpdateBucketMetadataAnnotationTableConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UpdateBucketMetadataInventoryTableConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UpdateBucketMetadataJournalTableConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UpdateObjectEncryption",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UploadPart",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "UploadPartCopy",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "WriteGetObjectResponse",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
    ])
}
