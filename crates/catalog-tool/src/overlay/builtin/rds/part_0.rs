use crate::overlay::{NameError, OperationOverlay, OverlayClass};
use super::super::{empty};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("AddRoleToDBCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AddRoleToDBInstance", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AddSourceIdentifierToSubscription", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AddTagsToResource", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ApplyPendingMaintenanceAction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AuthorizeDBSecurityGroupIngress", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("BacktrackDBCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CancelDBCapacityReservation", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CancelExportTask", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyDBClusterParameterGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyDBClusterSnapshot", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyDBParameterGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyDBSnapshot", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyOptionGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateBlueGreenDeployment", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateCustomDBEngineVersion", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateDBCapacityReservation", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateDBCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateDBClusterEndpoint", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateDBClusterParameterGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
