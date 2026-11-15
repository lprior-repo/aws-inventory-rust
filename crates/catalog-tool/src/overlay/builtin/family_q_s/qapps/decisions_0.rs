// Pinned operation purposes: .local/aws-models/models/qapps/service/2023-11-27/qapps-2023-11-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associates a rating or review for a library item with the user submitting the request. This increments the rating count for the specified library item.
    ("AssociateLibraryItemReview", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation creates a link between the user's identity calling the operation and a specific Q App. This is useful to mark the Q App as a favorite for the user if the user doesn't own the Amazon Q App so they can still run it and see it in their inventory of Q Apps.
    ("AssociateQAppWithUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates Categories for the Amazon Q Business application environment instance. Web experience users use Categories to tag and filter library items. For more information, see Custom labels for Amazon Q Apps .
    ("BatchCreateCategory", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes Categories for the Amazon Q Business application environment instance. Web experience users use Categories to tag and filter library items. For more information, see Custom labels for Amazon Q Apps .
    ("BatchDeleteCategory", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates Categories for the Amazon Q Business application environment instance. Web experience users use Categories to tag and filter library items. For more information, see Custom labels for Amazon Q Apps .
    ("BatchUpdateCategory", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new library item for an Amazon Q App, allowing it to be discovered and used by other allowed users.
    ("CreateLibraryItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a presigned URL for an S3 POST operation to upload a file. You can use this URL to set a default file for a FileUploadCard in a Q App definition or to provide a file for a single Q App run. The scope parameter determines how the file will be used, either at the app definition level or the app session level. The IAM permissions are derived from the qapps:ImportDocument action. For more info
    ("CreatePresignedUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon Q App based on the provided definition. The Q App definition specifies the cards and flow of the Q App. This operation also calculates the dependencies between the cards by inspecting the references in the prompts.
    ("CreateQApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a library item for an Amazon Q App, removing it from the library so it can no longer be discovered or used by other users.
    ("DeleteLibraryItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q App owned by the user. If the Q App was previously published to the library, it is also removed from the library.
    ("DeleteQApp", Decision::Denied(DenyReason::Mutating)),
    ("DescribeQAppPermissions", Decision::Deferred("Required selectors instanceId, appId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourceArn, appId, permissions. Pinned purpose: Describes read permissions for a Amazon Q App in Amazon Q Business application environment instance.")),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a rating or review previously submitted by the user for a library item.
    ("DisassociateLibraryItemReview", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a Q App from a user removing the user's access to run the Q App.
    ("DisassociateQAppFromUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Exports the collected data of a Q App data collection session.
    ("ExportQAppSessionData", Decision::Denied(DenyReason::Mutating)),
    ("GetLibraryItem", Decision::Deferred("Required selectors instanceId, libraryItemId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members libraryItemId, appId, appVersion, categories, status, createdAt, createdBy, updatedAt, updatedBy, ratingCount, isRatedByUser, userCount, isVerified. Pinned purpose: Retrieves details about a library item for an Amazon Q App, including its metadata, categories, ratings, and usage statistics.")),
    ("GetQApp", Decision::Deferred("Required selectors instanceId, appId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appId, appArn, title, description, initialPrompt, appVersion, status, createdAt, createdBy, updatedAt, updatedBy, requiredCapabilities, appDefinition. Pinned purpose: Retrieves the full details of an Q App, including its definition specifying the cards and flow.")),
    ("GetQAppSession", Decision::Deferred("Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionName, appVersion, latestPublishedAppVersion, status, cardStatus, userIsHost. Pinned purpose: Retrieves the current state and results for an active session of an Amazon Q App.")),
    ("GetQAppSessionMetadata", Decision::Deferred("Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionName, sharingConfiguration, sessionOwner. Pinned purpose: Retrieves the current configuration of a Q App session.")),
    // Changes service state rather than reading inventory. Pinned purpose: Uploads a file that can then be used either as a default in a FileUploadCard from Q App definition or as a file that is used inside a single Q App run. The purpose of the document is determined by a scope parameter that indicates whether it is at the app definition level or at the app session level.
    ("ImportDocument", Decision::Denied(DenyReason::Mutating)),
    ("ListCategories", Decision::Deferred("Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members categories. Pinned purpose: Lists the categories of a Amazon Q Business application environment instance. For more information, see Custom labels for Amazon Q Apps .")),
    ("ListLibraryItems", Decision::Deferred("Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members libraryItems, nextToken. Pinned purpose: Lists the library items for Amazon Q Apps that are published and available for users in your Amazon Web Services account.")),
    ("ListQAppSessionData", Decision::Deferred("Required selectors instanceId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionId, sessionArn, sessionData, nextToken. Pinned purpose: Lists the collected data of a Q App data collection session.")),
    ("ListQApps", Decision::Deferred("Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members apps, nextToken. Pinned purpose: Lists the Amazon Q Apps owned by or associated with the user either because they created it or because they used it from the library in the past. The user identity is extracted from the credentials used to invoke this operation..")),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceARN. Pinned purpose: Lists the tags associated with an Amazon Q Apps resource.")),
    ("PredictQApp", Decision::Deferred("Required selectors instanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members app, problemStatement. Pinned purpose: Generates an Amazon Q App definition based on either a conversation or a problem statement provided as input.The resulting app definition can be used to call CreateQApp . This API doesn't create Amazon Q Apps directly.")),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a new session for an Amazon Q App, allowing inputs to be provided and the app to be run. Each Q App session will be condensed into a single conversation in the web experience.
    ("StartQAppSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an active session for an Amazon Q App.This deletes all data related to the session and makes it invalid for future uses. The results of the session will be persisted as part of the conversation.
    ("StopQAppSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates tags with an Amazon Q Apps resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates tags from an Amazon Q Apps resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the library item for an Amazon Q App.
    ("UpdateLibraryItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the verification status of a library item for an Amazon Q App.
    ("UpdateLibraryItemMetadata", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Amazon Q App, allowing modifications to its title, description, and definition.
    ("UpdateQApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates read permissions for a Amazon Q App in Amazon Q Business application environment instance.
    ("UpdateQAppPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the session for a given Q App sessionId . This is only valid when at least one card of the session is in the WAITING state. Data for each WAITING card can be provided as input. If inputs are not provided, the call will be accepted but session will not move forward. Inputs for cards that are not in the WAITING status will be ignored.
    ("UpdateQAppSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration metadata of a session for a given Q App sessionId .
    ("UpdateQAppSessionMetadata", Decision::Denied(DenyReason::Mutating)),
];
