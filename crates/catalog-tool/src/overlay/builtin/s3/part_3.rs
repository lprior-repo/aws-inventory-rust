use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteBucketIntelligentTieringConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketInventoryConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketLifecycle",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketMetadataConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketMetadataTableConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketMetricsConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketOwnershipControls",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketPolicy",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketReplication",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketTagging",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBucketWebsite",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteObject",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteObjectAnnotation",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteObjectTagging",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteObjects",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeletePublicAccessBlock",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutBucketAbac",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutBucketAccelerateConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutBucketAcl",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "PutBucketAnalyticsConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
    ])
}
