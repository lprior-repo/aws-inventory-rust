//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iottwinmaker/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListWorkspaces",
            policy::query(
                policy::enumerate(
                    "workspaceSummaries",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn"],
            )?,
        ),
        (
            "GetWorkspace",
            policy::query(
                policy::bind(
                    policy::enumerate("$", policy::arn("arn")?, None)?,
                    "workspaceId",
                    "ListWorkspaces",
                    "workspaceId",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListEntities",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "entitySummaries",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "workspaceId",
                    "ListWorkspaces",
                    "workspaceId",
                )?,
                &["arn", "entityId", "parentEntityId"],
            )?,
        ),
        (
            "ListScenes",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "sceneSummaries",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "workspaceId",
                    "ListWorkspaces",
                    "workspaceId",
                )?,
                &["arn", "sceneId", "contentLocation"],
            )?,
        ),
        (
            "ListSyncJobs",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "syncJobSummaries",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "workspaceId",
                    "ListWorkspaces",
                    "workspaceId",
                )?,
                &["arn", "workspaceId", "syncSource"],
            )?,
        ),
    ];
    policy::finish(
        "iottwinmaker",
        approved,
        DEFERRED,
        "Workspace list, workspace-bound metadata detail, entity, scene and sync-job summaries are approved. Scene contentLocation is an s3:// reference, never scene bytes; nested SyncJobStatus contains state/error metadata only. Property values/history and query results are content and denied. ListComponentTypes remains deferred: pinned namespace filter documents membership only, not a customer-owner predicate; summaries lack an owner/isAWSComponentType field. Do not inventory AWS namespace templates as customer-created types.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetComponentType",
    "GetEntity",
    "GetMetadataTransferJob",
    "GetPricingPlan",
    "GetScene",
    "GetSyncJob",
    "ListComponentTypes",
    "ListComponents",
    "ListMetadataTransferJobs",
    "ListProperties",
    "ListSyncResources",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "BatchPutPropertyValues",
    "CancelMetadataTransferJob",
    "CreateComponentType",
    "CreateEntity",
    "CreateMetadataTransferJob",
    "CreateScene",
    "CreateSyncJob",
    "CreateWorkspace",
    "DeleteComponentType",
    "DeleteEntity",
    "DeleteScene",
    "DeleteSyncJob",
    "DeleteWorkspace",
    "TagResource",
    "UntagResource",
    "UpdateComponentType",
    "UpdateEntity",
    "UpdatePricingPlan",
    "UpdateScene",
    "UpdateWorkspace",
];

const CONTENT: &[&str] = &[
    "ExecuteQuery",
    "GetPropertyValue",
    "GetPropertyValueHistory",
];
