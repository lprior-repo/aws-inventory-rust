use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![(
        "ListStreams",
        policy::query(
            policy::enumerate(
                "streams",
                policy::arn("streamArn")?,
                Some(("nextToken", "nextToken")),
            )?,
            &["streamArn"],
        )?,
    )];
    policy::finish(
        "keyspacesstreams",
        approved,
        DEFERRED,
        "Only CDC stream metadata is enumerated; record retrieval and record-position capabilities are denied. Shard-level metadata detail is not yet planned.",
        &[(policy::DenyReason::Content, CONTENT)],
    )
}

const DEFERRED: &[&str] = &["GetStream"];

const CONTENT: &[&str] = &["GetRecords", "GetShardIterator"];
