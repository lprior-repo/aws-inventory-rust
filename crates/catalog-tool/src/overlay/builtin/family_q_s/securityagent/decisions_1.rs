use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListDiscoveredEndpoints",
        Decision::Deferred(
            "Required selectors pentestJobId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members discoveredEndpoints, nextToken. Pinned purpose: Returns a paginated list of endpoints discovered during a pentest job execution.",
        ),
    ),
    (
        "ListFindings",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findingsSummaries, nextToken. Pinned purpose: Lists the security findings for a pentest job.",
        ),
    ),
    (
        "ListIntegratedResources",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members integratedResourceSummaries, nextToken. Pinned purpose: Lists the integrated resources for an agent space, optionally filtered by integration or resource type.",
        ),
    ),
    (
        "ListMemberships",
        Decision::Deferred(
            "Required selectors applicationId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members membershipSummaries, nextToken. Pinned purpose: Returns a paginated list of membership summaries for the specified agent space within an application.",
        ),
    ),
    (
        "ListPentestJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members taskSummaries, nextToken. Pinned purpose: Returns a paginated list of task summaries for the specified pentest job, optionally filtered by step name or category.",
        ),
    ),
    (
        "ListPentestJobsForPentest",
        Decision::Deferred(
            "Required selectors pentestId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentestJobSummaries, nextToken. Pinned purpose: Returns a paginated list of pentest job summaries for the specified pentest configuration.",
        ),
    ),
    (
        "ListPentests",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentestSummaries, nextToken. Pinned purpose: Returns a paginated list of pentest summaries for the specified agent space.",
        ),
    ),
    (
        "ListSecurityRequirements",
        Decision::Deferred(
            "Required selectors packId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members securityRequirementSummaries, nextToken. Pinned purpose: Lists security requirements within a pack.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns the tags associated with the specified resource.",
        ),
    ),
    (
        "ListThreatModelJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId, threatModelJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobTaskSummaries, nextToken. Pinned purpose: Returns a paginated list of task summaries for the specified threat model job.",
        ),
    ),
    (
        "ListThreatModelJobs",
        Decision::Deferred(
            "Required selectors threatModelId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobSummaries, nextToken. Pinned purpose: Returns a paginated list of threat model job summaries for the specified threat model.",
        ),
    ),
    (
        "ListThreatModels",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelSummaries, nextToken. Pinned purpose: Returns a paginated list of threat model summaries for the specified agent space.",
        ),
    ),
    (
        "ListThreats",
        Decision::Deferred(
            "Required selectors threatJobId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threats, nextToken. Pinned purpose: Returns a paginated list of threats for a threat model job.",
        ),
    ),
    (
        "StartCodeRemediation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartCodeReviewJob", Decision::Denied(DenyReason::Mutating)),
    ("StartPentestJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StartThreatModelJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopCodeReviewJob", Decision::Denied(DenyReason::Mutating)),
    ("StopPentestJob", Decision::Denied(DenyReason::Mutating)),
    ("StopThreatModelJob", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAgentSpace", Decision::Denied(DenyReason::Mutating)),
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCodeReview", Decision::Denied(DenyReason::Mutating)),
    ("UpdateFinding", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateIntegratedResources",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateIntegration", Decision::Denied(DenyReason::Mutating)),
    ("UpdatePentest", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdatePrivateConnectionCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSecurityRequirementPack",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTargetDomain", Decision::Denied(DenyReason::Mutating)),
    ("UpdateThreat", Decision::Denied(DenyReason::Mutating)),
    ("UpdateThreatModel", Decision::Denied(DenyReason::Mutating)),
    (
        "VerifyTargetDomain",
        Decision::Deferred(
            "Required selectors targetDomainId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members targetDomainId, domainName, createdAt, updatedAt, verifiedAt, status, verificationStatusReason. Pinned purpose: Initiates verification of a target domain. This checks whether the domain ownership verification token has been properly configured.",
        ),
    ),
];
