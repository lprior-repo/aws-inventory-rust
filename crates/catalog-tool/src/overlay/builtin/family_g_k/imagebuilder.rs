use super::policy;
use crate::overlay::{NameError, OperationOverlay, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListImagePipelines",
            tagged(
                "imagePipelineList",
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
            policy::tags(owned("containerRecipeSummaryList")?, "tags")?,
        ),
        (
            "ListImageRecipes",
            policy::tags(owned("imageRecipeSummaryList")?, "tags")?,
        ),
        (
            "ListDistributionConfigurations",
            tagged("distributionConfigurationSummaryList", &["arn"])?,
        ),
        (
            "ListInfrastructureConfigurations",
            tagged("infrastructureConfigurationSummaryList", &["arn"])?,
        ),
        (
            "ListLifecyclePolicies",
            tagged("lifecyclePolicySummaryList", &["arn"])?,
        ),
        (
            "ListComponentBuildVersions",
            tagged("componentSummaryList", &["arn"])?,
        ),
        (
            "ListWorkflowBuildVersions",
            tagged("workflowSummaryList", &["arn"])?,
        ),
        ("ListComponents", owned("componentVersionList")?),
        ("ListImages", owned("imageVersionList")?),
        ("ListWorkflows", owned("workflowVersionList")?),
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

fn listing(items: &str, members: &[&str]) -> Result<OperationOverlay, NameError> {
    let mut plan = policy::enumerate(items, policy::arn("arn")?, Some(("nextToken", "nextToken")))?;
    plan.page_size = Some(25);
    policy::query(plan, members)
}

fn owned(items: &str) -> Result<OperationOverlay, NameError> {
    policy::fixed(listing(items, &["arn"])?, "owner", "Self")
}

fn tagged(items: &str, members: &[&str]) -> Result<OperationOverlay, NameError> {
    policy::tags(listing(items, members)?, "tags")
}

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
