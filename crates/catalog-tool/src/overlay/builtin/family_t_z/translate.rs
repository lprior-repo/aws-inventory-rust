use super::super::service;
use super::{defer, deny, translate_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "translate",
        vec![
            ("CreateParallelData", deny(DenyReason::Mutating)),
            ("DeleteParallelData", deny(DenyReason::Mutating)),
            ("DeleteTerminology", deny(DenyReason::Mutating)),
            (
                "DescribeTextTranslationJob",
                translate_plans_1::describe_text_translation_job()?,
            ),
            ("GetParallelData", deny(DenyReason::Content)),
            ("GetTerminology", deny(DenyReason::Content)),
            ("ImportTerminology", deny(DenyReason::Mutating)),
            ("ListLanguages", deny(DenyReason::ReferenceData)),
            ("ListParallelData", translate_plans_1::list_parallel_data()?),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "ListTerminologies",
                translate_plans_1::list_terminologies()?,
            ),
            (
                "ListTextTranslationJobs",
                translate_plans_1::list_text_translation_jobs()?,
            ),
            ("StartTextTranslationJob", deny(DenyReason::Mutating)),
            ("StopTextTranslationJob", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            (
                "TranslateDocument",
                defer(
                    "Missing executable inventory binding for Document, SourceLanguageCode, TargetLanguageCode: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "TranslateText",
                defer(
                    "Missing executable inventory binding for Text, SourceLanguageCode, TargetLanguageCode: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateParallelData", deny(DenyReason::Mutating)),
        ],
    )
}
