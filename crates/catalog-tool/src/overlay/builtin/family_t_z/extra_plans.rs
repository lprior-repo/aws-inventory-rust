use super::{bind, inventory, path, paths};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn describe_domain() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Domain",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "DomainId",
        "Arn",
        "Name",
        "Description",
        "DomainStatus",
        "ServerSideEncryptionConfiguration.KmsKeyId",
        "ServerSideEncryptionUpdateDetails.OldKmsKeyId",
        "ServerSideEncryptionUpdateDetails.UpdateStatus",
        "ServerSideEncryptionUpdateDetails.Message",
        "WatchlistDetails.DefaultWatchlistId",
    ])?;
    bind(&mut operation, "DomainId", "ListDomains", "DomainId")?;
    Ok(operation)
}

pub(super) fn get_account_link() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "AccountLink",
        Identity::Key(paths(&["AccountLinkId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AccountLinkId",
        "AccountLinkStatus",
        "SourceAccountId",
        "TargetAccountId",
    ])?;
    bind(&mut operation, "LinkId", "ListAccountLinks", "AccountLinkId")?;
    Ok(operation)
}

pub(super) fn get_portal_service_provider_metadata() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("portalArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "portalArn",
        "serviceProviderSamlMetadata",
    ])?;
    bind(&mut operation, "portalArn", "ListPortals", "portalArn")?;
    Ok(operation)
}
