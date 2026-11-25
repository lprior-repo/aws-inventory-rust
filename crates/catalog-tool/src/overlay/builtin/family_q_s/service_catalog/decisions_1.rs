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
    ("ProvisionProduct", Decision::Denied(DenyReason::Mutating)),
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
    (
        "TerminateProvisionedProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateConstraint", Decision::Denied(DenyReason::Mutating)),
    ("UpdatePortfolio", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdatePortfolioShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateProduct", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateProvisionedProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateProvisionedProductProperties",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateProvisioningArtifact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateServiceAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTagOption", Decision::Denied(DenyReason::Mutating)),
];
