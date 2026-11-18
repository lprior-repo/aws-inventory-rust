use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_device() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "device",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "serialNumber",
        "name",
        "model",
        "environmentId",
        "status",
        "currentSoftwareSetId",
        "currentSoftwareSetVersion",
        "desiredSoftwareSetId",
        "pendingSoftwareSetId",
        "pendingSoftwareSetVersion",
        "softwareSetUpdateSchedule",
        "softwareSetComplianceStatus",
        "softwareSetUpdateStatus",
        "arn",
        "kmsKeyArn",
        "lastUserId",
    ])?;
    bind(&mut operation, "id", "ListDevices", "id")?;
    Ok(operation)
}

pub(super) fn list_devices() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "devices",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "serialNumber",
        "name",
        "model",
        "environmentId",
        "status",
        "currentSoftwareSetId",
        "desiredSoftwareSetId",
        "pendingSoftwareSetId",
        "softwareSetUpdateSchedule",
        "arn",
        "lastUserId",
    ])?;
    operation.page_size = Some(50);
    Ok(operation)
}
