// Pinned operation purposes: .local/aws-models/models/sso-admin/service/2020-07-20/sso-admin-2020-07-20.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListPermissionSetProvisioningStatus",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PermissionSetsProvisioningStatus, NextToken. Pinned purpose: Lists the status of the permission set provisioning requests for a specified IAM Identity Center instance.",
        ),
    ),
    (
        "ListPermissionSetsProvisionedToAccount",
        Decision::Deferred(
            "Required selectors InstanceArn, AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, PermissionSets. Pinned purpose: Lists all the permission sets that are provisioned to a specified Amazon Web Services account.",
        ),
    ),
    (
        "ListRegions",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Regions, NextToken. Pinned purpose: Lists all enabled Regions of an IAM Identity Center instance, including those that are being added or removed. This operation returns Regions with ACTIVE, ADDING, or REMOVING status. The following actions are related to ListRegions : AddRegion Remove",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags that are attached to a specified resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: The process by which a specified permission set is provisioned to the specified target.
    (
        "ProvisionPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or updates the list of authorized targets for an IAM Identity Center access scope for an application.
    (
        "PutApplicationAccessScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Configure how users gain access to an application. If AssignmentsRequired is true (default value), users don’t have access to the application unless an assignment is created using the CreateApplicationAssignment API . If false , all users have access to the application. If an assignment is created using CreateApplicationAssignment ., the user retains access if AssignmentsRequired is set to true .
    (
        "PutApplicationAssignmentConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or updates an authentication method for an application.
    (
        "PutApplicationAuthenticationMethod",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a configuration for an application to use grants. Conceptually grants are authorization to request actions related to tokens. This configuration will be used when parties are requesting and receiving tokens during the trusted identity propagation process. For more information on the IAM Identity Center supported grant workflows, see SAML 2.0 and OAuth 2.0 . A grant is created between your
    (
        "PutApplicationGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the session configuration for an application in IAM Identity Center. The session configuration determines how users can access an application. This includes whether user background sessions are enabled. User background sessions allow users to start a job on a supported Amazon Web Services managed application without having to remain signed in to an active session while the job runs.
    (
        "PutApplicationSessionConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Attaches an inline policy to a permission set. If the permission set is already referenced by one or more account assignments, you will need to call ProvisionPermissionSet after this action to apply the corresponding IAM policy updates to all assigned accounts.
    (
        "PutInlinePolicyToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Attaches an Amazon Web Services managed or customer managed policy to the specified PermissionSet as a permissions boundary.
    (
        "PutPermissionsBoundaryToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes an additional Region from an IAM Identity Center instance. This operation initiates an asynchronous workflow to clean up IAM Identity Center resources in the specified additional Region. The Region status is set to REMOVING and the Region record is deleted when the workflow completes. The request must be made from the primary Region. The target Region cannot be the primary Region, and no o
    ("RemoveRegion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates a set of tags with a specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a set of tags from a specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates application properties.
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Update the details for the instance of IAM Identity Center that is owned by the Amazon Web Services account. In a single UpdateInstance request, you can perform only one of the following operations: Update the encryption configuration of the instance by specifying EncryptionConfiguration . Enable permission sets for the instance by specifying PermissionSetsEnabled . Update the network configuratio
    ("UpdateInstance", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the IAM Identity Center identity store attributes that you can use with the IAM Identity Center instance for attributes-based access control (ABAC). When using an external identity provider as an identity source, you can pass attributes through the SAML assertion as an alternative to configuring attributes from the IAM Identity Center identity store. If a SAML assertion passes any of these
    (
        "UpdateInstanceAccessControlAttributeConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing permission set.
    (
        "UpdatePermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the name of the trusted token issuer, or the path of a source attribute or destination attribute for a trusted token issuer configuration. Updating this trusted token issuer configuration might cause users to lose access to any applications that are configured to use the trusted token issuer.
    (
        "UpdateTrustedTokenIssuer",
        Decision::Denied(DenyReason::Mutating),
    ),
];
