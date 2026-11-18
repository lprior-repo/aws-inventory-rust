use super::{inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_network_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "networkSettings",
        Identity::Arn(path("networkSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["networkSettingsArn", "vpcId"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_portals() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "portals",
        Identity::Arn(path("portalArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "portalArn",
        "rendererType",
        "browserType",
        "portalStatus",
        "portalEndpoint",
        "displayName",
        "browserSettingsArn",
        "dataProtectionSettingsArn",
        "userSettingsArn",
        "networkSettingsArn",
        "sessionLoggerArn",
        "trustStoreArn",
        "userAccessLoggingSettingsArn",
        "authenticationType",
        "ipAccessSettingsArn",
        "instanceType",
        "maxConcurrentSessions",
        "portalCustomDomain",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_session_loggers() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "sessionLoggers",
        Identity::Arn(path("sessionLoggerArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["sessionLoggerArn", "displayName"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_trust_stores() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "trustStores",
        Identity::Arn(path("trustStoreArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["trustStoreArn"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_user_access_logging_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "userAccessLoggingSettings",
        Identity::Arn(path("userAccessLoggingSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["userAccessLoggingSettingsArn", "kinesisStreamArn"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_user_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "userSettings",
        Identity::Arn(path("userSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "userSettingsArn",
        "copyAllowed",
        "pasteAllowed",
        "downloadAllowed",
        "uploadAllowed",
        "printAllowed",
        "disconnectTimeoutInMinutes",
        "idleDisconnectTimeoutInMinutes",
        "deepLinkAllowed",
        "webAuthnAllowed",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
