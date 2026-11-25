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
