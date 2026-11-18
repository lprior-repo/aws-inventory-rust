// Pinned operation purposes: .local/aws-models/models/qbusiness/service/2023-11-27/qbusiness-2023-11-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListMessages",
        Decision::Deferred(
            "Required selectors conversationId, applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messages, nextToken. Pinned purpose: Gets a list of messages associated with an Amazon Q Business web experience.",
        ),
    ),
    (
        "ListPluginActions",
        Decision::Deferred(
            "Required selectors applicationId, pluginId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items. Pinned purpose: Lists configured Amazon Q Business actions for a specific plugin in an Amazon Q Business application.",
        ),
    ),
    (
        "ListPluginTypeActions",
        Decision::Deferred(
            "Required selectors pluginType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items. Pinned purpose: Lists configured Amazon Q Business actions for any plugin type—both built-in and custom.",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists metadata for all Amazon Q Business plugin types.
    (
        "ListPluginTypeMetadata",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListSubscriptions",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, subscriptions. Pinned purpose: Lists all subscriptions created in an Amazon Q Business application.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceARN. Pinned purpose: Gets a list of tags associated with a specified resource. Amazon Q Business applications and data sources can have tags associated with them.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enables your end user to provide feedback on their Amazon Q Business generated chat responses.
    ("PutFeedback", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Create, or updates, a mapping of users—who have access to a document—to groups. You can also map sub groups to groups. For example, the group "Company Intellectual Property Teams" includes sub groups "Research" and "Engineering". These sub groups include their own list of users or people who work in these teams. Only users who work in research and engineering, and therefore belong in the intellect
    ("PutGroup", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Searches for relevant content in a Amazon Q Business application based on a query. This operation takes a search query text, the Amazon Q Business application identifier, and optional filters (such as content source and maximum results) as input. It returns a list of relevant content items, where each item includes the content text, the unique document identifier, the document title, the document
    (
        "SearchRelevantContent",
        Decision::Denied(DenyReason::Content),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a data source connector synchronization job. If a synchronization job is already in progress, Amazon Q Business returns a ConflictException .
    (
        "StartDataSourceSyncJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an Amazon Q Business data source connector synchronization job already in progress.
    (
        "StopDataSourceSyncJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds the specified tag to the specified Amazon Q Business application or data source resource. If the tag already exists, the existing value is replaced with the new value.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag from an Amazon Q Business application or a data source.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Amazon Q Business application. Amazon Q Business applications may securely transmit data for processing across Amazon Web Services Regions within your geography. For more information, see Cross region inference in Amazon Q Business . An Amazon Q Apps service-linked role will be created if it's absent in the Amazon Web Services account when QAppsConfiguration is enabled in the r
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a set of chat controls configured for an existing Amazon Q Business application.
    (
        "UpdateChatControlsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing chat response configuration in an Amazon Q Business application. This operation allows administrators to modify configuration settings, display name, and response parameters to refine how the system generates responses.
    (
        "UpdateChatResponseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing data accessor. This operation allows modifying the action configurations (the allowed actions and associated filters) and the display name of the data accessor. It does not allow changing the IAM role associated with the data accessor or other core properties of the data accessor.
    ("UpdateDataAccessor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Amazon Q Business data source connector.
    ("UpdateDataSource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Q Business index.
    ("UpdateIndex", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Q Business plugin.
    ("UpdatePlugin", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the retriever used for your Amazon Q Business application.
    ("UpdateRetriever", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the pricing tier for an Amazon Q Business subscription. Upgrades are instant. Downgrades apply at the start of the next month. Subscription tier determines feature access for the user. For more information on subscriptions and pricing tiers, see Amazon Q Business pricing .
    ("UpdateSubscription", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a information associated with a user id.
    ("UpdateUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Q Business web experience.
    (
        "UpdateWebExperience",
        Decision::Denied(DenyReason::Mutating),
    ),
];
