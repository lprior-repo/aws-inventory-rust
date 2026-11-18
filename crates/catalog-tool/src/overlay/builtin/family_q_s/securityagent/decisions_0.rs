// Pinned operation purposes: .local/aws-models/models/securityagent/service/2025-09-06/securityagent-2025-09-06.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Uploads an artifact to an agent space. Artifacts provide additional context for security testing, such as architecture diagrams, API specifications, or configuration files.
    ("AddArtifact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Batch creates security requirements in a customer managed pack.
    (
        "BatchCreateSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes one or more code reviews from an agent space.
    (
        "BatchDeleteCodeReviews",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes one or more pentests from an agent space.
    (
        "BatchDeletePentests",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Batch deletes security requirements from a customer managed pack.
    (
        "BatchDeleteSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes one or more threat models from an agent space.
    (
        "BatchDeleteThreatModels",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a batch of findings from their linked issues.
    (
        "BatchDisassociateFindingIssue",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Exports a batch of findings to the connected issue tracker as issues.
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
    // Changes service state rather than reading inventory. Pinned purpose: Batch updates security requirements within a customer managed pack.
    (
        "BatchUpdateSecurityRequirements",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new agent space. An agent space is a dedicated workspace for securing a specific application.
    ("CreateAgentSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new application. An application is the top-level organizational unit that supports IAM Identity Center integration.
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new code review configuration in an agent space. A code review defines the parameters for automated security-focused code analysis.
    ("CreateCodeReview", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new integration with a third-party provider, such as GitHub, for code review and remediation.
    ("CreateIntegration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new membership, granting a user or group access to an agent space within an application.
    ("CreateMembership", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new pentest configuration in an agent space. A pentest defines the security test parameters, including target assets, risk type exclusions, and logging configuration.
    ("CreatePentest", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a private connection for reaching a self-hosted provider instance over private networking using Amazon VPC Lattice.
    (
        "CreatePrivateConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a customer managed security requirement pack.
    (
        "CreateSecurityRequirementPack",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new target domain for penetration testing. A target domain is a web domain that must be registered and verified before it can be tested.
    ("CreateTargetDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new threat under a threat model job.
    ("CreateThreat", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new threat model configuration in an agent space. A threat model defines the parameters for automated threat analysis.
    ("CreateThreatModel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an agent space and all of its associated resources, including pentests, findings, and artifacts.
    ("DeleteAgentSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an application and its associated configuration, including IAM Identity Center settings.
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an artifact from an agent space.
    ("DeleteArtifact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an integration with a third-party provider.
    ("DeleteIntegration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a membership, revoking a user's or group's access to an agent space.
    ("DeleteMembership", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a private connection.
    (
        "DeletePrivateConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a customer managed security requirement pack and all its associated security requirements.
    (
        "DeleteSecurityRequirementPack",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a target domain registration. After deletion, the domain can no longer be used for penetration testing.
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
    // Changes service state rather than reading inventory. Pinned purpose: Imports security requirements from uploaded documents into a customer managed security requirement pack. The import process asynchronously extracts and generates structured security requirements from the provided source files.
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
