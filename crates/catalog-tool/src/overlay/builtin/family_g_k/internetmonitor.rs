//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/internetmonitor/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListMonitors", policy::query(policy::enumerate("Monitors", policy::arn("MonitorArn")?, Some(("NextToken", "NextToken")))?, &["MonitorArn"])?),
        ("GetMonitor", policy::query(policy::tags(policy::bind(policy::enumerate("$", policy::arn("MonitorArn")?, None)?, "MonitorName", "ListMonitors", "MonitorName")?, "Tags")?, &["MonitorArn"])?),
    ];
    policy::finish(
        "internetmonitor", approved, DEFERRED,
        "Monitor metadata is approved with a name-bound detail call. Query-result data is denied; health/internet events and query lifecycle are not approved resource discovery.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetHealthEvent", "GetInternetEvent", "GetQueryStatus", "ListHealthEvents", "ListInternetEvents",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateMonitor", "DeleteMonitor", "StartQuery", "StopQuery", "TagResource", "UntagResource",
    "UpdateMonitor",
];

const CONTENT: &[&str] = &[
    "GetQueryResults",
];
