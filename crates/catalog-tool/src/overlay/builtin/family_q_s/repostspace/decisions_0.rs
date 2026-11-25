use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchAddChannelRoleToAccessors",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("BatchAddRole", Decision::Denied(DenyReason::Mutating)),
    (
        "BatchRemoveChannelRoleFromAccessors",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("BatchRemoveRole", Decision::Denied(DenyReason::Mutating)),
    ("CreateChannel", Decision::Denied(DenyReason::Mutating)),
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    ("DeleteSpace", Decision::Denied(DenyReason::Mutating)),
    ("DeregisterAdmin", Decision::Denied(DenyReason::Mutating)),
    (
        "GetChannel",
        Decision::Deferred(
            "Required selectors spaceId, channelId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, channelId, channelName, channelDescription, createDateTime, deleteDateTime, channelRoles, channelStatus. Pinned purpose: Displays information about a channel in a private re:Post.",
        ),
    ),
    (
        "GetSpace",
        Decision::Deferred(
            "Required selectors spaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, arn, name, status, configurationStatus, clientId, identityStoreId, applicationArn, description, vanityDomainStatus, vanityDomain, randomDomain, customerRoleArn, createDateTime, deleteDateTime, tier, storageLimit, userAdmins, groupAdmins, roles, userKMSKey, userCount, contentSize, supportedEmailDomains. Pinned purpose: Displays information about the AWS re:Post Private private re:Post.",
        ),
    ),
    (
        "ListChannels",
        Decision::Deferred(
            "Required selectors spaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members channels, nextToken. Pinned purpose: Returns the list of channel within a private re:Post with some information about each channel.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns the tags that are associated with the AWS re:Post Private resource specified by the resourceArn. The only resource that can be tagged is a private re:Post.",
        ),
    ),
    ("RegisterAdmin", Decision::Denied(DenyReason::Mutating)),
    ("SendInvites", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateChannel", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
];
