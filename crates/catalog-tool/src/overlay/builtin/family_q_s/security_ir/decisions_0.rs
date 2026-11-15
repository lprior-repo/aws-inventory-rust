// Pinned operation purposes: .local/aws-models/models/security-ir/service/2018-05-10/security-ir-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("BatchGetMemberAccountDetails", Decision::Deferred("Required selectors membershipId, accountIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members items, errors. Pinned purpose: Provides information on whether the supplied account IDs are associated with a membership. AWS account ID's may appear less than 12 characters and need to be zero-prepended. An example would be 123123123 which is nine digits, and with zero-prepend wo")),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels an existing membership.
    ("CancelMembership", Decision::Denied(DenyReason::Mutating)),
    ("CloseCase", Decision::Deferred("Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members caseStatus, closedDate. Pinned purpose: Closes an existing case.")),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new case.
    ("CreateCase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a comment to an existing case.
    ("CreateCaseComment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new membership.
    ("CreateMembership", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the attributes of a case.
    ("GetCase", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns a Pre-Signed URL for uploading attachments into a case.
    ("GetCaseAttachmentDownloadUrl", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Uploads an attachment to a case.
    ("GetCaseAttachmentUploadUrl", Decision::Denied(DenyReason::Content)),
    ("GetFindingMetrics", Decision::Deferred("Required selectors membershipId, startDate, endDate are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findingsIngestedSecurityHub, findingsIngestedGuardDuty, findingsTriaged, findingsTriagedFalsePositive, findingsInvestigated, findingsInvestigatedFalsePositive, findingsEscalated, findingsEscalatedFalsePositive, findingsTruePositive, findingsInvestigatedInProgress, findingsEscalatedInProgress. Pinned purpose: Returns finding-lifecycle metrics for a membership over a date range.")),
    ("GetMembership", Decision::Deferred("Required selectors membershipId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members membershipId, accountId, region, membershipName, membershipArn, membershipStatus, membershipActivationTimestamp, membershipDeactivationTimestamp, customerType, numberOfAccountsCovered, incidentResponseTeam, optInFeatures, membershipAccountsConfigurations. Pinned purpose: Returns the attributes of a membership.")),
    ("ListCaseEdits", Decision::Deferred("Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items, total. Pinned purpose: Views the case history for edits made to a designated case.")),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Lists all cases the requester has access to.
    ("ListCases", Decision::Denied(DenyReason::Content)),
    ("ListComments", Decision::Deferred("Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items, total. Pinned purpose: Returns comments for a designated case.")),
    ("ListInvestigations", Decision::Deferred("Required selectors caseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, investigationActions. Pinned purpose: Investigation performed by an agent for a security incident...")),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns currently configured tags on a resource.")),
    // Changes service state rather than reading inventory. Pinned purpose: Send feedback based on response investigation action
    ("SendFeedback", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a tag(s) to a designated resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag(s) from a designate resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing case.
    ("UpdateCase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing case comment.
    ("UpdateCaseComment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the state transitions for a designated cases. Self-managed : the following states are available for self-managed cases. Submitted → Detection and Analysis Detection and Analysis → Containment, Eradication, and Recovery Detection and Analysis → Post-incident Activities Containment, Eradication, and Recovery → Detection and Analysis Containment, Eradication, and Recovery → Post-incident Acti
    ("UpdateCaseStatus", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates membership configuration.
    ("UpdateMembership", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resolver type for a case. This is a one-way action and cannot be reversed.
    ("UpdateResolverType", Decision::Denied(DenyReason::Mutating)),
];
