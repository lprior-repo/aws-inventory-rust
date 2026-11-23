use super::{inventory, path, paths, token};
use crate::overlay::{FixedValue, Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_profiles() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ProfileSummaries",
        Identity::Arn(path("ProfileArn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ProfileArn",
        "ProfileVersion",
        "ProfileName",
        "ProfileDescription",
        "Owner",
    ])?;
    operation.page_size = Some(50);
    operation.fixed.insert(path("ProfileOwnerType")?, FixedValue::String("SELF".to_owned()));
    Ok(operation)
}

pub(super) fn list_review_templates() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ReviewTemplates",
        Identity::Arn(path("TemplateArn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Description",
        "Owner",
        "TemplateArn",
        "TemplateName",
        "UpdateStatus",
    ])?;
    operation.page_size = Some(50);
    Ok(operation)
}

pub(super) fn list_workloads() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WorkloadSummaries",
        Identity::Arn(path("WorkloadArn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "WorkloadId",
        "WorkloadArn",
        "WorkloadName",
        "Owner",
        "ImprovementStatus",
    ])?;
    operation.page_size = Some(50);
    Ok(operation)
}
