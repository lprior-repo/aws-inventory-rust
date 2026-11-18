use super::super::service;
use super::{defer, deny, textract_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "textract",
        vec![
            ("AnalyzeDocument", deny(DenyReason::Content)),
            ("AnalyzeExpense", deny(DenyReason::Content)),
            ("AnalyzeID", deny(DenyReason::Content)),
            ("CreateAdapter", deny(DenyReason::Mutating)),
            ("CreateAdapterVersion", deny(DenyReason::Mutating)),
            ("DeleteAdapter", deny(DenyReason::Mutating)),
            ("DeleteAdapterVersion", deny(DenyReason::Mutating)),
            ("DetectDocumentText", deny(DenyReason::Content)),
            ("GetAdapter", textract_plans_1::get_adapter()?),
            (
                "GetAdapterVersion",
                textract_plans_1::get_adapter_version()?,
            ),
            ("GetDocumentAnalysis", deny(DenyReason::Content)),
            ("GetDocumentTextDetection", deny(DenyReason::Content)),
            ("GetExpenseAnalysis", deny(DenyReason::Content)),
            ("GetLendingAnalysis", deny(DenyReason::Content)),
            ("GetLendingAnalysisSummary", deny(DenyReason::Content)),
            ("ListAdapters", textract_plans_1::list_adapters()?),
            (
                "ListAdapterVersions",
                textract_plans_1::list_adapter_versions()?,
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("StartDocumentAnalysis", deny(DenyReason::Mutating)),
            ("StartDocumentTextDetection", deny(DenyReason::Mutating)),
            ("StartExpenseAnalysis", deny(DenyReason::Mutating)),
            ("StartLendingAnalysis", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateAdapter", deny(DenyReason::Mutating)),
        ],
    )
}
