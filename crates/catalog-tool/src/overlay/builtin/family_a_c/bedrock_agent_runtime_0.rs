// Pinned source: .local/aws-models/models/bedrock-agent-runtime/service/2023-07-26/bedrock-agent-runtime-2023-07-26.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AgenticRetrieveStream",
            deferred(
                "Model requires messages, retrievers, agenticRetrieveConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information from one or more knowledge bases using an agentic approach. Agentic retrieval uses a foundation model to intelligently decompose complex queries into sub-queries and iteratively retrieve relevant information…",
            ),
        ),
        (
            "CheckIngestedDocumentAcl",
            deferred(
                "Model requires knowledgeBaseId, dataSourceId, documentId, userContext. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Checks whether a user has access to a specific document by verifying against the ingested access control list (ACL) in a knowledge base. Use this operation to validate that document-level access control is working as expected…",
            ),
        ),
        (
            "CreateInvocation",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new invocation within a session. An invocation groups the related invocation steps that store the content from a conversation. For more information about sessions, see Store and retrieve conversation history and context…",
            ),
        ),
        (
            "CreateSession",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is sessionId, sessionArn, sessionStatus, createdAt; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates a session to temporarily store conversations for generative AI (GenAI) applications built with open-source frameworks such as LangGraph and LlamaIndex. Sessions enable you to save the state of conversations at…",
            ),
        ),
        (
            "DeleteAgentMemory",
            deferred(
                "Model requires agentId, agentAliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes memory from the specified memory identifier.",
            ),
        ),
        (
            "DeleteSession",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a session that you ended. You can't delete a session with an ACTIVE status. To delete an active session, you must first end it with the EndSession API operation. For more information about sessions, see Store and retrieve…",
            ),
        ),
        (
            "EndSession",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ends the session. After you end a session, you can still access its content but you can’t add to it. To delete the session and it's content, you use the DeleteSession API operation. For more information about sessions, see Store…",
            ),
        ),
        (
            "GenerateQuery",
            deferred(
                "Model requires queryGenerationInput, transformationConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates an SQL query from a natural language query. For more information, see Generate a query for structured data in the Amazon Bedrock User Guide.",
            ),
        ),
        (
            "GetAgentMemory",
            deferred(
                "Model requires agentId, agentAliasId, memoryType, memoryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the sessions stored in the memory of the agent.",
            ),
        ),
        (
            "GetDocumentContent",
            deferred(
                "Model requires knowledgeBaseId, dataSourceId, documentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the content of an ingested document from a knowledge base, or of an image embedded in the document. Returns a pre-signed URL for downloading the content. Use this operation to display the source of a Retrieve result.",
            ),
        ),
        (
            "GetExecutionFlowSnapshot",
            deferred(
                "Model requires flowIdentifier, flowAliasIdentifier, executionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the flow definition snapshot used for a flow execution. The snapshot represents the flow metadata and definition as it existed at the time the execution was started. Note that even if the flow is edited after an…",
            ),
        ),
        (
            "GetFlowExecution",
            deferred(
                "Model requires flowIdentifier, flowAliasIdentifier, executionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific flow execution, including its status, start and end times, and any errors that occurred during execution.",
            ),
        ),
        (
            "GetIngestedDocumentAcl",
            deferred(
                "Model requires knowledgeBaseId, dataSourceId, documentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the ingested access control list (ACL) for a specific document in a knowledge base. Use this operation to inspect the allow and deny lists that were ingested for a document to troubleshoot access control issues. To use…",
            ),
        ),
        (
            "GetInvocationStep",
            deferred(
                "Model requires invocationIdentifier, invocationStepId, sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a specific invocation step within an invocation in a session. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
        (
            "GetSession",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific session. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
        (
            "InvokeAgent",
            deferred(
                "Model requires agentId, agentAliasId, sessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Amazon Bedrock Agents (now Amazon Bedrock Agents Classic) is no longer open to new customers. For capabilities similar to Bedrock Agents Classic, explore Amazon Bedrock AgentCore. Existing customers can continue to use the…",
            ),
        ),
        (
            "InvokeFlow",
            deferred(
                "Model requires flowIdentifier, flowAliasIdentifier, inputs. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Invokes an alias of a flow to run the inputs that you specify and return the output of each node as a stream. If there's an error, the error is returned. For more information, see Test a flow in Amazon Bedrock in the Amazon…",
            ),
        ),
        (
            "InvokeInlineAgent",
            deferred(
                "Model requires foundationModel, instruction, sessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Invokes an inline Amazon Bedrock agent using the configurations you provide with the request. Specify the following fields for security purposes. (Optional) customerEncryptionKeyArn – The Amazon Resource Name (ARN) of a KMS key…",
            ),
        ),
        // Model: Lists events that occurred during a flow execution. Events provide detailed information about the execution progress, including node inputs and outputs, flow inputs and outputs, condition results,…
        ("ListFlowExecutionEvents", denied(DenyReason::Content)),
        (
            "ListFlowExecutions",
            deferred(
                "Model requires flowIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all executions of a flow. Results can be paginated and include summary information about each execution, such as status, start and end times, and the execution's Amazon Resource Name (ARN). Flow executions is in preview…",
            ),
        ),
        (
            "ListInvocationSteps",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all invocation steps associated with a session and optionally, an invocation within the session. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
        (
            "ListInvocations",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all invocations associated with a specific session. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
        (
            "ListSessions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is sessionSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all sessions in your Amazon Web Services account. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List all the tags for the resource you specify.",
            ),
        ),
        (
            "OptimizePrompt",
            deferred(
                "Model requires input, targetModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Optimizes a prompt for the task that you specify. For more information, see Optimize a prompt in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "PutInvocationStep",
            deferred(
                "Model requires sessionIdentifier, invocationIdentifier, invocationStepTime, payload. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add an invocation step to an invocation in a session. An invocation step stores fine-grained state checkpoints, including text and images, for each interaction. For more information about sessions, see Store and retrieve…",
            ),
        ),
        (
            "Rerank",
            deferred(
                "Model requires queries, sources, rerankingConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Reranks the relevance of sources based on queries. For more information, see Improve the relevance of query responses with a reranker model .",
            ),
        ),
        (
            "Retrieve",
            deferred(
                "Model requires knowledgeBaseId, retrievalQuery. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Queries a knowledge base and retrieves information from it.",
            ),
        ),
        (
            "RetrieveAndGenerate",
            deferred(
                "Model requires input. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Queries a knowledge base and generates responses based on the retrieved results and using the specified foundation model or inference profile . The response only cites sources that are relevant to the query. This API cannot be…",
            ),
        ),
        (
            "RetrieveAndGenerateStream",
            deferred(
                "Model requires input. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Queries a knowledge base and generates responses based on the retrieved results, with output in streaming format. This API cannot be used with managed knowledge bases. Use AgenticRetrieveStream or Retrieve with managed knowledge…",
            ),
        ),
        (
            "StartFlowExecution",
            deferred(
                "Model requires flowIdentifier, flowAliasIdentifier, inputs. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an execution of an Amazon Bedrock flow. Unlike flows that run until completion or time out after five minutes, flow executions let you run flows asynchronously for longer durations. Flow executions also yield control so…",
            ),
        ),
        (
            "StopFlowExecution",
            deferred(
                "Model requires flowIdentifier, flowAliasIdentifier, executionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an Amazon Bedrock flow's execution. This operation prevents further processing of the flow and changes the execution status to Aborted .",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associate tags with a resource. For more information, see Tagging resources in the Amazon Bedrock User Guide.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove tags from a resource.",
            ),
        ),
        (
            "UpdateSession",
            deferred(
                "Model requires sessionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the metadata or encryption settings of a session. For more information about sessions, see Store and retrieve conversation history and context with Amazon Bedrock sessions .",
            ),
        ),
    ])
}
