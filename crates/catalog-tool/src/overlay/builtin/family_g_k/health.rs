//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/health/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "health",
        approved,
        DEFERRED,
        "Event descriptions and affected-entity data are customer operational content; event types/lifecycle tables are reference data. Remaining organization/aggregate calls are not resource inventory and no callable policy is approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
            (policy::DenyReason::ReferenceData, REFERENCEDATA),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeAffectedAccountsForOrganization",
    "DescribeEntityAggregates",
    "DescribeEntityAggregatesForOrganization",
    "DescribeEventAggregates",
    "DescribeHealthServiceStatusForOrganization",
];

const MUTATING: &[&str] = &[
    "DisableHealthServiceAccessForOrganization",
    "EnableHealthServiceAccessForOrganization",
];

const CONTENT: &[&str] = &[
    "DescribeAffectedEntities",
    "DescribeAffectedEntitiesForOrganization",
    "DescribeEventDetails",
    "DescribeEventDetailsForOrganization",
    "DescribeEvents",
    "DescribeEventsForOrganization",
];

const REFERENCEDATA: &[&str] = &["DescribeEventTypes", "DescribeServiceLifecycle"];
