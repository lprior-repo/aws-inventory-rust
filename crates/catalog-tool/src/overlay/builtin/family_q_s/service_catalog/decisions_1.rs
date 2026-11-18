// Pinned operation purposes: .local/aws-models/models/service-catalog/service/2015-12-10/service-catalog-2015-12-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListLaunchPaths",
        Decision::Deferred(
            "Required selectors ProductId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LaunchPathSummaries, NextPageToken. Pinned purpose: Lists the paths to the specified product. A path describes how the user gets access to a specified product and is necessary when provisioning a product. A path also determines the constraints that are put on a product. A path is dependent on a specif",
        ),
    ),
    (
        "ListOrganizationPortfolioAccess",
        Decision::Deferred(
            "Required selectors PortfolioId, OrganizationNodeType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OrganizationNodes, NextPageToken. Pinned purpose: Lists the organization nodes that have access to the specified portfolio. This API can only be called by the management account in the organization or by a delegated admin. If a delegated admin is de-registered, they can no longer perform this operat",
        ),
    ),
    (
        "ListPortfolioAccess",
        Decision::Deferred(
            "Required selectors PortfolioId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountIds, NextPageToken. Pinned purpose: Lists the account IDs that have access to the specified portfolio. A delegated admin can list the accounts that have access to the shared portfolio. Note that if a delegated admin is de-registered, they can no longer perform this operation.",
        ),
    ),
    (
        "ListPortfoliosForProduct",
        Decision::Deferred(
            "Required selectors ProductId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PortfolioDetails, NextPageToken. Pinned purpose: Lists all portfolios that the specified product is associated with.",
        ),
    ),
    (
        "ListProvisioningArtifacts",
        Decision::Deferred(
            "Required selectors ProductId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProvisioningArtifactDetails, NextPageToken. Pinned purpose: Lists all provisioning artifacts (also known as versions) for the specified product.",
        ),
    ),
    (
        "ListProvisioningArtifactsForServiceAction",
        Decision::Deferred(
            "Required selectors ServiceActionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProvisioningArtifactViews, NextPageToken. Pinned purpose: Lists all provisioning artifacts (also known as versions) for the specified self-service action.",
        ),
    ),
    (
        "ListRecordHistory",
        Decision::Deferred(
            "Execution history entries may return operational parameters/status payloads; these are not account resource roots and need explicit record-content handling.",
        ),
    ),
    (
        "ListResourcesForTagOption",
        Decision::Deferred(
            "Required selectors TagOptionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceDetails, PageToken. Pinned purpose: Lists the resources associated with the specified TagOption.",
        ),
    ),
    (
        "ListServiceActionsForProvisioningArtifact",
        Decision::Deferred(
            "Required selectors ProductId, ProvisioningArtifactId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceActionSummaries, NextPageToken. Pinned purpose: Returns a paginated list of self-service actions associated with the specified Product ID and Provisioning Artifact ID.",
        ),
    ),
    (
        "ListStackInstancesForProvisionedProduct",
        Decision::Deferred(
            "Required selectors ProvisionedProductId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StackInstances, NextPageToken. Pinned purpose: Returns summary information about stack instances that are associated with the specified CFN_STACKSET type provisioned product. You can filter for stack instances that are associated with a specific Amazon Web Services account name or Region.",
        ),
    ),
    (
        "NotifyProvisionProductEngineWorkflowResult",
        Decision::Deferred(
            "Required selectors WorkflowToken, RecordId, Status, IdempotencyToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Notifies the result of the provisioning engine execution.",
        ),
    ),
    (
        "NotifyTerminateProvisionedProductEngineWorkflowResult",
        Decision::Deferred(
            "Required selectors WorkflowToken, RecordId, Status, IdempotencyToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Notifies the result of the terminate engine execution.",
        ),
    ),
    (
        "NotifyUpdateProvisionedProductEngineWorkflowResult",
        Decision::Deferred(
            "Required selectors WorkflowToken, RecordId, Status, IdempotencyToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Notifies the result of the update engine execution.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Provisions the specified product. A provisioned product is a resourced instance of a product. For example, provisioning a product that's based on an CloudFormation template launches an CloudFormation stack and its underlying resources. You can check the status of this request using DescribeRecord . If the request contains a tag key with an empty list of values, there's a tag conflict for that key.
    ("ProvisionProduct", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Rejects an offer to share the specified portfolio.
    (
        "RejectPortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ScanProvisionedProducts",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProvisionedProducts, NextPageToken. Pinned request/ownership contract: Lists the provisioned products that are available (not terminated). To use additional filtering, see SearchProvisionedProducts .",
        ),
    ),
    (
        "SearchProducts",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProductViewSummaries, ProductViewAggregations, NextPageToken. Pinned request/ownership contract: Gets information about the products to which the caller has access.",
        ),
    ),
    (
        "SearchProductsAsAdmin",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProductViewDetails, NextPageToken. Pinned request/ownership contract: Gets information about the products for the specified portfolio or all products.",
        ),
    ),
    (
        "SearchProvisionedProducts",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ProvisionedProducts, TotalResultsCount, NextPageToken. Pinned request/ownership contract: Gets information about the provisioned products that meet the specified criteria.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Terminates the specified provisioned product. This operation does not delete any records associated with the provisioned product. You can check the status of this request using DescribeRecord .
    (
        "TerminateProvisionedProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified constraint.
    ("UpdateConstraint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified portfolio. You cannot update a product that was shared with you.
    ("UpdatePortfolio", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified portfolio share. You can use this API to enable or disable TagOptions sharing or Principal sharing for an existing portfolio share. The portfolio share cannot be updated if the CreatePortfolioShare operation is IN_PROGRESS , as the share is not available to recipient entities. In this case, you must wait for the portfolio share to be completed. You must provide the accountId
    (
        "UpdatePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified product.
    ("UpdateProduct", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Requests updates to the configuration of the specified provisioned product. If there are tags associated with the object, they cannot be updated or added. Depending on the specific updates requested, this operation can update with no interruption, with some interruption, or replace the provisioned product entirely. You can check the status of this request using DescribeRecord .
    (
        "UpdateProvisionedProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Requests updates to the properties of the specified provisioned product.
    (
        "UpdateProvisionedProductProperties",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified provisioning artifact (also known as a version) for the specified product. You cannot update a provisioning artifact for a product that was shared with you.
    (
        "UpdateProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a self-service action.
    (
        "UpdateServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified TagOption.
    ("UpdateTagOption", Decision::Denied(DenyReason::Mutating)),
];
