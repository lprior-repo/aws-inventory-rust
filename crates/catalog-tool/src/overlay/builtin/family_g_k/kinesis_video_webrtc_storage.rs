//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kinesis-video-webrtc-storage/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![

    ];
    policy::finish(
        "kinesis-video-webrtc-storage", approved, DEFERRED,
        "Joining storage/viewer sessions starts or creates streaming session state; every operation is denied.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[

];

const MUTATING: &[&str] = &[
    "JoinStorageSession", "JoinStorageSessionAsViewer",
];
