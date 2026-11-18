//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/ivschat/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListRooms",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "rooms",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListLoggingConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "loggingConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
    ];
    policy::finish(
        "ivschat",
        approved,
        DEFERRED,
        "Room and logging-configuration summaries are metadata. Messaging, user disconnect and token creation mutate/disclose capabilities. Remaining detail paths are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &["GetLoggingConfiguration", "GetRoom", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "CreateChatToken",
    "CreateLoggingConfiguration",
    "CreateRoom",
    "DeleteLoggingConfiguration",
    "DeleteMessage",
    "DeleteRoom",
    "DisconnectUser",
    "SendEvent",
    "TagResource",
    "UntagResource",
    "UpdateLoggingConfiguration",
    "UpdateRoom",
];
