// Pinned Smithy source: .local/aws-models/models/mwaa/service/2020-07-01/mwaa-2020-07-01.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateCliToken", "CreateEnvironment", "CreateWebLoginToken", "DeleteEnvironment", 
            "InvokeRestApi", "PublishMetrics", "TagResource", "UntagResource", 
            "UpdateEnvironment",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetEnvironment",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Environment summaries are safe; GetEnvironment exposes arbitrary Airflow configuration values that may contain credentials."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists the Amazon Managed Workflows for Apache Airflow (MWAA) environments.
    Resource {
        operation: "ListEnvironments",
        items: "Environments", id: Id::Key(&["$"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["$"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mwaa", GROUPS, RESOURCES)
}
