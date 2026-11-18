use super::{bind, inventory, path, paths};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_db_backup() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
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
        "dbParameterGroupId",
        "dbInstanceType",
        "failoverMode",
        "dbStorageType",
        "allocatedStorage",
        "publiclyAccessible",
        "port",
        "networkType",
        "influxAuthParametersSecretArn",
    ])?;
    bind(&mut operation, "identifier", "ListDbBackups", "id")?;
    Ok(operation)
}

pub(super) fn get_db_cluster() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
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
        "publiclyAccessible",
        "dbParameterGroupIdentifier",
        "effectiveDbParameterGroupIdentifier",
        "influxAuthParametersSecretArn",
        "failoverMode",
        "kmsKeyId",
    ])?;
    bind(&mut operation, "dbClusterId", "ListDbClusters", "id")?;
    Ok(operation)
}

pub(super) fn get_db_instance() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
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
        "publiclyAccessible",
        "dbParameterGroupIdentifier",
        "availabilityZone",
        "secondaryAvailabilityZone",
        "influxAuthParametersSecretArn",
        "dbClusterId",
        "instanceMode",
        "kmsKeyId",
    ])?;
    bind(&mut operation, "identifier", "ListDbInstances", "id")?;
    Ok(operation)
}
