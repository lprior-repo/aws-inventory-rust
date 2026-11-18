use super::{bind, inventory, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn describe_organization() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(paths(&["OrganizationId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "OrganizationId",
        "Alias",
        "State",
        "DirectoryId",
        "DirectoryType",
        "DefaultMailDomain",
        "ErrorMessage",
        "ARN",
        "MigrationAdmin",
        "InteroperabilityEnabled",
    ])?;
    bind(
        &mut operation,
        "OrganizationId",
        "ListOrganizations",
        "OrganizationId",
    )?;
    Ok(operation)
}

pub(super) fn list_organizations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "OrganizationSummaries",
        Identity::Key(paths(&["OrganizationId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "OrganizationId",
        "Alias",
        "DefaultMailDomain",
        "ErrorMessage",
        "State",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
