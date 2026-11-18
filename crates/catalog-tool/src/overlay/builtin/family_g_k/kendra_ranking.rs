//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kendra-ranking/service.
//! Review is source-only; no live service or emulator support claim.
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

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &["DescribeRescoreExecutionPlan", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "CreateRescoreExecutionPlan",
    "DeleteRescoreExecutionPlan",
    "TagResource",
    "UntagResource",
    "UpdateRescoreExecutionPlan",
];

const PAID: &[&str] = &["Rescore"];
