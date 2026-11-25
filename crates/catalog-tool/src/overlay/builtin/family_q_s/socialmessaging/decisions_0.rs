use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateWhatsAppBusinessAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateWhatsAppDataset",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateWhatsAppMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateWhatsAppMessageTemplateFromLibrary",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateWhatsAppMessageTemplateMedia",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteWhatsAppMessageMedia",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    (
        "PutWhatsAppBusinessAccountEventDestinations",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutWhatsAppBusinessPublicKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SendWhatsAppCallEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SendWhatsAppConversionEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SendWhatsAppMessage",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateLinkedWhatsAppBusinessAccountPhoneNumber",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateWhatsAppFlow", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateWhatsAppFlowAssets",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateWhatsAppMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
];
