use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn describe_language_model() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "LanguageModel",
        Identity::Key(paths(&["ModelName"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ModelName",
        "LanguageCode",
        "BaseModelName",
        "ModelStatus",
        "UpgradeAvailability",
        "FailureReason",
    ])?;
    bind(&mut operation, "ModelName", "ListLanguageModels", "ModelName")?;
    Ok(operation)
}

pub(super) fn get_call_analytics_category() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "CategoryProperties",
        Identity::Key(paths(&["CategoryName"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "CategoryName",
        "InputType",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "CategoryName", "ListCallAnalyticsCategories", "CategoryName")?;
    Ok(operation)
}

pub(super) fn list_call_analytics_categories() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Categories",
        Identity::Key(paths(&["CategoryName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "CategoryName",
        "InputType",
    ])?;
    operation.tags = Some(path("Tags")?);
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_call_analytics_jobs() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "CallAnalyticsJobSummaries",
        Identity::Key(paths(&["CallAnalyticsJobName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "CallAnalyticsJobName",
        "LanguageCode",
        "CallAnalyticsJobStatus",
        "FailureReason",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_language_models() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Models",
        Identity::Key(paths(&["ModelName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ModelName",
        "LanguageCode",
        "BaseModelName",
        "ModelStatus",
        "UpgradeAvailability",
        "FailureReason",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_medical_scribe_jobs() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "MedicalScribeJobSummaries",
        Identity::Key(paths(&["MedicalScribeJobName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "MedicalScribeJobName",
        "LanguageCode",
        "MedicalScribeJobStatus",
        "FailureReason",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_medical_transcription_jobs() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "MedicalTranscriptionJobSummaries",
        Identity::Key(paths(&["MedicalTranscriptionJobName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "MedicalTranscriptionJobName",
        "LanguageCode",
        "TranscriptionJobStatus",
        "FailureReason",
        "OutputLocationType",
        "Specialty",
        "ContentIdentificationType",
        "Type",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_medical_vocabularies() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Vocabularies",
        Identity::Key(paths(&["VocabularyName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "VocabularyName",
        "LanguageCode",
        "VocabularyState",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_transcription_jobs() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "TranscriptionJobSummaries",
        Identity::Key(paths(&["TranscriptionJobName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "TranscriptionJobName",
        "LanguageCode",
        "TranscriptionJobStatus",
        "FailureReason",
        "OutputLocationType",
        "IdentifyLanguage",
        "IdentifyMultipleLanguages",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_vocabularies() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Vocabularies",
        Identity::Key(paths(&["VocabularyName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "VocabularyName",
        "LanguageCode",
        "VocabularyState",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_vocabulary_filters() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "VocabularyFilters",
        Identity::Key(paths(&["VocabularyFilterName"])?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "VocabularyFilterName",
        "LanguageCode",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}
