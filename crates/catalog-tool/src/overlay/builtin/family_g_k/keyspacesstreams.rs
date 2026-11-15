//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/keyspacesstreams/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListStreams", policy::query(policy::enumerate("streams", policy::arn("streamArn")?, Some(("nextToken", "nextToken")))?, &["streamArn"])?),
    ];
    policy::finish(
        "keyspacesstreams", approved, DEFERRED,
        "Only CDC stream metadata is enumerated; record retrieval and record-position capabilities are denied. Shard-level metadata detail is not yet planned.",
        &[(policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetStream",
];

const CONTENT: &[&str] = &[
    "GetRecords", "GetShardIterator",
];
