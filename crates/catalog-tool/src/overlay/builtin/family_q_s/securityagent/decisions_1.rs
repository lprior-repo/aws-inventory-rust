// Pinned operation purposes: .local/aws-models/models/securityagent/service/2025-09-06/securityagent-2025-09-06.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("ListDiscoveredEndpoints", Decision::Deferred("Required selectors pentestJobId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members discoveredEndpoints, nextToken. Pinned purpose: Returns a paginated list of endpoints discovered during a pentest job execution.")),
    ("ListFindings", Decision::Deferred("Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findingsSummaries, nextToken. Pinned purpose: Lists the security findings for a pentest job.")),
    ("ListIntegratedResources", Decision::Deferred("Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members integratedResourceSummaries, nextToken. Pinned purpose: Lists the integrated resources for an agent space, optionally filtered by integration or resource type.")),
    ("ListMemberships", Decision::Deferred("Required selectors applicationId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members membershipSummaries, nextToken. Pinned purpose: Returns a paginated list of membership summaries for the specified agent space within an application.")),
    ("ListPentestJobTasks", Decision::Deferred("Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members taskSummaries, nextToken. Pinned purpose: Returns a paginated list of task summaries for the specified pentest job, optionally filtered by step name or category.")),
    ("ListPentestJobsForPentest", Decision::Deferred("Required selectors pentestId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentestJobSummaries, nextToken. Pinned purpose: Returns a paginated list of pentest job summaries for the specified pentest configuration.")),
    ("ListPentests", Decision::Deferred("Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members pentestSummaries, nextToken. Pinned purpose: Returns a paginated list of pentest summaries for the specified agent space.")),
    ("ListSecurityRequirements", Decision::Deferred("Required selectors packId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members securityRequirementSummaries, nextToken. Pinned purpose: Lists security requirements within a pack.")),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns the tags associated with the specified resource.")),
    ("ListThreatModelJobTasks", Decision::Deferred("Required selectors agentSpaceId, threatModelJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobTaskSummaries, nextToken. Pinned purpose: Returns a paginated list of task summaries for the specified threat model job.")),
    ("ListThreatModelJobs", Decision::Deferred("Required selectors threatModelId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelJobSummaries, nextToken. Pinned purpose: Returns a paginated list of threat model job summaries for the specified threat model.")),
    ("ListThreatModels", Decision::Deferred("Required selectors agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threatModelSummaries, nextToken. Pinned purpose: Returns a paginated list of threat model summaries for the specified agent space.")),
    ("ListThreats", Decision::Deferred("Required selectors threatJobId, agentSpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members threats, nextToken. Pinned purpose: Returns a paginated list of threats for a threat model job.")),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates code remediation for one or more security findings. This creates pull requests in integrated repositories to fix the identified vulnerabilities.
    ("StartCodeRemediation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a new code review job for a code review configuration. The job executes the security-focused code analysis defined in the code review.
    ("StartCodeReviewJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a new pentest job for a pentest configuration. The job executes the security tests defined in the pentest.
    ("StartPentestJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a new threat model job for a threat model configuration.
    ("StartThreatModelJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running code review job. The job transitions to a stopping state and then to stopped after cleanup completes.
    ("StopCodeReviewJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running pentest job. The job transitions to a stopping state and then to stopped after cleanup completes.
    ("StopPentestJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running threat model job.
    ("StopThreatModelJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds tags to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration of an existing agent space, including its name, description, AWS resources, target domains, and code review settings.
    ("UpdateAgentSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration of an existing application, including the IAM role and default KMS key.
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing code review configuration.
    ("UpdateCodeReview", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the status or risk level of a security finding.
    ("UpdateFinding", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the integrated resources for an agent space, including their capabilities.
    ("UpdateIntegratedResources", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an integration's webhook, or rotates the HMAC signing secret of an existing one. The secret is returned only once, in this response, and cannot be retrieved again.
    ("UpdateIntegration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing pentest configuration.
    ("UpdatePentest", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the certificate associated with a private connection. Certificates can be added or replaced but not removed.
    ("UpdatePrivateConnectionCertificate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a security requirement pack. For customer managed packs, both metadata and status can be updated. For AWS managed packs, only status can be updated.
    ("UpdateSecurityRequirementPack", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the verification method for a target domain.
    ("UpdateTargetDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a threat.
    ("UpdateThreat", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing threat model configuration.
    ("UpdateThreatModel", Decision::Denied(DenyReason::Mutating)),
    ("VerifyTargetDomain", Decision::Deferred("Required selectors targetDomainId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members targetDomainId, domainName, createdAt, updatedAt, verifiedAt, status, verificationStatusReason. Pinned purpose: Initiates verification of a target domain. This checks whether the domain ownership verification token has been properly configured.")),
];
