use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribeReportCreation",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Status, S3Location, StartDate, ErrorMessage. Pinned request/ownership contract: Describes the status of the StartReportCreation operation. You can call this operation only from the organization's management account and from the us-east-1 Region.",
        ),
    ),
    (
        "GetComplianceSummary",
        Decision::Deferred(
            "ComplianceSummary aggregates by optional target, Region and resource type, rather than identifying an account-owned resource. Cross-account/Region aggregation cannot be given the caller request scope.",
        ),
    ),
    (
        "GetResources",
        Decision::Deferred(
            "Heterogeneous ResourceTagMapping.ResourceARN rows have no single canonical resource kind. This is a tag-observation source, not a new AWS resource type; static per-operation kind mapping cannot attach each row to its existing canonical resource identity.",
        ),
    ),
    (
        "GetTagKeys",
        Decision::Deferred(
            "Unique tag-key vocabulary is tag metadata, not independently owned AWS resource inventory.",
        ),
    ),
    (
        "GetTagValues",
        Decision::Deferred(
            "Required selectors Key are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PaginationToken, TagValues. Pinned purpose: Returns all tag values for the specified key that are used in the specified Amazon Web Services Region for the calling account. This operation supports pagination, where the response can be sent in multiple pages. You should check the PaginationToken",
        ),
    ),
    (
        "ListRequiredTags",
        Decision::Deferred(
            "Required-tag policy aggregates span organization targets and report output; not an account-owned resource root with a stable ARN.",
        ),
    ),
    (
        "StartReportCreation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResources", Decision::Denied(DenyReason::Mutating)),
    ("UntagResources", Decision::Denied(DenyReason::Mutating)),
];
