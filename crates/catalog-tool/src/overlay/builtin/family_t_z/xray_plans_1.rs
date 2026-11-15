use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn get_group() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Group",
        Identity::Arn(path("GroupARN")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "GroupName",
        "GroupARN",
        "FilterExpression",
    ])?;
    bind(&mut operation, "GroupARN", "GetGroups", "GroupARN")?;
    Ok(operation)
}

pub(super) fn get_groups() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Groups",
        Identity::Arn(path("GroupARN")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "GroupName",
        "GroupARN",
        "FilterExpression",
    ])?;
    Ok(operation)
}

pub(super) fn get_indexing_rules() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "IndexingRules",
        Identity::Key(paths(&["Name"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Name",
    ])?;
    Ok(operation)
}

pub(super) fn get_sampling_rules() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "SamplingRuleRecords",
        Identity::Arn(path("SamplingRule.RuleARN")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "SamplingRule.RuleARN",
        "SamplingRule.RuleName",
    ])?;
    Ok(operation)
}

pub(super) fn list_resource_policies() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ResourcePolicies",
        Identity::Key(paths(&["PolicyName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "PolicyName",
        "PolicyDocument",
        "PolicyRevisionId",
    ])?;
    Ok(operation)
}
