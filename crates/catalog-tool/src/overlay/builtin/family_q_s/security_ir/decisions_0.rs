use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchGetMemberAccountDetails",
        Decision::Deferred(
            "Required selectors membershipId, accountIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members items, errors. Pinned purpose: Provides information on whether the supplied account IDs are associated with a membership. AWS account ID's may appear less than 12 characters and need to be zero-prepended. An example would be 123123123 which is nine digits, and with zero-prepend wo",
        ),
    ),
    ("CancelMembership", Decision::Denied(DenyReason::Mutating)),
    (
        "CloseCase",
        Decision::Deferred(
            "Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members caseStatus, closedDate. Pinned purpose: Closes an existing case.",
        ),
    ),
    ("CreateCase", Decision::Denied(DenyReason::Mutating)),
    ("CreateCaseComment", Decision::Denied(DenyReason::Mutating)),
    ("CreateMembership", Decision::Denied(DenyReason::Mutating)),
    ("GetCase", Decision::Denied(DenyReason::Content)),
    (
        "GetCaseAttachmentDownloadUrl",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetCaseAttachmentUploadUrl",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetFindingMetrics",
        Decision::Deferred(
            "Required selectors membershipId, startDate, endDate are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findingsIngestedSecurityHub, findingsIngestedGuardDuty, findingsTriaged, findingsTriagedFalsePositive, findingsInvestigated, findingsInvestigatedFalsePositive, findingsEscalated, findingsEscalatedFalsePositive, findingsTruePositive, findingsInvestigatedInProgress, findingsEscalatedInProgress. Pinned purpose: Returns finding-lifecycle metrics for a membership over a date range.",
        ),
    ),
    (
        "GetMembership",
        Decision::Deferred(
            "Required selectors membershipId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members membershipId, accountId, region, membershipName, membershipArn, membershipStatus, membershipActivationTimestamp, membershipDeactivationTimestamp, customerType, numberOfAccountsCovered, incidentResponseTeam, optInFeatures, membershipAccountsConfigurations. Pinned purpose: Returns the attributes of a membership.",
        ),
    ),
    (
        "ListCaseEdits",
        Decision::Deferred(
            "Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items, total. Pinned purpose: Views the case history for edits made to a designated case.",
        ),
    ),
    ("ListCases", Decision::Denied(DenyReason::Content)),
    (
        "ListComments",
        Decision::Deferred(
            "Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items, total. Pinned purpose: Returns comments for a designated case.",
        ),
    ),
    (
        "ListInvestigations",
        Decision::Deferred(
            "Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, investigationActions. Pinned purpose: Investigation performed by an agent for a security incident...",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns currently configured tags on a resource.",
        ),
    ),
    ("SendFeedback", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCase", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCaseComment", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCaseStatus", Decision::Denied(DenyReason::Mutating)),
    ("UpdateMembership", Decision::Denied(DenyReason::Mutating)),
    ("UpdateResolverType", Decision::Denied(DenyReason::Mutating)),
];
