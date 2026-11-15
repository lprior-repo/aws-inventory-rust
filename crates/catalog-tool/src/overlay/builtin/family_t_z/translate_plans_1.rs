use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn describe_text_translation_job() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "TextTranslationJobProperties",
        Identity::Key(paths(&["JobId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "JobId",
        "JobName",
        "JobStatus",
        "SourceLanguageCode",
        "Message",
        "DataAccessRoleArn",
    ])?;
    bind(&mut operation, "JobId", "ListTextTranslationJobs", "JobId")?;
    Ok(operation)
}

pub(super) fn list_parallel_data() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ParallelDataPropertiesList",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Name",
        "Arn",
        "Description",
        "Status",
        "SourceLanguageCode",
        "Message",
        "ImportedDataSize",
        "ImportedRecordCount",
        "FailedRecordCount",
        "SkippedRecordCount",
        "LatestUpdateAttemptStatus",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_terminologies() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "TerminologyPropertiesList",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Name",
        "Description",
        "Arn",
        "SourceLanguageCode",
        "SizeBytes",
        "TermCount",
        "Directionality",
        "Message",
        "SkippedTermCount",
        "Format",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_text_translation_jobs() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "TextTranslationJobPropertiesList",
        Identity::Key(paths(&["JobId"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "JobId",
        "JobName",
        "JobStatus",
        "SourceLanguageCode",
        "Message",
        "DataAccessRoleArn",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
