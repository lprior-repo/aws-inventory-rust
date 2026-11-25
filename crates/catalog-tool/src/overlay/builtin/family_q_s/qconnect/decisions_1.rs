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
    (
        "NotifyRecommendationsReceived",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutFeedback", Decision::Denied(DenyReason::Mutating)),
    (
        "QueryAssistant",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members results, nextToken. Pinned purpose: This API will be discontinued starting June 1, 2024. To receive generative responses after March 1, 2024, you will need to create a new Assistant in the Connect Customer console and integrate the Amazon Q in Connect JavaScript library (amazon-q-conne",
        ),
    ),
    (
        "RegisterAssistantReplicaKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterKnowledgeBaseReplicaKey",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RemoveAssistantAIAgent",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("SendMessage", Decision::Denied(DenyReason::Mutating)),
    ("StartContentUpload", Decision::Denied(DenyReason::Mutating)),
    ("StartImportJob", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAIAgent", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAIPrompt", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAssistantAIAgent",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateContent", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateKnowledgeBaseTemplateUri",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMessageTemplateMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateQuickResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSession", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSessionData", Decision::Denied(DenyReason::Mutating)),
];
