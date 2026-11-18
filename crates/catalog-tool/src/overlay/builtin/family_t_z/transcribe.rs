use super::super::service;
use super::{defer, deny, transcribe_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "transcribe",
        vec![
            ("CreateCallAnalyticsCategory", deny(DenyReason::Mutating)),
            ("CreateLanguageModel", deny(DenyReason::Mutating)),
            ("CreateMedicalVocabulary", deny(DenyReason::Mutating)),
            ("CreateVocabulary", deny(DenyReason::Mutating)),
            ("CreateVocabularyFilter", deny(DenyReason::Mutating)),
            ("DeleteCallAnalyticsCategory", deny(DenyReason::Mutating)),
            ("DeleteCallAnalyticsJob", deny(DenyReason::Mutating)),
            ("DeleteLanguageModel", deny(DenyReason::Mutating)),
            ("DeleteMedicalScribeJob", deny(DenyReason::Mutating)),
            ("DeleteMedicalTranscriptionJob", deny(DenyReason::Mutating)),
            ("DeleteMedicalVocabulary", deny(DenyReason::Mutating)),
            ("DeleteTranscriptionJob", deny(DenyReason::Mutating)),
            ("DeleteVocabulary", deny(DenyReason::Mutating)),
            ("DeleteVocabularyFilter", deny(DenyReason::Mutating)),
            (
                "DescribeLanguageModel",
                transcribe_plans_1::describe_language_model()?,
            ),
            (
                "GetCallAnalyticsCategory",
                transcribe_plans_1::get_call_analytics_category()?,
            ),
            ("GetCallAnalyticsJob", deny(DenyReason::Content)),
            ("GetMedicalScribeJob", deny(DenyReason::Content)),
            ("GetMedicalTranscriptionJob", deny(DenyReason::Content)),
            ("GetMedicalVocabulary", deny(DenyReason::Content)),
            ("GetTranscriptionJob", deny(DenyReason::Content)),
            ("GetVocabulary", deny(DenyReason::Content)),
            ("GetVocabularyFilter", deny(DenyReason::Content)),
            (
                "ListCallAnalyticsCategories",
                transcribe_plans_1::list_call_analytics_categories()?,
            ),
            (
                "ListCallAnalyticsJobs",
                transcribe_plans_1::list_call_analytics_jobs()?,
            ),
            (
                "ListLanguageModels",
                transcribe_plans_1::list_language_models()?,
            ),
            (
                "ListMedicalScribeJobs",
                transcribe_plans_1::list_medical_scribe_jobs()?,
            ),
            (
                "ListMedicalTranscriptionJobs",
                transcribe_plans_1::list_medical_transcription_jobs()?,
            ),
            (
                "ListMedicalVocabularies",
                transcribe_plans_1::list_medical_vocabularies()?,
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "ListTranscriptionJobs",
                transcribe_plans_1::list_transcription_jobs()?,
            ),
            ("ListVocabularies", transcribe_plans_1::list_vocabularies()?),
            (
                "ListVocabularyFilters",
                transcribe_plans_1::list_vocabulary_filters()?,
            ),
            ("StartCallAnalyticsJob", deny(DenyReason::Mutating)),
            ("StartMedicalScribeJob", deny(DenyReason::Mutating)),
            ("StartMedicalTranscriptionJob", deny(DenyReason::Mutating)),
            ("StartTranscriptionJob", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateCallAnalyticsCategory", deny(DenyReason::Mutating)),
            ("UpdateLanguageModel", deny(DenyReason::Mutating)),
            ("UpdateMedicalVocabulary", deny(DenyReason::Mutating)),
            ("UpdateVocabulary", deny(DenyReason::Mutating)),
            ("UpdateVocabularyFilter", deny(DenyReason::Mutating)),
        ],
    )
}
