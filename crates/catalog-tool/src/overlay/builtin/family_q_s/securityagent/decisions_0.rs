use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AddArtifact", Decision::Denied(DenyReason::Mutating)),
    (
        "BatchCreateSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteCodeReviews",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeletePentests",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteThreatModels",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDisassociateFindingIssue",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchExportFindings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchGetAgentSpaces",
        Decision::Deferred(
            "Required selectors agentSpaceIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members agentSpaces, notFound. Pinned purpose: Retrieves information about one or more agent spaces.",
        ),
    ),
    (
        "BatchGetArtifactMetadata",
        Decision::Deferred(
            "Required selectors agentSpaceId, artifactIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members artifactMetadataList. Pinned purpose: Retrieves metadata for one or more artifacts in an agent space.",
        ),
    ),
    (
        "BatchGetCodeReviewJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId, codeReviewJobTaskIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviewJobTasks, notFound. Pinned purpose: Retrieves information about one or more tasks within a code review job.",
        ),
    ),
    (
        "BatchGetCodeReviewJobs",
        Decision::Deferred(
            "Required selectors codeReviewJobIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviewJobs, notFound. Pinned purpose: Retrieves information about one or more code review jobs in an agent space.",
        ),
    ),
    (
        "BatchGetCodeReviews",
        Decision::Deferred(
            "Required selectors codeReviewIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviews, notFound. Pinned purpose: Retrieves information about one or more code reviews in an agent space.",
        ),
    ),
    (
        "BatchGetFindings",
        Decision::Deferred(
            "Required selectors findingIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findings, notFound. Pinned purpose: Retrieves information about one or more security findings in an agent space.",
        ),
    ),
    (
        "BatchGetPentestJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId, taskIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tasks, notFound. Pinned purpose: Retrieves information about one or more tasks within a pentest job.",
        ),
    ),
    (
        "BatchGetPentestJobs",
        Decision::Deferred(
            "Required selectors pentestJobIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentestJobs, notFound. Pinned purpose: Retrieves information about one or more pentest jobs in an agent space.",
        ),
    ),
    (
        "BatchGetPentests",
        Decision::Deferred(
            "Required selectors pentestIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentests, notFound. Pinned purpose: Retrieves information about one or more pentests in an agent space.",
        ),
    ),
    (
        "BatchGetSecurityRequirements",
        Decision::Deferred(
            "Required selectors packId, securityRequirementNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members securityRequirements, errors. Pinned purpose: Batch retrieves security requirements from a pack.",
        ),
    ),
    (
        "BatchGetTargetDomains",
        Decision::Deferred(
            "Required selectors targetDomainIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members targetDomains, notFound. Pinned purpose: Retrieves information about one or more target domains.",
        ),
    ),
    (
        "BatchGetThreatModelJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId, threatModelJobTaskIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobTasks, notFound. Pinned purpose: Retrieves information about one or more tasks within a threat model job.",
        ),
    ),
    (
        "BatchGetThreatModelJobs",
        Decision::Deferred(
            "Required selectors threatModelJobIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobs, notFound. Pinned purpose: Retrieves information about one or more threat model jobs in an agent space.",
        ),
    ),
    (
        "BatchGetThreatModels",
        Decision::Deferred(
            "Required selectors threatModelIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModels, notFound. Pinned purpose: Retrieves information about one or more threat models in an agent space.",
        ),
    ),
    (
        "BatchGetThreats",
        Decision::Deferred(
            "Required selectors threatIds, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threats, notFound. Pinned purpose: Retrieves information about one or more threats.",
        ),
    ),
    (
        "BatchUpdateSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAgentSpace", Decision::Denied(DenyReason::Mutating)),
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    ("CreateCodeReview", Decision::Denied(DenyReason::Mutating)),
    ("CreateIntegration", Decision::Denied(DenyReason::Mutating)),
    ("CreateMembership", Decision::Denied(DenyReason::Mutating)),
    ("CreatePentest", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePrivateConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateSecurityRequirementPack",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTargetDomain", Decision::Denied(DenyReason::Mutating)),
    ("CreateThreat", Decision::Denied(DenyReason::Mutating)),
    ("CreateThreatModel", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAgentSpace", Decision::Denied(DenyReason::Mutating)),
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    ("DeleteArtifact", Decision::Denied(DenyReason::Mutating)),
    ("DeleteIntegration", Decision::Denied(DenyReason::Mutating)),
    ("DeleteMembership", Decision::Denied(DenyReason::Mutating)),
    (
        "DeletePrivateConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteSecurityRequirementPack",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTargetDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribePrivateConnection",
        Decision::Deferred(
            "Required selectors privateConnectionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members name, type, status, resourceGatewayId, hostAddress, vpcId, resourceConfigurationId, certificateExpiryTime, dnsResolution, failureMessage, tags. Pinned purpose: Retrieves the details of a private connection.",
        ),
    ),
    (
        "GetApplication",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, domain, applicationName, idcConfiguration, roleArn, defaultKmsKeyId. Pinned purpose: Retrieves information about an application.",
        ),
    ),
    (
        "GetArtifact",
        Decision::Deferred(
            "Required selectors agentSpaceId, artifactId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members agentSpaceId, artifactId, artifact, fileName, updatedAt. Pinned purpose: Retrieves an artifact from an agent space.",
        ),
    ),
    (
        "GetIntegration",
        Decision::Deferred(
            "Required selectors integrationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members integrationId, installationId, provider, providerType, displayName, kmsKeyId, targetUrl, siteUrl, webhookUrl, privateConnectionName. Pinned purpose: Retrieves information about an integration.",
        ),
    ),
    (
        "GetSecurityRequirementPack",
        Decision::Deferred(
            "Required selectors packId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members packId, name, description, vendorName, managementType, status, importStatus, createdAt, updatedAt, kmsKeyId. Pinned purpose: Retrieves information about a security requirement pack.",
        ),
    ),
    (
        "ImportSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "InitiateProviderRegistration",
        Decision::Deferred(
            "Required selectors provider are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members redirectTo, csrfState. Pinned purpose: Initiates the OAuth registration flow with a third-party provider. Returns a redirect URL and CSRF state token for completing the authorization.",
        ),
    ),
    (
        "ListActorMessages",
        Decision::Deferred(
            "Required selectors agentSpaceId, pentestId, actorIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messages, nextToken. Pinned purpose: Returns a paginated list of the email MFA messages received for an actor at its server-generated email address, most recent first.",
        ),
    ),
    (
        "ListArtifacts",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members artifactSummaries, nextToken. Pinned purpose: Returns a paginated list of artifact summaries for the specified agent space.",
        ),
    ),
    (
        "ListCodeReviewJobTasks",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviewJobTaskSummaries, nextToken. Pinned purpose: Returns a paginated list of task summaries for the specified code review job, optionally filtered by step name or category.",
        ),
    ),
    (
        "ListCodeReviewJobsForCodeReview",
        Decision::Deferred(
            "Required selectors codeReviewId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviewJobSummaries, nextToken. Pinned purpose: Returns a paginated list of code review job summaries for the specified code review configuration.",
        ),
    ),
    (
        "ListCodeReviews",
        Decision::Deferred(
            "Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members codeReviewSummaries, nextToken. Pinned purpose: Returns a paginated list of code review summaries for the specified agent space.",
        ),
    ),
];
