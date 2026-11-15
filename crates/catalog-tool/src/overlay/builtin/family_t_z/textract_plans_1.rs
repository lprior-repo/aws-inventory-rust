use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn get_adapter() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(paths(&["AdapterId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AdapterId",
        "AdapterName",
        "Description",
        "AutoUpdate",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "AdapterId", "ListAdapters", "AdapterId")?;
    Ok(operation)
}

pub(super) fn get_adapter_version() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(paths(&["AdapterId", "AdapterVersion"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AdapterId",
        "AdapterVersion",
        "Status",
        "StatusMessage",
        "KMSKeyId",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "AdapterId", "ListAdapterVersions", "AdapterId")?;
    bind(&mut operation, "AdapterVersion", "ListAdapterVersions", "AdapterVersion")?;
    Ok(operation)
}

pub(super) fn list_adapters() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Adapters",
        Identity::Key(paths(&["AdapterId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AdapterId",
        "AdapterName",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_adapter_versions() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "AdapterVersions",
        Identity::Key(paths(&["AdapterId", "AdapterVersion"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "AdapterId",
        "AdapterVersion",
        "Status",
        "StatusMessage",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
