use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateLibraryItemReview",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateQAppWithUser",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchCreateCategory",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteCategory",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateCategory",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateLibraryItem", Decision::Denied(DenyReason::Mutating)),
    ("CreatePresignedUrl", Decision::Denied(DenyReason::Mutating)),
    ("CreateQApp", Decision::Denied(DenyReason::Mutating)),
    ("DeleteLibraryItem", Decision::Denied(DenyReason::Mutating)),
    ("DeleteQApp", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeQAppPermissions",
        Decision::Deferred(
            "Required selectors instanceId, appId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourceArn, appId, permissions. Pinned purpose: Describes read permissions for a Amazon Q App in Amazon Q Business application environment instance.",
        ),
    ),
    (
        "DisassociateLibraryItemReview",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateQAppFromUser",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExportQAppSessionData",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetLibraryItem",
        Decision::Deferred(
            "Required selectors instanceId, libraryItemId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members libraryItemId, appId, appVersion, categories, status, createdAt, createdBy, updatedAt, updatedBy, ratingCount, isRatedByUser, userCount, isVerified. Pinned purpose: Retrieves details about a library item for an Amazon Q App, including its metadata, categories, ratings, and usage statistics.",
        ),
    ),
    (
        "GetQApp",
        Decision::Deferred(
            "Required selectors instanceId, appId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appId, appArn, title, description, initialPrompt, appVersion, status, createdAt, createdBy, updatedAt, updatedBy, requiredCapabilities, appDefinition. Pinned purpose: Retrieves the full details of an Q App, including its definition specifying the cards and flow.",
        ),
    ),
    (
        "GetQAppSession",
        Decision::Deferred(
            "Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionName, appVersion, latestPublishedAppVersion, status, cardStatus, userIsHost. Pinned purpose: Retrieves the current state and results for an active session of an Amazon Q App.",
        ),
    ),
    (
        "GetQAppSessionMetadata",
        Decision::Deferred(
            "Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionName, sharingConfiguration, sessionOwner. Pinned purpose: Retrieves the current configuration of a Q App session.",
        ),
    ),
    ("ImportDocument", Decision::Denied(DenyReason::Mutating)),
    (
        "ListCategories",
        Decision::Deferred(
            "Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members categories. Pinned purpose: Lists the categories of a Amazon Q Business application environment instance. For more information, see Custom labels for Amazon Q Apps .",
        ),
    ),
    (
        "ListLibraryItems",
        Decision::Deferred(
            "Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members libraryItems, nextToken. Pinned purpose: Lists the library items for Amazon Q Apps that are published and available for users in your Amazon Web Services account.",
        ),
    ),
    (
        "ListQAppSessionData",
        Decision::Deferred(
            "Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionData, nextToken. Pinned purpose: Lists the collected data of a Q App data collection session.",
        ),
    ),
    (
        "ListQApps",
        Decision::Deferred(
            "Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members apps, nextToken. Pinned purpose: Lists the Amazon Q Apps owned by or associated with the user either because they created it or because they used it from the library in the past. The user identity is extracted from the credentials used to invoke this operation..",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceARN. Pinned purpose: Lists the tags associated with an Amazon Q Apps resource.",
        ),
    ),
    (
        "PredictQApp",
        Decision::Deferred(
            "Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members app, problemStatement. Pinned purpose: Generates an Amazon Q App definition based on either a conversation or a problem statement provided as input.The resulting app definition can be used to call CreateQApp . This API doesn't create Amazon Q Apps directly.",
        ),
    ),
    ("StartQAppSession", Decision::Denied(DenyReason::Mutating)),
    ("StopQAppSession", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateLibraryItem", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateLibraryItemMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateQApp", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateQAppPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateQAppSession", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateQAppSessionMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
];
