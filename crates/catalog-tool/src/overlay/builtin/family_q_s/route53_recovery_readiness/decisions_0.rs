// Pinned operation purposes: .local/aws-models/models/route53-recovery-readiness/service/2019-12-02/route53-recovery-readiness-2019-12-02.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a cell in an account.
    ("CreateCell", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a cross-account readiness authorization. This lets you authorize another account to work with Route 53 Application Recovery Controller, for example, to check the readiness status of resources in a separate account.
    ("CreateCrossAccountAuthorization", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a readiness check in an account. A readiness check monitors a resource set in your application, such as a set of Amazon Aurora instances, that Application Recovery Controller is auditing recovery readiness for. The audits run once every minute on every resource that's associated with a readiness check.
    ("CreateReadinessCheck", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a recovery group in an account. A recovery group corresponds to an application and includes a list of the cells that make up the application.
    ("CreateRecoveryGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a resource set. A resource set is a set of resources of one type that span multiple cells. You can associate a resource set with a readiness check to monitor the resources for failover readiness.
    ("CreateResourceSet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a cell. When successful, the response code is 204, with no response body.
    ("DeleteCell", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes cross account readiness authorization.
    ("DeleteCrossAccountAuthorization", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a readiness check.
    ("DeleteReadinessCheck", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a recovery group.
    ("DeleteRecoveryGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a resource set.
    ("DeleteResourceSet", Decision::Denied(DenyReason::Mutating)),
    ("GetArchitectureRecommendations", Decision::Deferred("Required selectors RecoveryGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LastAuditTimestamp, NextToken, Recommendations. Pinned purpose: Gets recommendations about architecture designs for improving resiliency for an application, based on a recovery group.")),
    ("GetCell", Decision::Deferred("Required selectors CellName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CellArn, CellName, Cells, ParentReadinessScopes, Tags. Pinned purpose: Gets information about a cell including cell name, cell Amazon Resource Name (ARN), ARNs of nested cells for this cell, and a list of those cell ARNs with their associated recovery group ARNs.")),
    ("GetCellReadinessSummary", Decision::Deferred("Required selectors CellName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Readiness, ReadinessChecks. Pinned purpose: Gets readiness for a cell. Aggregates the readiness of all the resources that are associated with the cell into a single value.")),
    ("GetReadinessCheck", Decision::Deferred("Required selectors ReadinessCheckName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ReadinessCheckArn, ReadinessCheckName, ResourceSet, Tags. Pinned purpose: Gets details about a readiness check.")),
    ("GetReadinessCheckResourceStatus", Decision::Deferred("Required selectors ReadinessCheckName, ResourceIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Readiness, Rules. Pinned purpose: Gets individual readiness status for a readiness check. To see the overall readiness status for a recovery group, that considers the readiness status for all the readiness checks in the recovery group, use GetRecoveryGroupReadinessSummary.")),
    ("GetReadinessCheckStatus", Decision::Deferred("Required selectors ReadinessCheckName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Messages, NextToken, Readiness, Resources. Pinned purpose: Gets the readiness status for an individual readiness check. To see the overall readiness status for a recovery group, that considers the readiness status for all the readiness checks in a recovery group, use GetRecoveryGroupReadinessSummary.")),
    ("GetRecoveryGroup", Decision::Deferred("Required selectors RecoveryGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Cells, RecoveryGroupArn, RecoveryGroupName, Tags. Pinned purpose: Gets details about a recovery group, including a list of the cells that are included in it.")),
    ("GetRecoveryGroupReadinessSummary", Decision::Deferred("Required selectors RecoveryGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Readiness, ReadinessChecks. Pinned purpose: Displays a summary of information about a recovery group's readiness status. Includes the readiness checks for resources in the recovery group and the readiness status of each one.")),
    ("GetResourceSet", Decision::Deferred("Required selectors ResourceSetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceSetArn, ResourceSetName, ResourceSetType, Resources, Tags. Pinned purpose: Displays the details about a resource set, including a list of the resources in the set.")),
    ("ListCells", Decision::Deferred("route53-recovery-readiness has fixed control-plane routing (us-west-2), while the pinned endpoint rules are parameterized by Region and Account/HomeRegion uses caller home. Requires service-specific fixed routing; regional fan-out would misstate scope or fail.")),
    ("ListCrossAccountAuthorizations", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members CrossAccountAuthorizations, NextToken. Pinned request/ownership contract: Lists the cross-account readiness authorizations that are in place for an account.")),
    ("ListReadinessChecks", Decision::Deferred("route53-recovery-readiness has fixed control-plane routing (us-west-2), while the pinned endpoint rules are parameterized by Region and Account/HomeRegion uses caller home. Requires service-specific fixed routing; regional fan-out would misstate scope or fail.")),
    ("ListRecoveryGroups", Decision::Deferred("route53-recovery-readiness has fixed control-plane routing (us-west-2), while the pinned endpoint rules are parameterized by Region and Account/HomeRegion uses caller home. Requires service-specific fixed routing; regional fan-out would misstate scope or fail.")),
    ("ListResourceSets", Decision::Deferred("route53-recovery-readiness has fixed control-plane routing (us-west-2), while the pinned endpoint rules are parameterized by Region and Account/HomeRegion uses caller home. Requires service-specific fixed routing; regional fan-out would misstate scope or fail.")),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists all readiness rules, or lists the readiness rules for a specific resource type.
    ("ListRules", Decision::Denied(DenyReason::ReferenceData)),
    ("ListTagsForResources", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags for a resource.")),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a tag to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a cell to replace the list of nested cells with a new list of nested cells.
    ("UpdateCell", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a readiness check.
    ("UpdateReadinessCheck", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a recovery group.
    ("UpdateRecoveryGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a resource set.
    ("UpdateResourceSet", Decision::Denied(DenyReason::Mutating)),
];
