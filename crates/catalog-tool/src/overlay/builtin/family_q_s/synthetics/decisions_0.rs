use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AssociateResource", Decision::Denied(DenyReason::Mutating)),
    ("CreateCanary", Decision::Denied(DenyReason::Mutating)),
    ("CreateGroup", Decision::Denied(DenyReason::Mutating)),
    ("DeleteCanary", Decision::Denied(DenyReason::Mutating)),
    ("DeleteGroup", Decision::Denied(DenyReason::Mutating)),
    ("DescribeCanaries", Decision::Denied(DenyReason::Content)),
    (
        "DescribeCanariesLastRun",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "DescribeRuntimeVersions",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DisassociateResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("GetCanary", Decision::Denied(DenyReason::Content)),
    ("GetCanaryRuns", Decision::Denied(DenyReason::Content)),
    (
        "GetGroup",
        Decision::Deferred(
            "Required selectors GroupIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Group. Pinned purpose: Returns information about one group. Groups are a global resource, so you can use this operation from any Region.",
        ),
    ),
    (
        "ListAssociatedGroups",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Groups, NextToken. Pinned purpose: Returns a list of the groups that the specified canary is associated with. The canary that you specify must be in the current Region.",
        ),
    ),
    (
        "ListGroupResources",
        Decision::Deferred(
            "Required selectors GroupIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Resources, NextToken. Pinned purpose: This operation returns a list of the ARNs of the canaries that are associated with the specified group.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Displays the tags associated with a canary or group.",
        ),
    ),
    ("StartCanary", Decision::Denied(DenyReason::Mutating)),
    ("StartCanaryDryRun", Decision::Denied(DenyReason::Mutating)),
    ("StopCanary", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCanary", Decision::Denied(DenyReason::Mutating)),
];
