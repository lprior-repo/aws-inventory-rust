use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptPortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateBudgetWithResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociatePrincipalWithPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateProductWithPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateServiceActionWithProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateTagOptionWithResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchAssociateServiceActionWithProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDisassociateServiceActionFromProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CopyProduct", Decision::Denied(DenyReason::Mutating)),
    ("CreateConstraint", Decision::Denied(DenyReason::Mutating)),
    ("CreatePortfolio", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateProduct", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTagOption", Decision::Denied(DenyReason::Mutating)),
    ("DeleteConstraint", Decision::Denied(DenyReason::Mutating)),
    ("DeletePortfolio", Decision::Denied(DenyReason::Mutating)),
    (
        "DeletePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteProduct", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTagOption", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeConstraint",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConstraintDetail, ConstraintParameters, Status. Pinned purpose: Gets information about the specified constraint.",
        ),
    ),
    (
        "DescribeCopyProductStatus",
        Decision::Deferred(
            "Required selectors CopyProductToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CopyProductStatus, TargetProductId, StatusDetail. Pinned purpose: Gets the status of the specified copy product operation.",
        ),
    ),
    (
        "DescribePortfolio",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PortfolioDetail, Tags, TagOptions, Budgets. Pinned purpose: Gets information about the specified portfolio. A delegated admin is authorized to invoke this command.",
        ),
    ),
    (
        "DescribePortfolioShareStatus",
        Decision::Deferred(
            "Required selectors PortfolioShareToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PortfolioShareToken, PortfolioId, OrganizationNodeValue, Status, ShareDetails. Pinned purpose: Gets the status of the specified portfolio share operation. This API can only be called by the management account in the organization or by a delegated admin.",
        ),
    ),
    (
        "DescribePortfolioShares",
        Decision::Deferred(
            "Required selectors PortfolioId, Type are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextPageToken, PortfolioShareDetails. Pinned purpose: Returns a summary of each of the portfolio shares that were created for the specified portfolio. You can use this API to determine which accounts or organizational nodes this portfolio have been shared, whether the recipient entity has imported the s",
        ),
    ),
    (
        "DescribeProduct",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProductViewSummary, ProvisioningArtifacts, Budgets, LaunchPaths. Pinned request/ownership contract: Gets information about the specified product. Running this operation with administrator access results in a failure. DescribeProductAsAdmin should be used instead.",
        ),
    ),
    (
        "DescribeProductAsAdmin",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProductViewDetail, ProvisioningArtifactSummaries, Tags, TagOptions, Budgets. Pinned request/ownership contract: Gets information about the specified product. This operation is run with administrator access.",
        ),
    ),
    (
        "DescribeProductView",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProductViewSummary, ProvisioningArtifacts. Pinned purpose: Gets information about the specified product.",
        ),
    ),
    (
        "DescribeProvisionedProduct",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProvisionedProductDetail, CloudWatchDashboards. Pinned request/ownership contract: Gets information about the specified provisioned product.",
        ),
    ),
    (
        "DescribeProvisionedProductPlan",
        Decision::Deferred(
            "Required selectors PlanId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProvisionedProductPlanDetails, ResourceChanges, NextPageToken. Pinned purpose: Gets information about the resource changes for the specified plan.",
        ),
    ),
    (
        "DescribeProvisioningArtifact",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProvisioningArtifactDetail, Info, Status, ProvisioningArtifactParameters. Pinned request/ownership contract: Gets information about the specified provisioning artifact (also known as a version) for the specified product.",
        ),
    ),
    (
        "DescribeProvisioningParameters",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProvisioningArtifactParameters, ConstraintSummaries, UsageInstructions, TagOptions, ProvisioningArtifactPreferences, ProvisioningArtifactOutputs, ProvisioningArtifactOutputKeys. Pinned request/ownership contract: Gets information about the configuration required to provision the specified product using the specified provisioning artifact. If the output contains a TagOption key with an empty list of values, there is a TagOption conflict for that key. The end user cannot take action to fix the conflict, and launch is not blocked. In subsequent calls to ProvisionProduct , do not include conflicted TagOption keys as tags, or this causes the error \"Parameter v",
        ),
    ),
    (
        "DescribeRecord",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RecordDetail, RecordOutputs, NextPageToken. Pinned purpose: Gets information about the specified request operation. Use this operation after calling a request operation (for example, ProvisionProduct , TerminateProvisionedProduct , or UpdateProvisionedProduct ). If a provisioned product was transferred to a n",
        ),
    ),
    (
        "DescribeServiceAction",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceActionDetail. Pinned purpose: Describes a self-service action.",
        ),
    ),
    (
        "DescribeServiceActionExecutionParameters",
        Decision::Deferred(
            "Required selectors ProvisionedProductId, ServiceActionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceActionParameters. Pinned purpose: Finds the default parameters for a specific self-service action on a specific provisioned product and returns a map of the results to the user.",
        ),
    ),
    (
        "DescribeTagOption",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TagOptionDetail. Pinned purpose: Gets information about the specified TagOption.",
        ),
    ),
    (
        "DisableAWSOrganizationsAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateBudgetFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociatePrincipalFromPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateProductFromPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateServiceActionFromProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateTagOptionFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableAWSOrganizationsAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExecuteProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExecuteProvisionedProductServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAWSOrganizationsAccessStatus",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AccessStatus. Pinned request/ownership contract: Get the Access Status for Organizations portfolio share feature. This API can only be called by the management account in the organization or by a delegated admin.",
        ),
    ),
    (
        "GetProvisionedProductOutputs",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ImportAsProvisionedProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListAcceptedPortfolioShares",
        Decision::Deferred(
            "Returned portfolios are shared by another provider account. They are acceptance associations, not caller ownership of the provider Portfolio ARN; need recipient/provider association identity.",
        ),
    ),
    (
        "ListBudgetsForResource",
        Decision::Deferred(
            "Required selectors ResourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Budgets, NextPageToken. Pinned purpose: Lists all the budgets associated to the specified resource.",
        ),
    ),
];
