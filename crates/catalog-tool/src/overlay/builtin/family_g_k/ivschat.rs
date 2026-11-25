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
