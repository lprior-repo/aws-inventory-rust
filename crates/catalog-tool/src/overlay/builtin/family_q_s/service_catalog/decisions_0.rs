// Pinned operation purposes: .local/aws-models/models/service-catalog/service/2015-12-10/service-catalog-2015-12-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Accepts an offer to share the specified portfolio.
    (
        "AcceptPortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates the specified budget with the specified resource.
    (
        "AssociateBudgetWithResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates the specified principal ARN with the specified portfolio. If you share the portfolio with principal name sharing enabled, the PrincipalARN association is included in the share. The PortfolioID , PrincipalARN , and PrincipalType parameters are required. You can associate a maximum of 10 Principals with a portfolio using PrincipalType as IAM_PATTERN . When you associate a principal with p
    (
        "AssociatePrincipalWithPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates the specified product with the specified portfolio. A delegated admin is authorized to invoke this command.
    (
        "AssociateProductWithPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates a self-service action with a provisioning artifact.
    (
        "AssociateServiceActionWithProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associate the specified TagOption with the specified portfolio or product.
    (
        "AssociateTagOptionWithResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates multiple self-service actions with provisioning artifacts.
    (
        "BatchAssociateServiceActionWithProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a batch of self-service actions from the specified provisioning artifact.
    (
        "BatchDisassociateServiceActionFromProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Copies the specified source product to the specified target product or a new product. You can copy a product to the same account or another account. You can copy a product to the same Region or another Region. If you copy a product to another account, you must first share the product in a portfolio using CreatePortfolioShare . This operation is performed asynchronously. To track the progress of th
    ("CopyProduct", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a constraint. A delegated admin is authorized to invoke this command.
    ("CreateConstraint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a portfolio. A delegated admin is authorized to invoke this command.
    ("CreatePortfolio", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Shares the specified portfolio with the specified account or organization node. Shares to an organization node can only be created by the management account of an organization or by a delegated administrator. You can share portfolios to an organization, an organizational unit, or a specific account. Note that if a delegated admin is de-registered, they can no longer create portfolio shares. AWSOrg
    (
        "CreatePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a product. A delegated admin is authorized to invoke this command. The user or role that performs this operation must have the cloudformation:GetTemplate IAM policy permission. This policy permission is required when using the ImportFromPhysicalId template source in the information data section.
    ("CreateProduct", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a plan. A plan includes the list of resources to be created (when provisioning a new product) or modified (when updating a provisioned product) when the plan is executed. You can create one plan for each provisioned product. To create a plan for an existing provisioned product, the product status must be AVAILABLE or TAINTED. To view the resource changes in the change set, use DescribeProv
    (
        "CreateProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a provisioning artifact (also known as a version) for the specified product. You cannot create a provisioning artifact for a product that was shared with you. The user or role that performs this operation must have the cloudformation:GetTemplate IAM policy permission. This policy permission is required when using the ImportFromPhysicalId template source in the information data section.
    (
        "CreateProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a self-service action.
    (
        "CreateServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a TagOption.
    ("CreateTagOption", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified constraint. A delegated admin is authorized to invoke this command.
    ("DeleteConstraint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified portfolio. You cannot delete a portfolio if it was shared with you or if it has associated products, users, constraints, or shared accounts. A delegated admin is authorized to invoke this command.
    ("DeletePortfolio", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops sharing the specified portfolio with the specified account or organization node. Shares to an organization node can only be deleted by the management account of an organization or by a delegated administrator. Note that if a delegated admin is de-registered, portfolio shares created from that account are removed.
    (
        "DeletePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified product. You cannot delete a product if it was shared with you or is associated with a portfolio. A delegated admin is authorized to invoke this command.
    ("DeleteProduct", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified plan.
    (
        "DeleteProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified provisioning artifact (also known as a version) for the specified product. You cannot delete a provisioning artifact associated with a product that was shared with you. You cannot delete the last provisioning artifact for a product, because a product must have at least one provisioning artifact.
    (
        "DeleteProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a self-service action.
    (
        "DeleteServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified TagOption. You cannot delete a TagOption if it is associated with a product or portfolio.
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
    // Changes service state rather than reading inventory. Pinned purpose: Disable portfolio sharing through the Organizations service. This command will not delete your current shares, but prevents you from creating new shares throughout your organization. Current shares are not kept in sync with your organization structure if the structure changes after calling this API. Only the management account in the organization can call this API. You cannot call this API if ther
    (
        "DisableAWSOrganizationsAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates the specified budget from the specified resource.
    (
        "DisassociateBudgetFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a previously associated principal ARN from a specified portfolio. The PrincipalType and PrincipalARN must match the AssociatePrincipalWithPortfolio call request details. For example, to disassociate an association created with a PrincipalARN of PrincipalType IAM you must use the PrincipalType IAM when calling DisassociatePrincipalFromPortfolio . For portfolios that have been shared w
    (
        "DisassociatePrincipalFromPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates the specified product from the specified portfolio. A delegated admin is authorized to invoke this command.
    (
        "DisassociateProductFromPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates the specified self-service action association from the specified provisioning artifact.
    (
        "DisassociateServiceActionFromProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates the specified TagOption from the specified resource.
    (
        "DisassociateTagOptionFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable portfolio sharing feature through Organizations. This API will allow Service Catalog to receive updates on your organization in order to sync your shares with the current structure. This API can only be called by the management account in the organization. When you call this API, Service Catalog calls organizations:EnableAWSServiceAccess on your behalf so that your shares stay in sync with
    (
        "EnableAWSOrganizationsAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Provisions or modifies a product based on the resource changes for the specified plan.
    (
        "ExecuteProvisionedProductPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Executes a self-service action against a provisioned product.
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
    // Returns arbitrary provisioned product output values, potentially customer secrets; projecting an output key does not authorize fetching the values.
    (
        "GetProvisionedProductOutputs",
        Decision::Denied(DenyReason::Secret),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Requests the import of a resource as an Service Catalog provisioned product that is associated to an Service Catalog product and provisioning artifact. Once imported, all supported governance actions are supported on the provisioned product. Resource import only supports CloudFormation stack ARNs. CloudFormation StackSets, and non-root nested stacks, are not supported. The CloudFormation stack mus
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
