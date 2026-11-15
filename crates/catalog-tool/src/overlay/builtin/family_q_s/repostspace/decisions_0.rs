// Pinned operation purposes: .local/aws-models/models/repostspace/service/2022-05-13/repostspace-2022-05-13.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Add role to multiple users or groups in a private re:Post channel.
    ("BatchAddChannelRoleToAccessors", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Add a role to multiple users or groups in a private re:Post.
    ("BatchAddRole", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Remove a role from multiple users or groups in a private re:Post channel.
    ("BatchRemoveChannelRoleFromAccessors", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Remove a role from multiple users or groups in a private re:Post.
    ("BatchRemoveRole", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a channel in an AWS re:Post Private private re:Post.
    ("CreateChannel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an AWS re:Post Private private re:Post.
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an AWS re:Post Private private re:Post.
    ("DeleteSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the user or group from the list of administrators of the private re:Post.
    ("DeregisterAdmin", Decision::Denied(DenyReason::Mutating)),
    ("GetChannel", Decision::Deferred("Required selectors spaceId, channelId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, channelId, channelName, channelDescription, createDateTime, deleteDateTime, channelRoles, channelStatus. Pinned purpose: Displays information about a channel in a private re:Post.")),
    ("GetSpace", Decision::Deferred("Required selectors spaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, arn, name, status, configurationStatus, clientId, identityStoreId, applicationArn, description, vanityDomainStatus, vanityDomain, randomDomain, customerRoleArn, createDateTime, deleteDateTime, tier, storageLimit, userAdmins, groupAdmins, roles, userKMSKey, userCount, contentSize, supportedEmailDomains. Pinned purpose: Displays information about the AWS re:Post Private private re:Post.")),
    ("ListChannels", Decision::Deferred("Required selectors spaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members channels, nextToken. Pinned purpose: Returns the list of channel within a private re:Post with some information about each channel.")),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns the tags that are associated with the AWS re:Post Private resource specified by the resourceArn. The only resource that can be tagged is a private re:Post.")),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a user or group to the list of administrators of the private re:Post.
    ("RegisterAdmin", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Sends an invitation email to selected users and groups.
    ("SendInvites", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates tags with an AWS re:Post Private resource. Currently, the only resource that can be tagged is the private re:Post. If you specify a new tag key for the resource, the tag is appended to the list of tags that are associated with the resource. If you specify a tag key that’s already associated with the resource, the new tag value that you specify replaces the previous value for that tag.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the association of the tag with the AWS re:Post Private resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies an existing channel.
    ("UpdateChannel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies an existing AWS re:Post Private private re:Post.
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
];
