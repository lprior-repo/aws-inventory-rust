use super::super::service;
use super::{defer, deny, transfer_plans_1, transfer_plans_2};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "transfer",
        vec![
            ("CreateAccess", deny(DenyReason::Mutating)),
            ("CreateAgreement", deny(DenyReason::Mutating)),
            ("CreateConnector", deny(DenyReason::Mutating)),
            ("CreateProfile", deny(DenyReason::Mutating)),
            ("CreateServer", deny(DenyReason::Mutating)),
            ("CreateUser", deny(DenyReason::Mutating)),
            ("CreateWebApp", deny(DenyReason::Mutating)),
            ("CreateWorkflow", deny(DenyReason::Mutating)),
            ("DeleteAccess", deny(DenyReason::Mutating)),
            ("DeleteAgreement", deny(DenyReason::Mutating)),
            ("DeleteCertificate", deny(DenyReason::Mutating)),
            ("DeleteConnector", deny(DenyReason::Mutating)),
            ("DeleteHostKey", deny(DenyReason::Mutating)),
            ("DeleteProfile", deny(DenyReason::Mutating)),
            ("DeleteServer", deny(DenyReason::Mutating)),
            ("DeleteSshPublicKey", deny(DenyReason::Mutating)),
            ("DeleteUser", deny(DenyReason::Mutating)),
            ("DeleteWebApp", deny(DenyReason::Mutating)),
            ("DeleteWebAppCustomization", deny(DenyReason::Mutating)),
            ("DeleteWorkflow", deny(DenyReason::Mutating)),
            (
                "DescribeAccess",
                defer(
                    "Missing executable inventory binding for ServerId, ExternalId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("DescribeAgreement", transfer_plans_1::describe_agreement()?),
            (
                "DescribeCertificate",
                transfer_plans_1::describe_certificate()?,
            ),
            ("DescribeConnector", transfer_plans_1::describe_connector()?),
            (
                "DescribeExecution",
                defer(
                    "Missing executable inventory binding for ExecutionId, WorkflowId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "DescribeHostKey",
                defer(
                    "Missing executable inventory binding for ServerId, HostKeyId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("DescribeProfile", transfer_plans_1::describe_profile()?),
            ("DescribeSecurityPolicy", deny(DenyReason::ReferenceData)),
            ("DescribeServer", transfer_plans_1::describe_server()?),
            (
                "DescribeUser",
                defer(
                    "Missing executable inventory binding for ServerId, UserName: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("DescribeWebApp", transfer_plans_1::describe_web_app()?),
            ("DescribeWebAppCustomization", deny(DenyReason::Content)),
            ("DescribeWorkflow", transfer_plans_1::describe_workflow()?),
            ("ImportCertificate", deny(DenyReason::Mutating)),
            ("ImportHostKey", deny(DenyReason::Mutating)),
            ("ImportSshPublicKey", deny(DenyReason::Mutating)),
            (
                "ListAccesses",
                defer(
                    "Missing executable inventory binding for ServerId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("ListAgreements", transfer_plans_1::list_agreements()?),
            ("ListCertificates", transfer_plans_1::list_certificates()?),
            ("ListConnectors", transfer_plans_1::list_connectors()?),
            (
                "ListExecutions",
                defer(
                    "Missing executable inventory binding for WorkflowId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListFileTransferResults",
                defer(
                    "Missing executable inventory binding for ConnectorId, TransferId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("ListHostKeys", transfer_plans_1::list_host_keys()?),
            ("ListProfiles", transfer_plans_2::list_profiles()?),
            ("ListSecurityPolicies", deny(DenyReason::ReferenceData)),
            ("ListServers", transfer_plans_2::list_servers()?),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("ListUsers", transfer_plans_2::list_users()?),
            ("ListWebApps", transfer_plans_2::list_web_apps()?),
            ("ListWorkflows", transfer_plans_2::list_workflows()?),
            ("SendWorkflowStepState", deny(DenyReason::Mutating)),
            ("StartDirectoryListing", deny(DenyReason::Mutating)),
            ("StartFileTransfer", deny(DenyReason::Mutating)),
            ("StartRemoteDelete", deny(DenyReason::Mutating)),
            ("StartRemoteMove", deny(DenyReason::Mutating)),
            ("StartServer", deny(DenyReason::Mutating)),
            ("StopServer", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            (
                "TestConnection",
                defer(
                    "Missing executable inventory binding for ConnectorId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "TestIdentityProvider",
                defer(
                    "Missing executable inventory binding for ServerId, UserName: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateAccess", deny(DenyReason::Mutating)),
            ("UpdateAgreement", deny(DenyReason::Mutating)),
            ("UpdateCertificate", deny(DenyReason::Mutating)),
            ("UpdateConnector", deny(DenyReason::Mutating)),
            ("UpdateHostKey", deny(DenyReason::Mutating)),
            ("UpdateProfile", deny(DenyReason::Mutating)),
            ("UpdateServer", deny(DenyReason::Mutating)),
            ("UpdateUser", deny(DenyReason::Mutating)),
            ("UpdateWebApp", deny(DenyReason::Mutating)),
            ("UpdateWebAppCustomization", deny(DenyReason::Mutating)),
        ],
    )
}
