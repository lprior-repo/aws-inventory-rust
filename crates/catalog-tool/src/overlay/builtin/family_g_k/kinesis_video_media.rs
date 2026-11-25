use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "kinesis-video-media",
        approved,
        DEFERRED,
        "GetMedia returns a media payload stream, not inventory metadata; every operation is denied.",
        &[(policy::DenyReason::Content, CONTENT)],
    )
}

const DEFERRED: &[&str] = &[];

const CONTENT: &[&str] = &["GetMedia"];
