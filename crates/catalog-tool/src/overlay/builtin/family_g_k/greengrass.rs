//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/greengrass/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{MemberPath, MissingSelected, NameError, RowPredicate, RowValueType, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListGroups", policy::query(policy::enumerate("Groups", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn"])?),
        ("ListCoreDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListDeviceDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListFunctionDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListResourceDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListSubscriptionDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListLoggerDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListConnectorDefinitions", policy::query(policy::tags(policy::enumerate("Definitions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, "Tags")?, &["Arn"])?),
        ("ListConnectorDefinitionVersions", versions("ConnectorDefinitionId", "ListConnectorDefinitions")?),
        ("ListCoreDefinitionVersions", versions("CoreDefinitionId", "ListCoreDefinitions")?),
        ("ListDeviceDefinitionVersions", versions("DeviceDefinitionId", "ListDeviceDefinitions")?),
        ("ListFunctionDefinitionVersions", versions("FunctionDefinitionId", "ListFunctionDefinitions")?),
        ("ListLoggerDefinitionVersions", versions("LoggerDefinitionId", "ListLoggerDefinitions")?),
        ("ListResourceDefinitionVersions", versions("ResourceDefinitionId", "ListResourceDefinitions")?),
        ("ListSubscriptionDefinitionVersions", versions("SubscriptionDefinitionId", "ListSubscriptionDefinitions")?),
        ("ListGroupVersions", versions("GroupId", "ListGroups")?),
    ];
    policy::finish(
        "greengrass", approved, DEFERRED,
        "Definition containers, groups and immutable version summaries are ARN-addressed metadata. VersionInformation.Id is the parent ID, never the version identity. Explicit NextToken is modeled without a paginated trait; Get version bodies and component configurations remain unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

fn versions(input: &str, parent: &str) -> Result<crate::overlay::OperationOverlay, NameError> {
    let mut versions = policy::query(policy::bind(
        policy::enumerate("Versions", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?,
        input, parent, "Id",
    )?, &["Arn", "Id", "Version"])?;
    versions.row_policy.assertions.push(RowPredicate::SelectedRequestEquals {
        selected: MemberPath::selected("Id")?,
        request: MemberPath::selected(input)?,
        value_type: RowValueType::String,
        missing: MissingSelected::AllowAbsent,
    });
    Ok(versions)
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetAssociatedRole", "GetBulkDeploymentStatus", "GetConnectivityInfo", "GetConnectorDefinition",
    "GetConnectorDefinitionVersion", "GetCoreDefinition", "GetCoreDefinitionVersion", "GetDeploymentStatus",
    "GetDeviceDefinition", "GetDeviceDefinitionVersion", "GetFunctionDefinition",
    "GetFunctionDefinitionVersion", "GetGroup", "GetGroupCertificateAuthority",
    "GetGroupCertificateConfiguration", "GetGroupVersion", "GetLoggerDefinition",
    "GetLoggerDefinitionVersion", "GetResourceDefinition", "GetResourceDefinitionVersion",
    "GetServiceRoleForAccount", "GetSubscriptionDefinition", "GetSubscriptionDefinitionVersion",
    "GetThingRuntimeConfiguration", "ListBulkDeploymentDetailedReports", "ListBulkDeployments",
    "ListDeployments", "ListGroupCertificateAuthorities", "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AssociateRoleToGroup", "AssociateServiceRoleToAccount", "CreateConnectorDefinition",
    "CreateConnectorDefinitionVersion", "CreateCoreDefinition", "CreateCoreDefinitionVersion",
    "CreateDeployment", "CreateDeviceDefinition", "CreateDeviceDefinitionVersion",
    "CreateFunctionDefinition", "CreateFunctionDefinitionVersion", "CreateGroup",
    "CreateGroupCertificateAuthority", "CreateGroupVersion", "CreateLoggerDefinition",
    "CreateLoggerDefinitionVersion", "CreateResourceDefinition", "CreateResourceDefinitionVersion",
    "CreateSoftwareUpdateJob", "CreateSubscriptionDefinition", "CreateSubscriptionDefinitionVersion",
    "DeleteConnectorDefinition", "DeleteCoreDefinition", "DeleteDeviceDefinition",
    "DeleteFunctionDefinition", "DeleteGroup", "DeleteLoggerDefinition", "DeleteResourceDefinition",
    "DeleteSubscriptionDefinition", "DisassociateRoleFromGroup", "DisassociateServiceRoleFromAccount",
    "ResetDeployments", "StartBulkDeployment", "StopBulkDeployment", "TagResource", "UntagResource",
    "UpdateConnectivityInfo", "UpdateConnectorDefinition", "UpdateCoreDefinition", "UpdateDeviceDefinition",
    "UpdateFunctionDefinition", "UpdateGroup", "UpdateGroupCertificateConfiguration",
    "UpdateLoggerDefinition", "UpdateResourceDefinition", "UpdateSubscriptionDefinition",
    "UpdateThingRuntimeConfiguration",
];
