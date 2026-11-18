use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_browser_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "browserSettings",
        Identity::Arn(path("browserSettingsArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["browserSettingsArn", "browserPolicy", "customerManagedKey"])?;
    bind(
        &mut operation,
        "browserSettingsArn",
        "ListBrowserSettings",
        "browserSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn get_data_protection_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "dataProtectionSettings",
        Identity::Arn(path("dataProtectionSettingsArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "dataProtectionSettingsArn",
        "displayName",
        "description",
        "customerManagedKey",
    ])?;
    bind(
        &mut operation,
        "dataProtectionSettingsArn",
        "ListDataProtectionSettings",
        "dataProtectionSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn get_ip_access_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ipAccessSettings",
        Identity::Arn(path("ipAccessSettingsArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ipAccessSettingsArn",
        "displayName",
        "description",
        "customerManagedKey",
    ])?;
    bind(
        &mut operation,
        "ipAccessSettingsArn",
        "ListIpAccessSettings",
        "ipAccessSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn get_network_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "networkSettings",
        Identity::Arn(path("networkSettingsArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["networkSettingsArn", "vpcId"])?;
    bind(
        &mut operation,
        "networkSettingsArn",
        "ListNetworkSettings",
        "networkSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn get_portal() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "portal",
        Identity::Arn(path("portalArn")?),
        Paginator::None,
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
        "statusReason",
        "userAccessLoggingSettingsArn",
        "authenticationType",
        "ipAccessSettingsArn",
        "customerManagedKey",
        "instanceType",
        "maxConcurrentSessions",
        "portalCustomDomain",
    ])?;
    bind(&mut operation, "portalArn", "ListPortals", "portalArn")?;
    Ok(operation)
}

pub(super) fn get_session_logger() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "sessionLogger",
        Identity::Arn(path("sessionLoggerArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["sessionLoggerArn", "customerManagedKey", "displayName"])?;
    bind(
        &mut operation,
        "sessionLoggerArn",
        "ListSessionLoggers",
        "sessionLoggerArn",
    )?;
    Ok(operation)
}

pub(super) fn get_trust_store() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "trustStore",
        Identity::Arn(path("trustStoreArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["trustStoreArn"])?;
    bind(
        &mut operation,
        "trustStoreArn",
        "ListTrustStores",
        "trustStoreArn",
    )?;
    Ok(operation)
}

pub(super) fn get_user_access_logging_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "userAccessLoggingSettings",
        Identity::Arn(path("userAccessLoggingSettingsArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["userAccessLoggingSettingsArn", "kinesisStreamArn"])?;
    bind(
        &mut operation,
        "userAccessLoggingSettingsArn",
        "ListUserAccessLoggingSettings",
        "userAccessLoggingSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn get_user_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "userSettings",
        Identity::Arn(path("userSettingsArn")?),
        Paginator::None,
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
        "customerManagedKey",
        "deepLinkAllowed",
        "webAuthnAllowed",
    ])?;
    bind(
        &mut operation,
        "userSettingsArn",
        "ListUserSettings",
        "userSettingsArn",
    )?;
    Ok(operation)
}

pub(super) fn list_browser_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "browserSettings",
        Identity::Arn(path("browserSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["browserSettingsArn"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_data_protection_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "dataProtectionSettings",
        Identity::Arn(path("dataProtectionSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["dataProtectionSettingsArn", "displayName", "description"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_identity_providers() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "identityProviders",
        Identity::Arn(path("identityProviderArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "identityProviderArn",
        "identityProviderName",
        "identityProviderType",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "portalArn", "ListPortals", "portalArn")?;
    Ok(operation)
}

pub(super) fn list_ip_access_settings() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ipAccessSettings",
        Identity::Arn(path("ipAccessSettingsArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["ipAccessSettingsArn", "displayName", "description"])?;
    operation.page_size = Some(100);
    Ok(operation)
}
