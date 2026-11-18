// Pinned operation purposes: .local/aws-models/models/socialmessaging/service/2024-01-01/socialmessaging-2024-01-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: This is only used through the Amazon Web Services console during sign-up to associate your WhatsApp Business Account to your Amazon Web Services account.
    (
        "AssociateWhatsAppBusinessAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Meta Conversions API dataset for a WhatsApp Business Account.
    (
        "CreateWhatsAppDataset",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new WhatsApp Flow. Flows enable businesses to create rich, interactive forms and experiences that users can complete without leaving WhatsApp. The Flow is created in DRAFT status. If publish is set to true and a valid flowJson is provided, the Flow is published immediately.
    ("CreateWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new WhatsApp message template from a custom definition. Amazon Web Services End User Messaging Social does not store any WhatsApp message template content.
    (
        "CreateWhatsAppMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new WhatsApp message template using a template from Meta's template library.
    (
        "CreateWhatsAppMessageTemplateFromLibrary",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Uploads media for use in a WhatsApp message template.
    (
        "CreateWhatsAppMessageTemplateMedia",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a WhatsApp Flow permanently. Only Flows in DRAFT status can be deleted. Published or deprecated Flows cannot be deleted.
    ("DeleteWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a media object from the WhatsApp service. If the object is still in an Amazon S3 bucket you should delete it from there too.
    (
        "DeleteWhatsAppMessageMedia",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a WhatsApp message template.
    (
        "DeleteWhatsAppMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeprecateWhatsAppFlow",
        Decision::Deferred(
            "Required selectors id, flowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Deprecates a published WhatsApp Flow, marking it as no longer recommended for use. The Flow must be in PUBLISHED status. This is an irreversible operation.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociate a WhatsApp Business Account (WABA) from your Amazon Web Services account.
    (
        "DisassociateWhatsAppBusinessAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetLinkedWhatsAppBusinessAccount",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members account. Pinned purpose: Get the details of your linked WhatsApp Business Account.",
        ),
    ),
    (
        "GetLinkedWhatsAppBusinessAccountPhoneNumber",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members phoneNumber, linkedWhatsAppBusinessAccountId, callSettings. Pinned purpose: Retrieve the WABA account id and phone number details of a WhatsApp business account phone number.",
        ),
    ),
    (
        "GetWhatsAppBusinessPublicKey",
        Decision::Deferred(
            "Required selectors originationPhoneNumberId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members businessPublicKey, businessPublicKeySignatureStatus. Pinned purpose: Retrieves the business public key for a phone number and its signature status.",
        ),
    ),
    (
        "GetWhatsAppCallPermission",
        Decision::Deferred(
            "Required selectors originationPhoneNumberId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members permission, actions. Pinned purpose: Retrieves the current calling permission for a WhatsApp end user, along with the calling actions the business is allowed to take with that user. Provide the destination phone number or the business-scoped user ID to identify the end user.",
        ),
    ),
    (
        "GetWhatsAppFlow",
        Decision::Deferred(
            "Required selectors id, flowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowId, flowName, flowStatus, categories, validationErrors, jsonVersion, dataApiVersion, endpointUri, preview, whatsAppBusinessAccount, application, healthStatus. Pinned purpose: Retrieves the metadata and status of a WhatsApp Flow, including validation errors, preview information, and health status.",
        ),
    ),
    (
        "GetWhatsAppFlowPreview",
        Decision::Deferred(
            "Required selectors id, flowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowId, preview. Pinned purpose: Generates a web preview URL for testing a WhatsApp Flow before publishing. Preview URLs expire in 30 days and can be shared with stakeholders for review.",
        ),
    ),
    (
        "GetWhatsAppMessageMedia",
        Decision::Deferred(
            "Required selectors mediaId, originationPhoneNumberId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members mimeType, fileSize. Pinned purpose: Get a media file from the WhatsApp service. On successful completion the media file is retrieved from Meta and stored in the specified Amazon S3 bucket. Use either destinationS3File or destinationS3PresignedUrl for the destination. If both are used t",
        ),
    ),
    (
        "GetWhatsAppMessageTemplate",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members template. Pinned purpose: Retrieves a specific WhatsApp message template.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: List all tags associated with a resource, such as a phone number or WABA.",
        ),
    ),
    (
        "ListWhatsAppFlowAssets",
        Decision::Deferred(
            "Required selectors id, flowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowAssets, nextToken. Pinned purpose: Lists the assets (Flow JSON definition) of a WhatsApp Flow with presigned download URLs. Download URLs are generated by Meta and expire after a short period.",
        ),
    ),
    (
        "ListWhatsAppFlows",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flows, nextToken. Pinned purpose: Lists all WhatsApp Flows for a WhatsApp Business Account. Returns summary information including Flow ID, name, status, and categories.",
        ),
    ),
    (
        "ListWhatsAppMessageTemplates",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members templates, nextToken. Pinned purpose: Lists WhatsApp message templates for a specific WhatsApp Business Account.",
        ),
    ),
    (
        "ListWhatsAppTemplateLibrary",
        Decision::Deferred(
            "Required selectors id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members metaLibraryTemplates, nextToken. Pinned purpose: Lists templates available in Meta's template library for WhatsApp messaging.",
        ),
    ),
    (
        "PostWhatsAppMessageMedia",
        Decision::Deferred(
            "Required selectors originationPhoneNumberId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members mediaId. Pinned purpose: Upload a media file to the WhatsApp service. Only the specified originationPhoneNumberId has the permissions to send the media file when using SendWhatsAppMessage . You must use either sourceS3File or sourceS3PresignedUrl for the source. If both or n",
        ),
    ),
    (
        "PublishWhatsAppFlow",
        Decision::Deferred(
            "Required selectors id, flowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Publishes a WhatsApp Flow, making it available for use in template messages. The Flow must be in DRAFT status with valid Flow JSON that passes Meta's validation. This is an irreversible operation.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Add an event destination to log event data from WhatsApp for a WhatsApp Business Account (WABA). A WABA can only have one event destination at a time. All resources associated with the WABA use the same event destination.
    (
        "PutWhatsAppBusinessAccountEventDestinations",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sets the business public key used to encrypt the data exchanged with the endpoint of a data exchange Flow.
    (
        "PutWhatsAppBusinessPublicKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sends a WhatsApp calling event, such as connecting or terminating a call, for a business phone number. This operation passes the event through to Meta. To use this operation, the origination phone number must belong to a WhatsApp Business Account that is linked to your Amazon Web Services account.
    (
        "SendWhatsAppCallEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sends a conversion event to Meta's Conversions API for the specified WhatsApp Business Account dataset.
    (
        "SendWhatsAppConversionEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Send a WhatsApp message. For examples of sending a message using the Amazon Web Services CLI, see Sending messages in the Amazon Web Services End User Messaging Social User Guide .
    (
        "SendWhatsAppMessage",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or overwrites only the specified tags for the specified resource. When you specify an existing tag key, the value is overwritten with the new value.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the calling settings for a linked WhatsApp business phone number, such as whether calling is enabled and the hours during which the business accepts calls.
    (
        "UpdateLinkedWhatsAppBusinessAccountPhoneNumber",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the metadata of a WhatsApp Flow, such as its name or categories. This does not update the Flow JSON definition. Use UpdateWhatsAppFlowAssets to update the Flow JSON.
    ("UpdateWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the Flow JSON definition (assets) of a WhatsApp Flow. Updating a published Flow's assets reverts it to DRAFT status, requiring re-publishing.
    (
        "UpdateWhatsAppFlowAssets",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing WhatsApp message template.
    (
        "UpdateWhatsAppMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
];
