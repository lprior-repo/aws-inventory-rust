use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "kinesis-video-webrtc-storage",
        approved,
        DEFERRED,
        "Joining storage/viewer sessions starts or creates streaming session state; every operation is denied.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[];

const MUTATING: &[&str] = &["JoinStorageSession", "JoinStorageSessionAsViewer"];
