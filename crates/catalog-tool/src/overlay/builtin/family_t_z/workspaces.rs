use super::super::service;
use super::{defer, deny, extra_plans, workspaces_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workspaces",
        vec![
            ("AcceptAccountLinkInvitation", deny(DenyReason::Mutating)),
            ("AssociateConnectionAlias", deny(DenyReason::Mutating)),
            ("AssociateIpGroups", deny(DenyReason::Mutating)),
            ("AssociateWorkspaceApplication", deny(DenyReason::Mutating)),
            ("AuthorizeIpRules", deny(DenyReason::Mutating)),
            (
                "CopyWorkspaceImage",
                defer(
                    "Missing executable inventory binding for Name, SourceImageId, SourceRegion: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("CreateAccountLinkInvitation", deny(DenyReason::Mutating)),
            ("CreateConnectClientAddIn", deny(DenyReason::Mutating)),
            ("CreateConnectionAlias", deny(DenyReason::Mutating)),
            ("CreateIpGroup", deny(DenyReason::Mutating)),
            ("CreateStandbyWorkspaces", deny(DenyReason::Mutating)),
            ("CreateTags", deny(DenyReason::Mutating)),
            ("CreateUpdatedWorkspaceImage", deny(DenyReason::Mutating)),
            ("CreateWorkspaceBundle", deny(DenyReason::Mutating)),
            ("CreateWorkspaceImage", deny(DenyReason::Mutating)),
            ("CreateWorkspaces", deny(DenyReason::Mutating)),
            ("CreateWorkspacesPool", deny(DenyReason::Mutating)),
            ("DeleteAccountLinkInvitation", deny(DenyReason::Mutating)),
            ("DeleteClientBranding", deny(DenyReason::Mutating)),
            ("DeleteConnectClientAddIn", deny(DenyReason::Mutating)),
            ("DeleteConnectionAlias", deny(DenyReason::Mutating)),
            ("DeleteIpGroup", deny(DenyReason::Mutating)),
            ("DeleteTags", deny(DenyReason::Mutating)),
            ("DeleteWorkspaceBundle", deny(DenyReason::Mutating)),
            ("DeleteWorkspaceImage", deny(DenyReason::Mutating)),
            ("DeployWorkspaceApplications", deny(DenyReason::Mutating)),
            ("DeregisterWorkspaceDirectory", deny(DenyReason::Mutating)),
            (
                "DescribeAccount",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            (
                "DescribeAccountModifications",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            (
                "DescribeApplicationAssociations",
                defer(
                    "Missing executable inventory binding for ApplicationId, AssociatedResourceTypes: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeApplications",
                workspaces_plans_1::describe_applications()?,
            ),
            (
                "DescribeBundleAssociations",
                defer(
                    "Missing executable inventory binding for BundleId, AssociatedResourceTypes: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeClientBranding",
                defer(
                    "Missing executable inventory binding for ResourceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeClientProperties",
                defer(
                    "Missing executable inventory binding for ResourceIds: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeConnectClientAddIns",
                defer(
                    "Missing executable inventory binding for ResourceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeConnectionAliases",
                workspaces_plans_1::describe_connection_aliases()?,
            ),
            (
                "DescribeConnectionAliasPermissions",
                defer(
                    "Missing executable inventory binding for AliasId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeCustomWorkspaceImageImport",
                defer(
                    "Missing executable inventory binding for ImageId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeImageAssociations",
                defer(
                    "Missing executable inventory binding for ImageId, AssociatedResourceTypes: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeIpGroups",
                workspaces_plans_1::describe_ip_groups()?,
            ),
            (
                "DescribeTags",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "DescribeWorkspaceAssociations",
                defer(
                    "Missing executable inventory binding for WorkspaceId, AssociatedResourceTypes: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeWorkspaceBundles",
                workspaces_plans_1::describe_workspace_bundles()?,
            ),
            (
                "DescribeWorkspaceDirectories",
                workspaces_plans_1::describe_workspace_directories()?,
            ),
            (
                "DescribeWorkspaceImagePermissions",
                defer(
                    "Missing executable inventory binding for ImageId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeWorkspaceImages",
                workspaces_plans_1::describe_workspace_images()?,
            ),
            (
                "DescribeWorkspaces",
                workspaces_plans_1::describe_workspaces()?,
            ),
            (
                "DescribeWorkspacesConnectionStatus",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            (
                "DescribeWorkspaceSnapshots",
                defer(
                    "Missing executable inventory binding for WorkspaceId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeWorkspacesPools",
                workspaces_plans_1::describe_workspaces_pools()?,
            ),
            (
                "DescribeWorkspacesPoolSessions",
                defer(
                    "Missing executable inventory binding for PoolId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("DisassociateConnectionAlias", deny(DenyReason::Mutating)),
            ("DisassociateIpGroups", deny(DenyReason::Mutating)),
            (
                "DisassociateWorkspaceApplication",
                deny(DenyReason::Mutating),
            ),
            ("GetAccountLink", extra_plans::get_account_link()?),
            ("ImportClientBranding", deny(DenyReason::Mutating)),
            ("ImportCustomWorkspaceImage", deny(DenyReason::Mutating)),
            ("ImportWorkspaceImage", deny(DenyReason::Mutating)),
            (
                "ListAccountLinks",
                workspaces_plans_1::list_account_links()?,
            ),
            (
                "ListAvailableManagementCidrRanges",
                defer(
                    "Missing executable inventory binding for ManagementCidrRangeConstraint: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("MigrateWorkspace", deny(DenyReason::Mutating)),
            ("ModifyAccount", deny(DenyReason::Mutating)),
            (
                "ModifyCertificateBasedAuthProperties",
                deny(DenyReason::Mutating),
            ),
            ("ModifyClientProperties", deny(DenyReason::Mutating)),
            ("ModifyEndpointEncryptionMode", deny(DenyReason::Mutating)),
            ("ModifySamlProperties", deny(DenyReason::Mutating)),
            ("ModifySelfservicePermissions", deny(DenyReason::Mutating)),
            ("ModifyStreamingProperties", deny(DenyReason::Mutating)),
            (
                "ModifyWorkspaceAccessProperties",
                deny(DenyReason::Mutating),
            ),
            (
                "ModifyWorkspaceCreationProperties",
                deny(DenyReason::Mutating),
            ),
            ("ModifyWorkspaceProperties", deny(DenyReason::Mutating)),
            ("ModifyWorkspaceState", deny(DenyReason::Mutating)),
            ("RebootWorkspaces", deny(DenyReason::Mutating)),
            ("RebuildWorkspaces", deny(DenyReason::Mutating)),
            ("RegisterWorkspaceDirectory", deny(DenyReason::Mutating)),
            ("RejectAccountLinkInvitation", deny(DenyReason::Mutating)),
            ("RestoreWorkspace", deny(DenyReason::Mutating)),
            ("RevokeIpRules", deny(DenyReason::Mutating)),
            ("StartWorkspaces", deny(DenyReason::Mutating)),
            ("StartWorkspacesPool", deny(DenyReason::Mutating)),
            ("StopWorkspaces", deny(DenyReason::Mutating)),
            ("StopWorkspacesPool", deny(DenyReason::Mutating)),
            ("TerminateWorkspaces", deny(DenyReason::Mutating)),
            ("TerminateWorkspacesPool", deny(DenyReason::Mutating)),
            ("TerminateWorkspacesPoolSession", deny(DenyReason::Mutating)),
            ("UpdateConnectClientAddIn", deny(DenyReason::Mutating)),
            (
                "UpdateConnectionAliasPermission",
                deny(DenyReason::Mutating),
            ),
            ("UpdateRulesOfIpGroup", deny(DenyReason::Mutating)),
            ("UpdateWorkspaceBundle", deny(DenyReason::Mutating)),
            ("UpdateWorkspaceImagePermission", deny(DenyReason::Mutating)),
            ("UpdateWorkspacesPool", deny(DenyReason::Mutating)),
        ],
    )
}
