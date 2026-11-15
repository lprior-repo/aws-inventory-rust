use super::{inventory, path, paths, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Scope};

pub(super) fn list_target_groups() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "name",
        "type",
        "port",
        "protocol",
        "ipAddressType",
        "vpcIdentifier",
        "status",
        "lambdaEventStructureVersion",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
