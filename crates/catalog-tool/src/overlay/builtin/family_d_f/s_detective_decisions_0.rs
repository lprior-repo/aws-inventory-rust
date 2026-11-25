pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AcceptInvitation",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts an invitation for the member account to contribute data to a behavior graph.",
    ),
    (
        "BatchGetGraphMemberDatasources",
        "Requires GraphArn, AccountIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets data source package information for the behavior graph.",
    ),
    (
        "BatchGetMembershipDatasources",
        "Requires GraphArns; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information on the data source package history for an account.",
    ),
    (
        "CreateGraph",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Creates a new behavior graph for the calling account, and sets that account as the administrator account.",
    ),
    (
        "CreateMembers",
        "Requires GraphArn, Accounts; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: CreateMembers is used to send invitations to accounts.",
    ),
    (
        "DeleteGraph",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables the specified behavior graph and queues it to be deleted.",
    ),
    (
        "DeleteMembers",
        "Requires GraphArn, AccountIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified member accounts from the behavior graph.",
    ),
    (
        "DescribeOrganizationConfiguration",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the configuration for the organization behavior graph.",
    ),
    (
        "DisableOrganizationAdminAccount",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Removes the Detective administrator account in the current Region.",
    ),
    (
        "DisassociateMembership",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the member account from the specified behavior graph.",
    ),
    (
        "EnableOrganizationAdminAccount",
        "Requires AccountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Designates the Detective administrator account for the organization in the current Region.",
    ),
    (
        "GetInvestigation",
        "Requires GraphArn, InvestigationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Detective investigations lets you investigate IAM users and IAM roles using indicators of compromise.",
    ),
    (
        "GetMembers",
        "Requires GraphArn, AccountIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the membership details for specified member accounts for a behavior graph.",
    ),
    (
        "ListDatasourcePackages",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists data source packages in the behavior graph.",
    ),
    (
        "ListIndicators",
        "Requires GraphArn, InvestigationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the indicators from an investigation.",
    ),
    (
        "ListInvestigations",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Detective investigations lets you investigate IAM users and IAM roles using indicators of compromise.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the tag values that are assigned to a behavior graph.",
    ),
    (
        "RejectInvitation",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Rejects an invitation to contribute the account data to a behavior graph.",
    ),
    (
        "StartInvestigation",
        "Requires GraphArn, EntityArn, ScopeStartTime, ScopeEndTime; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Detective investigations lets you investigate IAM users and IAM roles using indicators of compromise.",
    ),
    (
        "StartMonitoringMember",
        "Requires GraphArn, AccountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Sends a request to enable data ingest for a member account that has a status of ACCEPTED_BUT_DISABLED .",
    ),
    (
        "TagResource",
        "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Applies tag values to a behavior graph.",
    ),
    (
        "UntagResource",
        "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from a behavior graph.",
    ),
    (
        "UpdateDatasourcePackages",
        "Requires GraphArn, DatasourcePackages; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts a data source package for the Detective behavior graph.",
    ),
    (
        "UpdateInvestigationState",
        "Requires GraphArn, InvestigationId, State; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the state of an investigation.",
    ),
    (
        "UpdateOrganizationConfiguration",
        "Requires GraphArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the configuration for the Organizations integration in the current Region.",
    ),
];
