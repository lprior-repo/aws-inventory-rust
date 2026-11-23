use super::{inventory, path, paths, token};
use crate::overlay::{Binding, FixedValue, Identity, NameError, OperationOverlay, Scope};

pub(super) fn describe_applications() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Applications",
        Identity::Key(paths(&["ApplicationId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ApplicationId",
        "Description",
        "LicenseType",
        "Name",
        "Owner",
        "State",
    ])?;
    operation.page_size = Some(25);
    // Owner accepts an account ID or AMAZON; never inventory vendor applications as ours.
    operation.bind.insert(path("Owner")?, Binding::Account);
    Ok(operation)
}

pub(super) fn describe_connection_aliases() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ConnectionAliases",
        Identity::Key(paths(&["AliasId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["ConnectionString", "AliasId", "State", "OwnerAccountId"])?;
    Ok(operation)
}

pub(super) fn describe_ip_groups() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Result",
        Identity::Key(paths(&["groupId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["groupId", "groupName", "groupDesc"])?;
    Ok(operation)
}

pub(super) fn describe_workspace_bundles() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Bundles",
        Identity::Key(paths(&["BundleId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "BundleId",
        "Name",
        "Owner",
        "Description",
        "ImageId",
        "State",
        "BundleType",
    ])?;
    Ok(operation)
}

pub(super) fn describe_workspace_directories() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Directories",
        Identity::Key(paths(&["DirectoryId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "DirectoryId",
        "Alias",
        "DirectoryName",
        "RegistrationCode",
        "CustomerUserName",
        "IamRoleId",
        "DirectoryType",
        "WorkspaceSecurityGroupId",
        "State",
        "Tenancy",
        "EndpointEncryptionMode",
        "WorkspaceDirectoryName",
        "WorkspaceDirectoryDescription",
        "UserIdentityType",
        "WorkspaceType",
        "ErrorMessage",
    ])?;
    Ok(operation)
}

pub(super) fn describe_workspace_images() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Images",
        Identity::Key(paths(&["ImageId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ImageId",
        "Name",
        "Description",
        "State",
        "RequiredTenancy",
        "ErrorCode",
        "ErrorMessage",
        "OwnerAccountId",
    ])?;
    operation.fixed.insert(path("ImageType")?, FixedValue::String("OWNED".to_owned()));
    Ok(operation)
}

pub(super) fn describe_workspaces() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Workspaces",
        Identity::Key(paths(&["WorkspaceId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "WorkspaceId",
        "DirectoryId",
        "UserName",
        "IpAddress",
        "Ipv6Address",
        "State",
        "BundleId",
        "SubnetId",
        "ErrorMessage",
        "ErrorCode",
        "ComputerName",
        "VolumeEncryptionKey",
        "UserVolumeEncryptionEnabled",
        "RootVolumeEncryptionEnabled",
        "WorkspaceName",
    ])?;
    operation.page_size = Some(25);
    Ok(operation)
}

pub(super) fn describe_workspaces_pools() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WorkspacesPools",
        Identity::Arn(path("PoolArn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "PoolId",
        "PoolArn",
        "PoolName",
        "Description",
        "State",
        "BundleId",
        "DirectoryId",
        "RunningMode",
    ])?;
    Ok(operation)
}

pub(super) fn list_account_links() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "AccountLinks",
        Identity::Key(paths(&["AccountLinkId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AccountLinkId",
        "AccountLinkStatus",
        "SourceAccountId",
        "TargetAccountId",
    ])?;
    operation.page_size = Some(25);
    Ok(operation)
}
