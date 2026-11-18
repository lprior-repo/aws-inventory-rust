use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_profiles() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Profiles",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["Arn", "ProfileId", "As2Id", "ProfileType"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_servers() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Servers",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "Domain",
        "IdentityProviderType",
        "EndpointType",
        "LoggingRole",
        "ServerId",
        "State",
        "UserCount",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_users() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Users",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "HomeDirectory",
        "HomeDirectoryType",
        "Role",
        "SshPublicKeyCount",
        "UserName",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "ServerId", "ListServers", "ServerId")?;
    Ok(operation)
}

pub(super) fn list_web_apps() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WebApps",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "WebAppId",
        "AccessEndpoint",
        "WebAppEndpoint",
        "EndpointType",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_workflows() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Workflows",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["WorkflowId", "Description", "Arn"])?;
    operation.page_size = Some(100);
    Ok(operation)
}
