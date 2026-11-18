//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/imagebuilder/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListImagePipelines",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "imagePipelineList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "imageRecipeArn",
                    "containerRecipeArn",
                    "infrastructureConfigurationArn",
                    "distributionConfigurationArn",
                ],
            )?,
        ),
        (
            "ListContainerRecipes",
            policy::query(
                policy::tags(
                    policy::fixed(
                        policy::enumerate(
                            "containerRecipeSummaryList",
                            policy::arn("arn")?,
                            Some(("nextToken", "nextToken")),
                        )?,
                        "owner",
                        "Self",
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListImageRecipes",
            policy::query(
                policy::tags(
                    policy::fixed(
                        policy::enumerate(
                            "imageRecipeSummaryList",
                            policy::arn("arn")?,
                            Some(("nextToken", "nextToken")),
                        )?,
                        "owner",
                        "Self",
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListDistributionConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "distributionConfigurationSummaryList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListInfrastructureConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "infrastructureConfigurationSummaryList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListLifecyclePolicies",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "lifecyclePolicySummaryList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListComponentBuildVersions",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "componentSummaryList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListWorkflowBuildVersions",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "workflowSummaryList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListComponents",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "componentVersionList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "owner",
                    "Self",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListImages",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "imageVersionList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "owner",
                    "Self",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListWorkflows",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "workflowVersionList",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "owner",
                    "Self",
                )?,
                &["arn"],
            )?,
        ),
    ];
    policy::finish(
        "imagebuilder",
        approved,
        DEFERRED,
        "Owned safe summaries include component/workflow builds and separate component/image/workflow semantic versions, owner=Self and byName's modeled false default. Component build lists exclude deprecated builds. Owned deprecated images remain returned. ListImageBuildVersions is content-denied: ImageSummary.state.failureContext.componentFailure.errorMessage can contain failed-step output; projecting metadata cannot make the full response safe. Bodies and execution content stay unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetComponentPolicy",
    "GetContainerRecipePolicy",
    "GetDistributionConfiguration",
    "GetImage",
    "GetImagePipeline",
    "GetImagePolicy",
    "GetImageRecipePolicy",
    "GetInfrastructureConfiguration",
    "GetLifecycleExecution",
    "GetLifecyclePolicy",
    "GetMarketplaceResource",
    "GetWorkflowExecution",
    "ListImagePackages",
    "ListImagePipelineImages",
    "ListImageScanFindingAggregations",
    "ListImageScanFindings",
    "ListLifecycleExecutionResources",
    "ListLifecycleExecutions",
    "ListTagsForResource",
    "ListWaitingWorkflowSteps",
    "ListWorkflowExecutions",
    "ListWorkflowStepExecutions",
];

const MUTATING: &[&str] = &[
    "CancelImageCreation",
    "CancelLifecycleExecution",
    "CreateComponent",
    "CreateContainerRecipe",
    "CreateDistributionConfiguration",
    "CreateImage",
    "CreateImagePipeline",
    "CreateImageRecipe",
    "CreateInfrastructureConfiguration",
    "CreateLifecyclePolicy",
    "CreateWorkflow",
    "DeleteComponent",
    "DeleteContainerRecipe",
    "DeleteDistributionConfiguration",
    "DeleteImage",
    "DeleteImagePipeline",
    "DeleteImageRecipe",
    "DeleteInfrastructureConfiguration",
    "DeleteLifecyclePolicy",
    "DeleteWorkflow",
    "DistributeImage",
    "ImportComponent",
    "ImportDiskImage",
    "ImportVmImage",
    "PutComponentPolicy",
    "PutContainerRecipePolicy",
    "PutImagePolicy",
    "PutImageRecipePolicy",
    "RetryImage",
    "SendWorkflowStepAction",
    "StartImagePipelineExecution",
    "StartResourceStateUpdate",
    "TagResource",
    "UntagResource",
    "UpdateDistributionConfiguration",
    "UpdateImagePipeline",
    "UpdateInfrastructureConfiguration",
    "UpdateLifecyclePolicy",
];

const CONTENT: &[&str] = &[
    "GetComponent",
    "GetContainerRecipe",
    "GetImageRecipe",
    "GetWorkflow",
    "GetWorkflowStepExecution",
    "ListImageBuildVersions",
];
