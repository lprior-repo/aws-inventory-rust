use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateDefaultView",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchGetView",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Views, Errors. Pinned request/ownership contract: Retrieves details about a list of views.",
        ),
    ),
    ("CreateIndex", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateResourceExplorerSetup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateView", Decision::Denied(DenyReason::Mutating)),
    ("DeleteIndex", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteResourceExplorerSetup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteView", Decision::Denied(DenyReason::Mutating)),
    (
        "DisassociateDefaultView",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAccountLevelServiceConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members OrgConfiguration. Pinned request/ownership contract: Retrieves the status of your account's Amazon Web Services service access, and validates the service linked role required to access the multi-account search feature. Only the management account can invoke this API call.",
        ),
    ),
    (
        "GetDefaultView",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ViewArn. Pinned request/ownership contract: Retrieves the Amazon Resource Name (ARN) of the view that is the default for the Amazon Web Services Region in which you call this operation. You can then call GetView to retrieve the details of that view.",
        ),
    ),
    (
        "GetManagedView",
        Decision::Deferred(
            "Required selectors ManagedViewArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ManagedView. Pinned purpose: Retrieves details of the specified Amazon Web Services-managed view .",
        ),
    ),
    (
        "GetResourceExplorerSetup",
        Decision::Deferred(
            "Required selectors TaskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Regions, NextToken. Pinned purpose: Retrieves the status and details of a Resource Explorer setup operation. This operation returns information about the progress of creating or deleting Resource Explorer configurations across Regions.",
        ),
    ),
    (
        "GetServiceIndex",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Arn, Type. Pinned request/ownership contract: Retrieves information about the Resource Explorer index in the current Amazon Web Services Region. This operation returns the ARN and type of the index if one exists.",
        ),
    ),
    (
        "GetServiceView",
        Decision::Deferred(
            "Required selectors ServiceViewArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members View. Pinned purpose: Retrieves details about a specific Resource Explorer service view. This operation returns the configuration and properties of the specified view.",
        ),
    ),
    (
        "GetView",
        Decision::Deferred(
            "Required selectors ViewArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members View, Tags. Pinned purpose: Retrieves details of the specified view.",
        ),
    ),
    (
        "ListIndexes",
        Decision::Deferred(
            "Returns indexes across Regions. Per-item Region and Arn do not belong to one request region; account scope would incorrectly globalize regional indexes. Requires per-item scope routing, not an arbitrary local-region key.",
        ),
    ),
    (
        "ListIndexesForMembers",
        Decision::Deferred(
            "Required selectors AccountIdList are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Indexes, NextToken. Pinned purpose: Retrieves a list of a member's indexes in all Amazon Web Services Regions that are currently collecting resource information for Amazon Web Services Resource Explorer. Only the management account or a delegated administrator with service access enabl",
        ),
    ),
    (
        "ListManagedViews",
        Decision::Deferred(
            "AWS-managed view catalog, not account-owned mutable view inventory; ListViews is the customer-owned view root.",
        ),
    ),
    (
        "ListResources",
        Decision::Deferred(
            "Heterogeneous Resource.Arn plus OwningAccountId/Region spans resource kinds and scopes. A static operation kind/scope would create incorrect resources; requires per-item canonical-kind and scope dispatch.",
        ),
    ),
    (
        "ListServiceIndexes",
        Decision::Deferred(
            "Returns service-managed indexes with per-item Region; requires per-item scope routing rather than local-region or global relabeling.",
        ),
    ),
    (
        "ListServiceViews",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListSupportedResourceTypes",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags that are attached to the specified resource.",
        ),
    ),
    (
        "Search",
        Decision::Deferred(
            "Required selectors QueryString are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Resources, NextToken, ViewArn, Count. Pinned purpose: Searches for resources and displays details about all resources that match the specified criteria. You must specify a query string. All search queries must use a view. If you don't explicitly specify a view, then Amazon Web Services Resource Explorer",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateIndexType", Decision::Denied(DenyReason::Mutating)),
    ("UpdateView", Decision::Denied(DenyReason::Mutating)),
];
