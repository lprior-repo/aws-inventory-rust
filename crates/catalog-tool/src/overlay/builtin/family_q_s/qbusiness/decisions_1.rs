use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListMessages",
        Decision::Deferred(
            "Required selectors conversationId, applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messages, nextToken. Pinned purpose: Gets a list of messages associated with an Amazon Q Business web experience.",
        ),
    ),
    (
        "ListPluginActions",
        Decision::Deferred(
            "Required selectors applicationId, pluginId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items. Pinned purpose: Lists configured Amazon Q Business actions for a specific plugin in an Amazon Q Business application.",
        ),
    ),
    (
        "ListPluginTypeActions",
        Decision::Deferred(
            "Required selectors pluginType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items. Pinned purpose: Lists configured Amazon Q Business actions for any plugin type—both built-in and custom.",
        ),
    ),
    (
        "ListPluginTypeMetadata",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListSubscriptions",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, subscriptions. Pinned purpose: Lists all subscriptions created in an Amazon Q Business application.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceARN. Pinned purpose: Gets a list of tags associated with a specified resource. Amazon Q Business applications and data sources can have tags associated with them.",
        ),
    ),
    ("PutFeedback", Decision::Denied(DenyReason::Mutating)),
    ("PutGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "SearchRelevantContent",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "StartDataSourceSyncJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopDataSourceSyncJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateChatControlsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateChatResponseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDataAccessor", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDataSource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateIndex", Decision::Denied(DenyReason::Mutating)),
    ("UpdatePlugin", Decision::Denied(DenyReason::Mutating)),
    ("UpdateRetriever", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSubscription", Decision::Denied(DenyReason::Mutating)),
    ("UpdateUser", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateWebExperience",
        Decision::Denied(DenyReason::Mutating),
    ),
];
