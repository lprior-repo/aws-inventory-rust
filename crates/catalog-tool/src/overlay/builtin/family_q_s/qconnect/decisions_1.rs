// Pinned operation purposes: .local/aws-models/models/qconnect/service/2020-10-19/qconnect-2020-10-19.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListContents",
        Decision::Deferred(
            "Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members contentSummaries, nextToken. Pinned purpose: Lists the content.",
        ),
    ),
    (
        "ListImportJobs",
        Decision::Deferred(
            "Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members importJobSummaries, nextToken. Pinned purpose: Lists information about import jobs.",
        ),
    ),
    (
        "ListMessageTemplateVersions",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, messageTemplateId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateVersionSummaries, nextToken. Pinned purpose: Lists all the available versions for the specified Amazon Q in Connect message template.",
        ),
    ),
    (
        "ListMessageTemplates",
        Decision::Deferred(
            "Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateSummaries, nextToken. Pinned purpose: Lists all the available Amazon Q in Connect message templates for the specified knowledge base.",
        ),
    ),
    (
        "ListMessages",
        Decision::Deferred(
            "Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messages, nextToken. Pinned purpose: Lists messages on an Amazon Q in Connect session.",
        ),
    ),
    (
        "ListModels",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members modelSummaries, nextToken. Pinned purpose: Lists the models available to an Amazon Q in Connect assistant in the assistant's Amazon Web Services Region. The available models are determined by the region of the specified assistant.",
        ),
    ),
    (
        "ListQuickResponses",
        Decision::Deferred(
            "Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members quickResponseSummaries, nextToken. Pinned purpose: Lists information about quick response.",
        ),
    ),
    (
        "ListSpans",
        Decision::Deferred(
            "Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spans, nextToken. Pinned purpose: Retrieves AI agent execution traces for a session, providing granular visibility into agent orchestration flows, LLM interactions, and tool invocations.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags for the specified resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified recommendations from the specified assistant's queue of newly available recommendations. You can use this API in conjunction with GetRecommendations and a waitTimeSeconds input for long-polling behavior and avoiding duplicate recommendations.
    (
        "NotifyRecommendationsReceived",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Provides feedback against the specified assistant for the specified target. This API only supports generative targets.
    ("PutFeedback", Decision::Denied(DenyReason::Mutating)),
    (
        "QueryAssistant",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members results, nextToken. Pinned purpose: This API will be discontinued starting June 1, 2024. To receive generative responses after March 1, 2024, you will need to create a new Assistant in the Connect Customer console and integrate the Amazon Q in Connect JavaScript library (amazon-q-conne",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a replica Region KMS key for an Amazon Q in Connect assistant. To use this operation, you must have kms:GenerateDataKey permission on the replica key.
    (
        "RegisterAssistantReplicaKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a replica Region KMS key for an Amazon Q in Connect knowledge base. To use this operation, you must have kms:GenerateDataKey permission on the replica key. You must also have kms:CreateGrant permission on the primary key.
    (
        "RegisterKnowledgeBaseReplicaKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the AI Agent that is set for use by default on an Amazon Q in Connect Assistant.
    (
        "RemoveAssistantAIAgent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a URI template from a knowledge base.
    (
        "RemoveKnowledgeBaseTemplateUri",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RenderMessageTemplate",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, messageTemplateId, attributes are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members content, sourceConfigurationSummary, attributesNotInterpolated, attachments. Pinned purpose: Renders the Amazon Q in Connect message template based on the attribute values provided and generates the message content. For any variable present in the message template, if the attribute value is neither provided in the attribute request parameter",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Retrieves content from knowledge sources based on a query.
    ("Retrieve", Decision::Denied(DenyReason::Content)),
    (
        "SearchContent",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, searchExpression are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members contentSummaries, nextToken. Pinned purpose: Searches for content in a specified knowledge base. Can be used to get a specific content resource by its name.",
        ),
    ),
    (
        "SearchMessageTemplates",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, searchExpression are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members results, nextToken. Pinned purpose: Searches for Amazon Q in Connect message templates in the specified knowledge base.",
        ),
    ),
    (
        "SearchQuickResponses",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, searchExpression are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members results, nextToken. Pinned purpose: Searches existing Amazon Q in Connect quick responses in an Amazon Q in Connect knowledge base.",
        ),
    ),
    (
        "SearchSessions",
        Decision::Deferred(
            "Required selectors assistantId, searchExpression are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sessionSummaries, nextToken. Pinned purpose: Searches for sessions.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Submits a message to the Amazon Q in Connect session.
    ("SendMessage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Get a URL to upload content to a knowledge base. To upload content, first make a PUT request to the returned URL with your file, making sure to include the required headers. Then use CreateContent to finalize the content creation process or UpdateContent to modify an existing resource. You can only upload content to a knowledge base of type CUSTOM.
    ("StartContentUpload", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Start an asynchronous job to import Amazon Q in Connect resources from an uploaded source file. Before calling this API, use StartContentUpload to upload an asset that contains the resource data. For importing Amazon Q in Connect quick responses, you need to upload a csv file including the quick responses. For information about how to format the csv file for importing quick responses, see Import q
    ("StartImportJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds the specified tags to the specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an AI Agent.
    ("UpdateAIAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an AI Guardrail.
    ("UpdateAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an AI Prompt.
    ("UpdateAIPrompt", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the AI Agent that is set for use by default on an Amazon Q in Connect Assistant.
    (
        "UpdateAssistantAIAgent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates information about the content.
    ("UpdateContent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the template URI of a knowledge base. This is only supported for knowledge bases of type EXTERNAL. Include a single variable in ${variable} format; this interpolated by Amazon Q in Connect using ingested content. For example, if you ingest a Salesforce article, it has an Id value, and you can set the template URI to https://myInstanceName.lightning.force.com/lightning/r/Knowledge__kav/*${I
    (
        "UpdateKnowledgeBaseTemplateUri",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the Amazon Q in Connect message template. Partial update is supported. If any field is not supplied, it will remain unchanged for the message template that is referenced by the $LATEST qualifier. Any modification will only apply to the message template that is referenced by the $LATEST qualifier. The fields for all available versions will remain unchanged.
    (
        "UpdateMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the Amazon Q in Connect message template metadata. Note that any modification to the message template’s name, description and grouping configuration will applied to the message template pointed by the $LATEST qualifier and all available versions. Partial update is supported. If any field is not supplied, it will remain unchanged for the message template.
    (
        "UpdateMessageTemplateMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Amazon Q in Connect quick response.
    (
        "UpdateQuickResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a session. A session is a contextual container used for generating recommendations. Connect Customer updates the existing Amazon Q in Connect session for each contact on which Amazon Q in Connect is enabled.
    ("UpdateSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the data stored on an Amazon Q in Connect Session.
    ("UpdateSessionData", Decision::Denied(DenyReason::Mutating)),
];
