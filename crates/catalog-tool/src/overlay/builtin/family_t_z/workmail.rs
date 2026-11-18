use super::super::service;
use super::{defer, deny, workmail_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workmail",
        vec![
            ("AssociateDelegateToResource", deny(DenyReason::Mutating)),
            ("AssociateMemberToGroup", deny(DenyReason::Mutating)),
            (
                "AssumeImpersonationRole",
                defer(
                    "Missing executable inventory binding for OrganizationId, ImpersonationRoleId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("CancelMailboxExportJob", deny(DenyReason::Mutating)),
            ("CreateAlias", deny(DenyReason::Mutating)),
            (
                "CreateAvailabilityConfiguration",
                deny(DenyReason::Mutating),
            ),
            ("CreateGroup", deny(DenyReason::Mutating)),
            (
                "CreateIdentityCenterApplication",
                deny(DenyReason::Mutating),
            ),
            ("CreateImpersonationRole", deny(DenyReason::Mutating)),
            ("CreateMobileDeviceAccessRule", deny(DenyReason::Mutating)),
            ("CreateOrganization", deny(DenyReason::Mutating)),
            ("CreateResource", deny(DenyReason::Mutating)),
            ("CreateUser", deny(DenyReason::Mutating)),
            ("DeleteAccessControlRule", deny(DenyReason::Mutating)),
            ("DeleteAlias", deny(DenyReason::Mutating)),
            (
                "DeleteAvailabilityConfiguration",
                deny(DenyReason::Mutating),
            ),
            (
                "DeleteEmailMonitoringConfiguration",
                deny(DenyReason::Mutating),
            ),
            ("DeleteGroup", deny(DenyReason::Mutating)),
            (
                "DeleteIdentityCenterApplication",
                deny(DenyReason::Mutating),
            ),
            (
                "DeleteIdentityProviderConfiguration",
                deny(DenyReason::Mutating),
            ),
            ("DeleteImpersonationRole", deny(DenyReason::Mutating)),
            ("DeleteMailboxPermissions", deny(DenyReason::Mutating)),
            (
                "DeleteMobileDeviceAccessOverride",
                deny(DenyReason::Mutating),
            ),
            ("DeleteMobileDeviceAccessRule", deny(DenyReason::Mutating)),
            ("DeleteOrganization", deny(DenyReason::Mutating)),
            ("DeletePersonalAccessToken", deny(DenyReason::Mutating)),
            ("DeleteResource", deny(DenyReason::Mutating)),
            ("DeleteRetentionPolicy", deny(DenyReason::Mutating)),
            ("DeleteUser", deny(DenyReason::Mutating)),
            ("DeregisterFromWorkMail", deny(DenyReason::Mutating)),
            ("DeregisterMailDomain", deny(DenyReason::Mutating)),
            (
                "DescribeEmailMonitoringConfiguration",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeEntity",
                defer(
                    "Missing executable inventory binding for OrganizationId, Email: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeGroup",
                defer(
                    "Missing executable inventory binding for OrganizationId, GroupId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeIdentityProviderConfiguration",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeInboundDmarcSettings",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeMailboxExportJob",
                defer(
                    "Missing executable inventory binding for JobId, OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeOrganization",
                workmail_plans_1::describe_organization()?,
            ),
            (
                "DescribeResource",
                defer(
                    "Missing executable inventory binding for OrganizationId, ResourceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeUser",
                defer(
                    "Missing executable inventory binding for OrganizationId, UserId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DisassociateDelegateFromResource",
                deny(DenyReason::Mutating),
            ),
            ("DisassociateMemberFromGroup", deny(DenyReason::Mutating)),
            (
                "GetAccessControlEffect",
                defer(
                    "Missing executable inventory binding for OrganizationId, IpAddress, Action: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetDefaultRetentionPolicy",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetImpersonationRole",
                defer(
                    "Missing executable inventory binding for OrganizationId, ImpersonationRoleId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetImpersonationRoleEffect",
                defer(
                    "Missing executable inventory binding for OrganizationId, ImpersonationRoleId, TargetUser: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetMailboxDetails",
                defer(
                    "Missing executable inventory binding for OrganizationId, UserId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetMailDomain",
                defer(
                    "Missing executable inventory binding for OrganizationId, DomainName: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetMobileDeviceAccessEffect",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetMobileDeviceAccessOverride",
                defer(
                    "Missing executable inventory binding for OrganizationId, UserId, DeviceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetPersonalAccessTokenMetadata",
                defer(
                    "Missing executable inventory binding for OrganizationId, PersonalAccessTokenId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListAccessControlRules",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListAliases",
                defer(
                    "Missing executable inventory binding for OrganizationId, EntityId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListAvailabilityConfigurations",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListGroupMembers",
                defer(
                    "Missing executable inventory binding for OrganizationId, GroupId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListGroups",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListGroupsForEntity",
                defer(
                    "Missing executable inventory binding for OrganizationId, EntityId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListImpersonationRoles",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListMailboxExportJobs",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListMailboxPermissions",
                defer(
                    "Missing executable inventory binding for OrganizationId, EntityId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListMailDomains",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListMobileDeviceAccessOverrides",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListMobileDeviceAccessRules",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("ListOrganizations", workmail_plans_1::list_organizations()?),
            (
                "ListPersonalAccessTokens",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListResourceDelegates",
                defer(
                    "Missing executable inventory binding for OrganizationId, ResourceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListResources",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "ListUsers",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("PutAccessControlRule", deny(DenyReason::Mutating)),
            (
                "PutEmailMonitoringConfiguration",
                deny(DenyReason::Mutating),
            ),
            (
                "PutIdentityProviderConfiguration",
                deny(DenyReason::Mutating),
            ),
            ("PutInboundDmarcSettings", deny(DenyReason::Mutating)),
            ("PutMailboxPermissions", deny(DenyReason::Mutating)),
            ("PutMobileDeviceAccessOverride", deny(DenyReason::Mutating)),
            ("PutRetentionPolicy", deny(DenyReason::Mutating)),
            ("RegisterMailDomain", deny(DenyReason::Mutating)),
            ("RegisterToWorkMail", deny(DenyReason::Mutating)),
            ("ResetPassword", deny(DenyReason::Mutating)),
            ("StartMailboxExportJob", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            (
                "TestAvailabilityConfiguration",
                defer(
                    "Missing executable inventory binding for OrganizationId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("UntagResource", deny(DenyReason::Mutating)),
            (
                "UpdateAvailabilityConfiguration",
                deny(DenyReason::Mutating),
            ),
            ("UpdateDefaultMailDomain", deny(DenyReason::Mutating)),
            ("UpdateGroup", deny(DenyReason::Mutating)),
            ("UpdateImpersonationRole", deny(DenyReason::Mutating)),
            ("UpdateMailboxQuota", deny(DenyReason::Mutating)),
            ("UpdateMobileDeviceAccessRule", deny(DenyReason::Mutating)),
            ("UpdatePrimaryEmailAddress", deny(DenyReason::Mutating)),
            ("UpdateResource", deny(DenyReason::Mutating)),
            ("UpdateUser", deny(DenyReason::Mutating)),
        ],
    )
}
