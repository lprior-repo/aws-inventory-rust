use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AddRegion", Decision::Denied(DenyReason::Mutating)),
    (
        "AttachCustomerManagedPolicyReferenceToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AttachManagedPolicyToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAccountAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateApplicationAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateInstanceAccessControlAttributeConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateTrustedTokenIssuer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccountAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteApplicationAccessScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteApplicationAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteApplicationAuthenticationMethod",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteApplicationGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteInlinePolicyFromPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteInstanceAccessControlAttributeConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeletePermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeletePermissionsBoundaryFromPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteTrustedTokenIssuer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeAccountAssignmentCreationStatus",
        Decision::Deferred(
            "Required selectors InstanceArn, AccountAssignmentCreationRequestId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignmentCreationStatus. Pinned purpose: Describes the status of the assignment creation request.",
        ),
    ),
    (
        "DescribeAccountAssignmentDeletionStatus",
        Decision::Deferred(
            "Required selectors InstanceArn, AccountAssignmentDeletionRequestId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignmentDeletionStatus. Pinned purpose: Describes the status of the assignment deletion request.",
        ),
    ),
    (
        "DescribeApplication",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationArn, ApplicationProviderArn, Name, ApplicationAccount, InstanceArn, IdentityStoreArn, Status, PortalOptions, Description, CreatedDate, CreatedFrom. Pinned purpose: Retrieves the details of an application associated with an instance of IAM Identity Center.",
        ),
    ),
    (
        "DescribeApplicationAssignment",
        Decision::Deferred(
            "Required selectors ApplicationArn, PrincipalId, PrincipalType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PrincipalType, PrincipalId, ApplicationArn. Pinned purpose: Retrieves a direct assignment of a user or group to an application. If the user doesn’t have a direct assignment to the application, the user may still have access to the application through a group. Therefore, don’t use this API to test access to an",
        ),
    ),
    (
        "DescribeApplicationProvider",
        Decision::Deferred(
            "Required selectors ApplicationProviderArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationProviderArn, FederationProtocol, DisplayData, ResourceServerConfig. Pinned purpose: Retrieves details about a provider that can be used to connect an Amazon Web Services managed application or customer managed application to IAM Identity Center.",
        ),
    ),
    (
        "DescribeInstance",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InstanceArn, IdentityStoreId, IdentityStoreArn, OwnerAccountId, Name, CreatedDate, Status, StatusReason, PrimaryRegion, Regions, EncryptionConfigurationDetails, NetworkConfigurationDetails, PermissionSetsEnabled. Pinned purpose: Returns the details of an instance of IAM Identity Center. The status can be one of the following: CREATE_IN_PROGRESS - The instance is in the process of being created. When the instance is ready for use, DescribeInstance returns the status of ACTIVE",
        ),
    ),
    (
        "DescribeInstanceAccessControlAttributeConfiguration",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, StatusReason, InstanceAccessControlAttributeConfiguration. Pinned purpose: Returns the list of IAM Identity Center identity store attributes that have been configured to work with attributes-based access control (ABAC) for the specified IAM Identity Center instance. This will not return attributes configured and sent by an ",
        ),
    ),
    (
        "DescribePermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PermissionSet. Pinned purpose: Gets the details of the permission set.",
        ),
    ),
    (
        "DescribePermissionSetProvisioningStatus",
        Decision::Deferred(
            "Required selectors InstanceArn, ProvisionPermissionSetRequestId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PermissionSetProvisioningStatus. Pinned purpose: Describes the status for the given permission set provisioning request.",
        ),
    ),
    (
        "DescribeRegion",
        Decision::Deferred(
            "Required selectors InstanceArn, RegionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RegionName, Status, AddedDate, IsPrimaryRegion. Pinned purpose: Retrieves details about a specific Region enabled in an IAM Identity Center instance. Details include the Region name, current status (ACTIVE, ADDING, or REMOVING), the date when the Region was added, and whether it is the primary Region. The request",
        ),
    ),
    (
        "DescribeTrustedTokenIssuer",
        Decision::Deferred(
            "Required selectors TrustedTokenIssuerArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrustedTokenIssuerArn, Name, TrustedTokenIssuerType, TrustedTokenIssuerConfiguration. Pinned purpose: Retrieves details about a trusted token issuer configuration stored in an instance of IAM Identity Center. Details include the name of the trusted token issuer, the issuer URL, and the path of the source attribute and the destination attribute for a ",
        ),
    ),
    (
        "DetachCustomerManagedPolicyReferenceFromPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DetachManagedPolicyFromPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetApplicationAccessScope",
        Decision::Deferred(
            "Required selectors ApplicationArn, Scope are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Scope, AuthorizedTargets. Pinned purpose: Retrieves the authorized targets for an IAM Identity Center access scope for an application.",
        ),
    ),
    (
        "GetApplicationAssignmentConfiguration",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AssignmentRequired. Pinned purpose: Retrieves the configuration of PutApplicationAssignmentConfiguration .",
        ),
    ),
    (
        "GetApplicationAuthenticationMethod",
        Decision::Deferred(
            "Required selectors ApplicationArn, AuthenticationMethodType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AuthenticationMethod. Pinned purpose: Retrieves details about an authentication method used by an application.",
        ),
    ),
    (
        "GetApplicationGrant",
        Decision::Deferred(
            "Required selectors ApplicationArn, GrantType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Grant. Pinned purpose: Retrieves details about an application grant.",
        ),
    ),
    (
        "GetApplicationSessionConfiguration",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UserBackgroundSessionApplicationStatus. Pinned purpose: Retrieves the session configuration for an application in IAM Identity Center. The session configuration determines how users can access an application. This includes whether user background sessions are enabled. User background sessions allow users ",
        ),
    ),
    (
        "GetInlinePolicyForPermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InlinePolicy. Pinned purpose: Obtains the inline policy assigned to the permission set.",
        ),
    ),
    (
        "GetPermissionsBoundaryForPermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PermissionsBoundary. Pinned purpose: Obtains the permissions boundary for a specified PermissionSet .",
        ),
    ),
    (
        "ListAccountAssignmentCreationStatus",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignmentsCreationStatus, NextToken. Pinned purpose: Lists the status of the Amazon Web Services account assignment creation requests for a specified IAM Identity Center instance.",
        ),
    ),
    (
        "ListAccountAssignmentDeletionStatus",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignmentsDeletionStatus, NextToken. Pinned purpose: Lists the status of the Amazon Web Services account assignment deletion requests for a specified IAM Identity Center instance.",
        ),
    ),
    (
        "ListAccountAssignments",
        Decision::Deferred(
            "Required selectors InstanceArn, AccountId, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignments, NextToken. Pinned purpose: Lists the assignee of the specified Amazon Web Services account with the specified permission set.",
        ),
    ),
    (
        "ListAccountAssignmentsForPrincipal",
        Decision::Deferred(
            "Required selectors InstanceArn, PrincipalId, PrincipalType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountAssignments, NextToken. Pinned purpose: Retrieves a list of the IAM Identity Center associated Amazon Web Services accounts that the principal has access to. This action must be called from the management account containing your organization instance of IAM Identity Center. This action is ",
        ),
    ),
    (
        "ListAccountsForProvisionedPermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountIds, NextToken. Pinned purpose: Lists all the Amazon Web Services accounts where the specified permission set is provisioned.",
        ),
    ),
    (
        "ListApplicationAccessScopes",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Scopes, NextToken. Pinned purpose: Lists the access scopes and authorized targets associated with an application.",
        ),
    ),
    (
        "ListApplicationAssignments",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationAssignments, NextToken. Pinned purpose: Lists Amazon Web Services account users that are assigned to an application.",
        ),
    ),
    (
        "ListApplicationAssignmentsForPrincipal",
        Decision::Deferred(
            "Required selectors InstanceArn, PrincipalId, PrincipalType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationAssignments, NextToken. Pinned purpose: Lists the applications to which a specified principal is assigned. You must provide a filter when calling this action from a member account against your organization instance of IAM Identity Center. A filter is not required when called from the manag",
        ),
    ),
    (
        "ListApplicationAuthenticationMethods",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AuthenticationMethods, NextToken. Pinned purpose: Lists all of the authentication methods supported by the specified application.",
        ),
    ),
    (
        "ListApplicationGrants",
        Decision::Deferred(
            "Required selectors ApplicationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Grants, NextToken. Pinned purpose: List the grants associated with an application.",
        ),
    ),
    (
        "ListApplicationProviders",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListCustomerManagedPolicyReferencesInPermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CustomerManagedPolicyReferences, NextToken. Pinned purpose: Lists all customer managed policies attached to a specified PermissionSet .",
        ),
    ),
    (
        "ListManagedPoliciesInPermissionSet",
        Decision::Deferred(
            "Required selectors InstanceArn, PermissionSetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AttachedManagedPolicies, NextToken. Pinned purpose: Lists the Amazon Web Services managed policy that is attached to a specified permission set.",
        ),
    ),
];
