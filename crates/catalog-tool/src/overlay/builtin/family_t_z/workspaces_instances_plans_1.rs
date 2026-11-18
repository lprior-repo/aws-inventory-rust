use super::{bind, inventory, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_workspace_instance() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(paths(&["WorkspaceInstanceId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["ProvisionState", "WorkspaceInstanceId"])?;
    bind(
        &mut operation,
        "WorkspaceInstanceId",
        "ListWorkspaceInstances",
        "WorkspaceInstanceId",
    )?;
    Ok(operation)
}

pub(super) fn list_workspace_instances() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WorkspaceInstances",
        Identity::Key(paths(&["WorkspaceInstanceId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["ProvisionState", "WorkspaceInstanceId"])?;
    operation.page_size = Some(25);
    Ok(operation)
}
