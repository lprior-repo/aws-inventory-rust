use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_db_instances() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "status",
        "endpoint",
        "port",
        "networkType",
        "dbInstanceType",
        "dbStorageType",
        "allocatedStorage",
        "deploymentType",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_db_instances_for_cluster() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "status",
        "endpoint",
        "port",
        "networkType",
        "dbInstanceType",
        "dbStorageType",
        "allocatedStorage",
        "deploymentType",
        "instanceMode",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "dbClusterId", "ListDbClusters", "id")?;
    Ok(operation)
}

pub(super) fn list_db_parameter_groups() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["id", "name", "arn", "description"])?;
    operation.page_size = Some(100);
    Ok(operation)
}
