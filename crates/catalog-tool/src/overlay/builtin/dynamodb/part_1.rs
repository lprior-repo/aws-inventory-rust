use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeExport",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "DescribeEndpoints",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
                ..empty()
            },
        ),
        (
            "DescribeLimits",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
                ..empty()
            },
        ),
        (
            "BatchExecuteStatement",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "BatchGetItem",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "ExecuteStatement",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "ExecuteTransaction",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "GetItem",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "Query",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "Scan",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "SearchVectors",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "TransactGetItems",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Content)),
                ..empty()
            },
        ),
        (
            "BatchWriteItem",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "CancelBulkOperation",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "CreateBackup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "CreateGlobalTable",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "CreateTable",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteBackup",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteItem",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
        (
            "DeleteResourcePolicy",
            OperationOverlay {
                class: Some(OverlayClass::Deny(DenyReason::Mutating)),
                ..empty()
            },
        ),
    ])
}
