use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_db_parameter_group() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(vec![path("id")?]),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["id", "name", "arn", "description"])?;
    bind(&mut operation, "identifier", "ListDbParameterGroups", "id")?;
    Ok(operation)
}

pub(super) fn list_db_backups() -> Result<OperationOverlay, NameError> {
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
        "expiresAfter",
        "dbResourceId",
        "type",
        "engineType",
        "deploymentType",
        "kmsKeyId",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_db_clusters() -> Result<OperationOverlay, NameError> {
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
        "readerEndpoint",
        "port",
        "deploymentType",
        "dbInstanceType",
        "networkType",
        "dbStorageType",
        "allocatedStorage",
        "engineType",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
