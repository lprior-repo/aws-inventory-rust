use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelBatchJobExecution",
            "CreateApplication",
            "CreateDataSetExportTask",
            "CreateDataSetImportTask",
            "CreateDeployment",
            "CreateEnvironment",
            "DeleteApplication",
            "DeleteApplicationFromEnvironment",
            "DeleteEnvironment",
            "StartApplication",
            "StartBatchJob",
            "StopApplication",
            "TagResource",
            "UntagResource",
            "UpdateApplication",
            "UpdateEnvironment",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetApplication",
            "GetApplicationVersion",
            "GetDataSetDetails",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetBatchJobExecution", "ListBatchJobRestartPoints"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (applicationId, executionId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDataSetExportTask", "GetDataSetImportTask"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (applicationId, taskId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDeployment"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (deploymentId, applicationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetSignedBluinsightsUrl", "ListEngineVersions"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "ListApplicationVersions",
            "ListBatchJobDefinitions",
            "ListBatchJobExecutions",
            "ListDataSetExportHistory",
            "ListDataSetImportHistory",
            "ListDataSets",
            "ListDeployments",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (applicationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListApplications",
        items: "applications",
        id: Id::Arn("applicationArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "applicationArn",
            "name",
            "applicationId",
            "status",
            "engineType",
            "environmentId",
            "versionStatus",
            "deploymentStatus",
            "roleArn",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListEnvironments",
        items: "environments",
        id: Id::Arn("environmentArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "environmentArn",
            "name",
            "environmentId",
            "instanceType",
            "status",
            "engineType",
            "networkType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetEnvironment",
        items: "$",
        id: Id::Arn("environmentArn"),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "environmentArn",
            "name",
            "environmentId",
            "instanceType",
            "status",
            "engineType",
            "vpcId",
            "loadBalancerArn",
            "kmsKeyId",
            "networkType",
        ],
        tags: Some("tags"),
        bindings: &[("environmentId", "ListEnvironments", "environmentId")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("m2", GROUPS, RESOURCES)
}
