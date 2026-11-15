use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn get_network() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("networkArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "networkId",
        "networkName",
        "accessLevel",
        "awsAccountId",
        "networkArn",
        "standing",
        "freeTrialExpiration",
        "migrationState",
        "encryptionKeyArn",
    ])?;
    bind(&mut operation, "networkId", "ListNetworks", "networkId")?;
    Ok(operation)
}

pub(super) fn list_networks() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "networks",
        Identity::Arn(path("networkArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "networkId",
        "networkName",
        "accessLevel",
        "awsAccountId",
        "networkArn",
        "standing",
        "freeTrialExpiration",
        "migrationState",
        "encryptionKeyArn",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
