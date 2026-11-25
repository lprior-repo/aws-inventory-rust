use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![(
        "ListRescoreExecutionPlans",
        policy::query(
            policy::enumerate(
                "SummaryItems",
                policy::key("Id")?,
                Some(("NextToken", "NextToken")),
            )?,
            &["Id"],
        )?,
    )];
    policy::finish(
        "kendra-ranking",
        approved,
        DEFERRED,
        "Execution-plan identifiers/status summaries are metadata. Rescore is billed semantic ranking and denied. Remaining detail/tag expansion needs separate review.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Paid, PAID),
        ],
    )
}

const DEFERRED: &[&str] = &["DescribeRescoreExecutionPlan", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "CreateRescoreExecutionPlan",
    "DeleteRescoreExecutionPlan",
    "TagResource",
    "UntagResource",
    "UpdateRescoreExecutionPlan",
];

const PAID: &[&str] = &["Rescore"];
