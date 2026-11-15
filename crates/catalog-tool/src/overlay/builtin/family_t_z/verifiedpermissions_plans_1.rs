use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn get_identity_source() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Key(paths(&["policyStoreId", "identitySourceId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "identitySourceId",
        "policyStoreId",
        "principalEntityType",
    ])?;
    bind(&mut operation, "policyStoreId", "ListIdentitySources", "policyStoreId")?;
    bind(&mut operation, "identitySourceId", "ListIdentitySources", "identitySourceId")?;
    Ok(operation)
}

pub(super) fn get_policy_store() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "policyStoreId",
        "arn",
        "description",
        "deletionProtection",
        "cedarVersion",
    ])?;
    operation.tags = Some(path("tags")?);
    bind(&mut operation, "policyStoreId", "ListPolicyStores", "policyStoreId")?;
    Ok(operation)
}

pub(super) fn get_policy_store_alias() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("aliasArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "aliasName",
        "policyStoreId",
        "aliasArn",
        "state",
    ])?;
    bind(&mut operation, "aliasName", "ListPolicyStoreAliases", "aliasName")?;
    Ok(operation)
}

pub(super) fn list_identity_sources() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "identitySources",
        Identity::Key(paths(&["policyStoreId", "identitySourceId"])?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "identitySourceId",
        "policyStoreId",
        "principalEntityType",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "policyStoreId", "ListPolicyStores", "policyStoreId")?;
    Ok(operation)
}

pub(super) fn list_policies() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "policies",
        Identity::Key(paths(&["policyStoreId", "policyId"])?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "policyStoreId",
        "policyId",
        "policyType",
        "effect",
        "name",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "policyStoreId", "ListPolicyStores", "policyStoreId")?;
    Ok(operation)
}

pub(super) fn list_policy_store_aliases() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "policyStoreAliases",
        Identity::Arn(path("aliasArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "aliasName",
        "policyStoreId",
        "aliasArn",
        "state",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_policy_stores() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "policyStores",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "policyStoreId",
        "arn",
        "description",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_policy_templates() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "policyTemplates",
        Identity::Key(paths(&["policyStoreId", "policyTemplateId"])?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "policyStoreId",
        "policyTemplateId",
        "description",
        "name",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "policyStoreId", "ListPolicyStores", "policyStoreId")?;
    Ok(operation)
}
