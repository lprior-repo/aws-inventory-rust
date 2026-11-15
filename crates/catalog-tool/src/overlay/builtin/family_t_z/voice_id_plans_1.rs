use super::{inventory, path, paths, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Scope};

pub(super) fn list_domains() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "DomainSummaries",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "DomainId",
        "Arn",
        "Name",
        "Description",
        "DomainStatus",
    ])?;
    operation.page_size = Some(10);
    Ok(operation)
}
