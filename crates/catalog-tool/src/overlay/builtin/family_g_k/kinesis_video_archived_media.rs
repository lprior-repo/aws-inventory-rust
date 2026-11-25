use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "kinesis-video-archived-media",
        approved,
        DEFERRED,
        "Clips, images, media, fragment inventories and playback URLs expose media content/capabilities rather than resource configuration; every operation is denied.",
        &[(policy::DenyReason::Content, CONTENT)],
    )
}

const DEFERRED: &[&str] = &[];

const CONTENT: &[&str] = &[
    "GetClip",
    "GetDASHStreamingSessionURL",
    "GetHLSStreamingSessionURL",
    "GetImages",
    "GetMediaForFragmentList",
    "ListFragments",
];
