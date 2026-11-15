// Pinned operation purposes: .local/aws-models/models/qconnect/service/2020-10-19/qconnect-2020-10-19.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("ActivateMessageTemplate", Decision::Deferred("Required selectors knowledgeBaseId, messageTemplateId, versionNumber are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateArn, messageTemplateId, versionNumber. Pinned purpose: Activates a specific version of the Amazon Q in Connect message template. After the version is activated, the previous active version will be deactivated automatically. You can use the $ACTIVE_VERSION qualifier later to reference the version that is ")),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect AI Agent.
    ("CreateAIAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates and Amazon Q in Connect AI Agent version.
    ("CreateAIAgentVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect AI Guardrail.
    ("CreateAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect AI Guardrail version.
    ("CreateAIGuardrailVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect AI Prompt.
    ("CreateAIPrompt", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect AI Prompt version.
    ("CreateAIPromptVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect assistant.
    ("CreateAssistant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an association between an Amazon Q in Connect assistant and another resource. Currently, the only supported association is with a knowledge base. An assistant can have only a single association.
    ("CreateAssistantAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates Amazon Q in Connect content. Before to calling this API, use StartContentUpload to upload an asset.
    ("CreateContent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an association between a content resource in a knowledge base and step-by-step guides . Step-by-step guides offer instructions to agents for resolving common customer issues. You create a content association to integrate Amazon Q in Connect and step-by-step guides. After you integrate Amazon Q and step-by-step guides, when Amazon Q provides a recommendation to an agent based on the intent 
    ("CreateContentAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a knowledge base. When using this API, you cannot reuse Amazon AppIntegrations DataIntegrations with external knowledge bases such as Salesforce and ServiceNow. If you do, you'll get an InvalidRequestException error. For example, you're programmatically managing your external knowledge base, and you want to add or remove one of the fields that is being ingested from Salesforce. Do the foll
    ("CreateKnowledgeBase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect message template. The name of the message template has to be unique for each knowledge base. The channel subtype of the message template is immutable and cannot be modified after creation. After the message template is created, you can use the $LATEST qualifier to reference the created message template.
    ("CreateMessageTemplate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Uploads an attachment file to the specified Amazon Q in Connect message template. The name of the message template attachment has to be unique for each message template referenced by the $LATEST qualifier. The body of the attachment file should be encoded using base64 encoding. After the file is uploaded, you can use the pre-signed Amazon S3 URL returned in response to download the uploaded file.
    ("CreateMessageTemplateAttachment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon Q in Connect message template version from the current content and configuration of a message template. Versions are immutable and monotonically increasing. Once a version is created, you can reference a specific version of the message template by passing in &lt;message-template-id&gt;:&lt;versionNumber&gt; as the message template identifier. An error is displayed if the suppl
    ("CreateMessageTemplateVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Q in Connect quick response.
    ("CreateQuickResponse", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a session. A session is a contextual container used for generating recommendations. Connect Customer creates a new Amazon Q in Connect session for each contact on which Amazon Q in Connect is enabled.
    ("CreateSession", Decision::Denied(DenyReason::Mutating)),
    ("DeactivateMessageTemplate", Decision::Deferred("Required selectors knowledgeBaseId, messageTemplateId, versionNumber are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members messageTemplateArn, messageTemplateId, versionNumber. Pinned purpose: Deactivates a specific version of the Amazon Q in Connect message template . After the version is deactivated, you can no longer use the $ACTIVE_VERSION qualifier to reference the version in active status.")),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q in Connect AI Agent.
    ("DeleteAIAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q in Connect AI Agent Version.
    ("DeleteAIAgentVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q in Connect AI Guardrail.
    ("DeleteAIGuardrail", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete and Amazon Q in Connect AI Guardrail version.
    ("DeleteAIGuardrailVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q in Connect AI Prompt.
    ("DeleteAIPrompt", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete and Amazon Q in Connect AI Prompt version.
    ("DeleteAIPromptVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an assistant.
    ("DeleteAssistant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an assistant association.
    ("DeleteAssistantAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the content.
    ("DeleteContent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the content association. For more information about content associations--what they are and when they are used--see Integrate Amazon Q in Connect with step-by-step guides in the Connect Customer Administrator Guide .
    ("DeleteContentAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the quick response import job.
    ("DeleteImportJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the knowledge base. When you use this API to delete an external knowledge base such as Salesforce or ServiceNow, you must also delete the Amazon AppIntegrations DataIntegration. This is because you can't reuse the DataIntegration after it's been associated with an external knowledge base. However, you can delete and recreate it. See DeleteDataIntegration and CreateDataIntegration in the Am
    ("DeleteKnowledgeBase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Q in Connect message template entirely or a specific version of the message template if version is supplied in the request. You can provide the message template identifier as &lt;message-template-id&gt;:&lt;versionNumber&gt; to delete a specific version of the message template. If it is not supplied, the message template and all available versions will be deleted.
    ("DeleteMessageTemplate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the attachment file from the Amazon Q in Connect message template that is referenced by $LATEST qualifier. Attachments on available message template versions will remain unchanged.
    ("DeleteMessageTemplateAttachment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a quick response.
    ("DeleteQuickResponse", Decision::Denied(DenyReason::Mutating)),
    ("GetAIAgent", Decision::Deferred("Required selectors assistantId, aiAgentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgent, versionNumber. Pinned purpose: Gets an Amazon Q in Connect AI Agent.")),
    ("GetAIGuardrail", Decision::Deferred("Required selectors assistantId, aiGuardrailId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrail, versionNumber. Pinned purpose: Gets the Amazon Q in Connect AI Guardrail.")),
    ("GetAIPrompt", Decision::Deferred("Required selectors assistantId, aiPromptId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiPrompt, versionNumber. Pinned purpose: Gets and Amazon Q in Connect AI Prompt.")),
    ("GetAssistant", Decision::Deferred("Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assistant. Pinned purpose: Retrieves information about an assistant.")),
    ("GetAssistantAssociation", Decision::Deferred("Required selectors assistantAssociationId, assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assistantAssociation. Pinned purpose: Retrieves information about an assistant association.")),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Retrieves content, including a pre-signed URL to download the content.
    ("GetContent", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the content association. For more information about content associations--what they are and when they are used--see Integrate Amazon Q in Connect with step-by-step guides in the Connect Customer Administrator Guide .
    ("GetContentAssociation", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Retrieves summary information about the content.
    ("GetContentSummary", Decision::Denied(DenyReason::Content)),
    ("GetImportJob", Decision::Deferred("Required selectors importJobId, knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members importJob. Pinned purpose: Retrieves the started import job.")),
    ("GetKnowledgeBase", Decision::Deferred("Required selectors knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members knowledgeBase. Pinned purpose: Retrieves information about the knowledge base.")),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Retrieves the Amazon Q in Connect message template. The message template identifier can contain an optional qualifier, for example, &lt;message-template-id&gt;:&lt;qualifier&gt; , which is either an actual version number or an Amazon Q Connect managed qualifier $ACTIVE_VERSION | $LATEST . If it is not supplied, then $LATEST is assumed implicitly.
    ("GetMessageTemplate", Decision::Denied(DenyReason::Content)),
    ("GetNextMessage", Decision::Deferred("Required selectors assistantId, sessionId, nextMessageToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members type, response, requestMessageId, conversationState, nextMessageToken, conversationSessionData, chunkedResponseTerminated. Pinned purpose: Retrieves next message on an Amazon Q in Connect session.")),
    ("GetQuickResponse", Decision::Deferred("Required selectors quickResponseId, knowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members quickResponse. Pinned purpose: Retrieves the quick response.")),
    ("GetRecommendations", Decision::Deferred("Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members recommendations, triggers. Pinned purpose: This API will be discontinued starting June 1, 2024. To receive generative responses after March 1, 2024, you will need to create a new Assistant in the Connect Customer console and integrate the Amazon Q in Connect JavaScript library (amazon-q-conne")),
    ("GetSession", Decision::Deferred("Required selectors assistantId, sessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members session. Pinned purpose: Retrieves information for a specified session.")),
    ("ListAIAgentVersions", Decision::Deferred("Required selectors assistantId, aiAgentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgentVersionSummaries, nextToken. Pinned purpose: List AI Agent versions.")),
    ("ListAIAgents", Decision::Deferred("Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiAgentSummaries, nextToken. Pinned purpose: Lists AI Agents.")),
    ("ListAIGuardrailVersions", Decision::Deferred("Required selectors assistantId, aiGuardrailId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrailVersionSummaries, nextToken. Pinned purpose: Lists AI Guardrail versions.")),
    ("ListAIGuardrails", Decision::Deferred("Required selectors assistantId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members aiGuardrailSummaries, nextToken. Pinned purpose: Lists the AI Guardrails available on the Amazon Q in Connect assistant.")),
    ("ListContentAssociations", Decision::Deferred("Required selectors knowledgeBaseId, contentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members contentAssociationSummaries, nextToken. Pinned purpose: Lists the content associations. For more information about content associations--what they are and when they are used--see Integrate Amazon Q in Connect with step-by-step guides in the Connect Customer Administrator Guide .")),
];
