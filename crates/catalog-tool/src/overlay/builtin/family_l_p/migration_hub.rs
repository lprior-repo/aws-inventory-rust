use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateCreatedArtifact",
            "AssociateDiscoveredResource",
            "AssociateSourceResource",
            "CreateProgressUpdateStream",
            "DeleteProgressUpdateStream",
            "DisassociateCreatedArtifact",
            "DisassociateDiscoveredResource",
            "DisassociateSourceResource",
            "ImportMigrationTask",
            "NotifyApplicationState",
            "NotifyMigrationTaskState",
            "PutResourceAttributes",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeApplicationState",
            "DescribeMigrationTask",
            "ListCreatedArtifacts",
            "ListDiscoveredResources",
            "ListMigrationTaskUpdates",
            "ListMigrationTasks",
            "ListSourceResources",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Remaining migration-task detail requires progress-stream/task context; inventory emits stream and application-state summaries only.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListProgressUpdateStreams",
        items: "ProgressUpdateStreamSummaryList",
        id: Id::Key(&["ProgressUpdateStreamName"]),
        scope: Scope::HomeRegion,
        tokens: Some(("NextToken", "NextToken")),
        query: &["ProgressUpdateStreamName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListApplicationStates",
        items: "ApplicationStateList",
        id: Id::Key(&["ApplicationId"]),
        scope: Scope::HomeRegion,
        tokens: Some(("NextToken", "NextToken")),
        query: &["ApplicationId", "ApplicationStatus"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("migration-hub", GROUPS, RESOURCES)
}
