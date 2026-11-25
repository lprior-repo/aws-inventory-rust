use super::support::reviewed;
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ebs",
        vec![],
        &[],
        &[
            ("GetSnapshotBlock", DenyReason::Content),
            ("ListSnapshotBlocks", DenyReason::Content),
            ("ListChangedBlocks", DenyReason::Content),
            ("StartSnapshot", DenyReason::Mutating),
            ("PutSnapshotBlock", DenyReason::Mutating),
            ("CompleteSnapshot", DenyReason::Mutating),
        ],
    )
}
