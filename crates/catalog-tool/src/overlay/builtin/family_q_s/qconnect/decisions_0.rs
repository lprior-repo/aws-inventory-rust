use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ActivateMessageTemplate",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, messageTemplateId, versionNumber are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateArn, messageTemplateId, versionNumber. Pinned purpose: Activates a specific version of the Amazon Q in Connect message template. After the version is activated, the previous active version will be deactivated automatically. You can use the $ACTIVE_VERSION qualifier later to reference the version that is ",
        ),
    ),
    ("CreateAIAgent", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAIAgentVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAIGuardrailVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAIPrompt", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAIPromptVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAssistant", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAssistantAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateContent", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateContentAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateMessageTemplateAttachment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateMessageTemplateVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateQuickResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSession", Decision::Denied(DenyReason::Mutating)),
    (
        "DeactivateMessageTemplate",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, messageTemplateId, versionNumber are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateArn, messageTemplateId, versionNumber. Pinned purpose: Deactivates a specific version of the Amazon Q in Connect message template . After the version is deactivated, you can no longer use the $ACTIVE_VERSION qualifier to reference the version in active status.",
        ),
    ),
    ("DeleteAIAgent", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAIAgentVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAIGuardrailVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAIPrompt", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAIPromptVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAssistant", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAssistantAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteContent", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteContentAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteImportJob", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteMessageTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteMessageTemplateAttachment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteQuickResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAIAgent",
        Decision::Deferred(
            "Required selectors assistantId, aiAgentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgent, versionNumber. Pinned purpose: Gets an Amazon Q in Connect AI Agent.",
        ),
    ),
    (
        "GetAIGuardrail",
        Decision::Deferred(
            "Required selectors assistantId, aiGuardrailId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrail, versionNumber. Pinned purpose: Gets the Amazon Q in Connect AI Guardrail.",
        ),
    ),
    (
        "GetAIPrompt",
        Decision::Deferred(
            "Required selectors assistantId, aiPromptId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiPrompt, versionNumber. Pinned purpose: Gets and Amazon Q in Connect AI Prompt.",
        ),
    ),
    (
        "GetAssistant",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assistant. Pinned purpose: Retrieves information about an assistant.",
        ),
    ),
    (
        "GetAssistantAssociation",
        Decision::Deferred(
            "Required selectors assistantAssociationId, assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assistantAssociation. Pinned purpose: Retrieves information about an assistant association.",
        ),
    ),
    ("GetContent", Decision::Denied(DenyReason::Content)),
    (
        "GetContentAssociation",
        Decision::Denied(DenyReason::Content),
    ),
    ("GetContentSummary", Decision::Denied(DenyReason::Content)),
    (
        "GetImportJob",
        Decision::Deferred(
            "Required selectors importJobId, knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members importJob. Pinned purpose: Retrieves the started import job.",
        ),
    ),
    (
        "GetKnowledgeBase",
        Decision::Deferred(
            "Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members knowledgeBase. Pinned purpose: Retrieves information about the knowledge base.",
        ),
    ),
    ("GetMessageTemplate", Decision::Denied(DenyReason::Content)),
    (
        "GetNextMessage",
        Decision::Deferred(
            "Required selectors assistantId, sessionId, nextMessageToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members type, response, requestMessageId, conversationState, nextMessageToken, conversationSessionData, chunkedResponseTerminated. Pinned purpose: Retrieves next message on an Amazon Q in Connect session.",
        ),
    ),
    (
        "GetQuickResponse",
        Decision::Deferred(
            "Required selectors quickResponseId, knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members quickResponse. Pinned purpose: Retrieves the quick response.",
        ),
    ),
    (
        "GetRecommendations",
        Decision::Deferred(
            "Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members recommendations, triggers. Pinned purpose: This API will be discontinued starting June 1, 2024. To receive generative responses after March 1, 2024, you will need to create a new Assistant in the Connect Customer console and integrate the Amazon Q in Connect JavaScript library (amazon-q-conne",
        ),
    ),
    (
        "GetSession",
        Decision::Deferred(
            "Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members session. Pinned purpose: Retrieves information for a specified session.",
        ),
    ),
    (
        "ListAIAgentVersions",
        Decision::Deferred(
            "Required selectors assistantId, aiAgentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgentVersionSummaries, nextToken. Pinned purpose: List AI Agent versions.",
        ),
    ),
    (
        "ListAIAgents",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgentSummaries, nextToken. Pinned purpose: Lists AI Agents.",
        ),
    ),
    (
        "ListAIGuardrailVersions",
        Decision::Deferred(
            "Required selectors assistantId, aiGuardrailId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrailVersionSummaries, nextToken. Pinned purpose: Lists AI Guardrail versions.",
        ),
    ),
    (
        "ListAIGuardrails",
        Decision::Deferred(
            "Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrailSummaries, nextToken. Pinned purpose: Lists the AI Guardrails available on the Amazon Q in Connect assistant.",
        ),
    ),
    (
        "ListContentAssociations",
        Decision::Deferred(
            "Required selectors knowledgeBaseId, contentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members contentAssociationSummaries, nextToken. Pinned purpose: Lists the content associations. For more information about content associations--what they are and when they are used--see Integrate Amazon Q in Connect with step-by-step guides in the Connect Customer Administrator Guide .",
        ),
    ),
];
