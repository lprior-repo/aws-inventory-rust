use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AddAttachmentsToSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AddCommunicationToCase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CompleteAttachmentUpload",
        Decision::Deferred(
            "Required selectors uploadId, completedUploads are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members uploadStatus. Pinned purpose: Completes an attachment upload that was started with GetAttachmentUploadLinks . After you upload a part of the file to its presigned Amazon S3 URL, call CompleteAttachmentUpload with the partIndex and eTag of that part. You can include one part per c",
        ),
    ),
    ("CreateCase", Decision::Denied(DenyReason::Mutating)),
    ("DescribeAttachment", Decision::Denied(DenyReason::Content)),
    (
        "DescribeAttachmentUploadStatus",
        Decision::Deferred(
            "Required selectors uploadId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members uploadStatus, fileName, uploadProgress. Pinned purpose: Returns the current status, file name, and progress of a multipart attachment upload that was started with GetAttachmentUploadLinks . Use this operation to track where an upload is in the workflow. While parts are still being uploaded and reported th",
        ),
    ),
    ("DescribeCases", Decision::Denied(DenyReason::Content)),
    (
        "DescribeCommunications",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "DescribeCreateCaseOptions",
        Decision::Deferred(
            "Required selectors issueType, serviceCode, language, categoryCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members languageAvailability, communicationTypes. Pinned purpose: Returns a list of CreateCaseOption types along with the corresponding supported hours and language availability. You can specify the language categoryCode , issueType and serviceCode used to retrieve the CreateCaseOptions. You must have an Amazon Web",
        ),
    ),
    (
        "DescribeServices",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeSeverityLevels",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeSupportedLanguages",
        Decision::Deferred(
            "Required selectors issueType, serviceCode, categoryCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members supportedLanguages. Pinned purpose: Returns a list of supported languages for a specified categoryCode , issueType and serviceCode . The returned supported languages will include a ISO 639-1 code for the language , and the language display name. You must have an Amazon Web Services Bus",
        ),
    ),
    (
        "DescribeTrustedAdvisorCheckRefreshStatuses",
        Decision::Deferred(
            "Required selectors checkIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members statuses. Pinned purpose: Returns the refresh status of the Trusted Advisor checks that have the specified check IDs. You can get the check IDs by calling the DescribeTrustedAdvisorChecks operation. Some checks are refreshed automatically, and you can't return their refresh s",
        ),
    ),
    (
        "DescribeTrustedAdvisorCheckResult",
        Decision::Deferred(
            "Required selectors checkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members result. Pinned purpose: Returns the results of the Trusted Advisor check that has the specified check ID. You can get the check IDs by calling the DescribeTrustedAdvisorChecks operation. The response contains a TrustedAdvisorCheckResult object, which contains these three ob",
        ),
    ),
    (
        "DescribeTrustedAdvisorCheckSummaries",
        Decision::Deferred(
            "Required selectors checkIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members summaries. Pinned purpose: Returns the results for the Trusted Advisor check summaries for the check IDs that you specified. You can get the check IDs by calling the DescribeTrustedAdvisorChecks operation. The response contains an array of TrustedAdvisorCheckSummary objects. Y",
        ),
    ),
    (
        "DescribeTrustedAdvisorChecks",
        Decision::Deferred(
            "Required selectors language are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members checks. Pinned purpose: Returns information about all available Trusted Advisor checks, including the name, ID, category, description, and metadata. You must specify a language code. The response contains a TrustedAdvisorCheckDescription object for each check. You must set ",
        ),
    ),
    (
        "GetAttachmentDownloadLink",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetAttachmentUploadLinks",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "RefreshTrustedAdvisorCheck",
        Decision::Deferred(
            "Required selectors checkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status. Pinned purpose: Refreshes the Trusted Advisor check that you specify using the check ID. You can get the check IDs by calling the DescribeTrustedAdvisorChecks operation. Some checks are refreshed automatically. If you call the RefreshTrustedAdvisorCheck operation to",
        ),
    ),
    (
        "ResolveCase",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members initialCaseStatus, finalCaseStatus. Pinned request/ownership contract: Resolves a support case. This operation takes a caseId and returns the initial and final state of the case. You must have an Amazon Web Services Business Support+, Amazon Web Services Enterprise Support, or Amazon Web Services Unified Operations plan to use the Amazon Web Services Support API. If you're in an Amazon Web Services Region that doesn't offer one of these Amazon Web Services Support plans, or if you haven't transitioned to one of thes",
        ),
    ),
];
