// Pinned source: .local/aws-models/models/bedrock-agentcore-control/service/2023-06-05/bedrock-agentcore-control-2023-06-05.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddDatasetExamples",
            deferred(
                "Model requires datasetId, source. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds examples to the dataset's DRAFT. All examples are validated against the dataset's schema type before any writes occur. If any example fails validation, the entire batch is rejected (all-or-nothing semantics).",
            ),
        ),
        (
            "BatchPutGatewayRateLimits",
            deferred(
                "Model requires gatewayIdentifier, rateLimits. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Atomically creates or updates multiple rate limits for a gateway. The operation updates existing limits with matching keys and creates new limits for new keys. If the operation fails, the service applies no changes. Retry the…",
            ),
        ),
        (
            "CreateAgentRuntime",
            deferred(
                "Model requires agentRuntimeName, agentRuntimeArtifact, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Bedrock AgentCore Runtime.",
            ),
        ),
        (
            "CreateAgentRuntimeEndpoint",
            deferred(
                "Model requires agentRuntimeId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AgentCore Runtime endpoint.",
            ),
        ),
        (
            "CreateApiKeyCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new API key credential provider.",
            ),
        ),
        (
            "CreateBrowser",
            deferred(
                "Model requires name, networkConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom browser.",
            ),
        ),
        (
            "CreateBrowserProfile",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a browser profile in Amazon Bedrock AgentCore. A browser profile stores persistent browser data such as cookies, local storage, session storage, and browsing history that can be saved from browser sessions and reused in…",
            ),
        ),
        (
            "CreateCapacityProvider",
            deferred(
                "Model requires name, permissionsConfiguration, computeConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a capacity provider. A capacity provider defines the Amazon EC2 infrastructure for AgentCore Runtime, including the operating system, allowed instance types, networking, and storage. It also specifies the IAM permissions…",
            ),
        ),
        (
            "CreateCodeInterpreter",
            deferred(
                "Model requires name, networkConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom code interpreter.",
            ),
        ),
        (
            "CreateConfigurationBundle",
            deferred(
                "Model requires bundleName, components. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new configuration bundle resource. A configuration bundle stores versioned component configurations for agent evaluation workflows.",
            ),
        ),
        (
            "CreateConsentPortal",
            deferred(
                "Model requires executionRoleArn, idpConfig, name, sources. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new consent portal.",
            ),
        ),
        (
            "CreateDataset",
            deferred(
                "Model requires datasetName, source, schemaType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new dataset resource asynchronously. Returns immediately with status CREATING. Poll GetDataset until status transitions to ACTIVE or CREATE_FAILED.",
            ),
        ),
        (
            "CreateDatasetVersion",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Publishes the current DRAFT as a new numbered version. The DRAFT is preserved and remains editable after publishing. Returns immediately with status UPDATING. Poll GetDataset until status transitions to ACTIVE or UPDATE_FAILED.",
            ),
        ),
        (
            "CreateEvaluator",
            deferred(
                "Model requires evaluatorName, evaluatorConfig, level. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom evaluator for agent quality assessment. Custom evaluators can use either LLM-as-a-Judge configurations with user-defined prompts, rating scales, and model settings, or code-based configurations with…",
            ),
        ),
        (
            "CreateGateway",
            deferred(
                "Model requires name, roleArn, authorizerType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a gateway for Amazon Bedrock Agent. A gateway serves as an integration point between your agent and external services. If you specify CUSTOM_JWT as the authorizerType , you must provide an authorizerConfiguration .",
            ),
        ),
        (
            "CreateGatewayRateLimit",
            deferred(
                "Model requires gatewayIdentifier, dimensionKeys, entries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a rate limit for a gateway. Rate limits define throttling rules for each dimension that control request rates, token consumption rates, and concurrent connections through the gateway.",
            ),
        ),
        (
            "CreateGatewayRule",
            deferred(
                "Model requires gatewayIdentifier, priority, actions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a rule for a gateway. Rules define conditions and actions that control how requests are routed and processed through the gateway, including principal-based access control and path-based routing.",
            ),
        ),
        (
            "CreateGatewayTarget",
            deferred(
                "Model requires gatewayIdentifier, targetConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a target for a gateway. A target defines an endpoint that the gateway can connect to.",
            ),
        ),
        (
            "CreateHarness",
            deferred(
                "Model requires harnessName, executionRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to create a harness.",
            ),
        ),
        (
            "CreateHarnessEndpoint",
            deferred(
                "Model requires harnessId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to create a harness endpoint.",
            ),
        ),
        (
            "CreateMemory",
            deferred(
                "Model requires name, eventExpiryDuration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Amazon Bedrock AgentCore Memory resource.",
            ),
        ),
        (
            "CreateOauth2CredentialProvider",
            deferred(
                "Model requires name, credentialProviderVendor, oauth2ProviderConfigInput. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new OAuth2 credential provider.",
            ),
        ),
        (
            "CreateOnlineEvaluationConfig",
            deferred(
                "Model requires onlineEvaluationConfigName, rule, dataSourceConfig, evaluationExecutionRoleArn, enableOnCreate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an online evaluation configuration for continuous monitoring of agent performance. Online evaluation automatically samples live traffic from CloudWatch logs at specified rates and applies evaluators to assess agent…",
            ),
        ),
        (
            "CreatePaymentConnector",
            deferred(
                "Model requires paymentManagerId, name, type, credentialProviderConfigurations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new payment connector for a payment manager. A payment connector integrates with a supported payment provider to enable payment processing capabilities.",
            ),
        ),
        (
            "CreatePaymentCredentialProvider",
            deferred(
                "Model requires name, credentialProviderVendor, providerConfigurationInput. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new payment credential provider for storing authentication credentials used by payment connectors to communicate with external payment providers.",
            ),
        ),
        (
            "CreatePaymentManager",
            deferred(
                "Model requires name, authorizerType, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new payment manager in your Amazon Web Services account. A payment manager serves as the top-level resource for managing payment processing capabilities, including payment connectors that integrate with supported…",
            ),
        ),
        (
            "CreatePolicy",
            deferred(
                "Model requires name, definition, policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a policy within the AgentCore Policy system. Policies provide real-time, deterministic control over agentic interactions with AgentCore Gateway. Using Cedar or Dogwood, you can define fine-grained policies that specify…",
            ),
        ),
        (
            "CreatePolicyEngine",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new policy engine within the AgentCore Policy system. A policy engine is a collection of policies that evaluates and authorizes agent tool calls. When associated with Gateways (each Gateway can be associated with at…",
            ),
        ),
        (
            "CreateRegistry",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new registry in your Amazon Web Services account. A registry serves as a centralized catalog for organizing and managing registry records, including MCP servers, A2A agents, agent skills, and custom resource types. If…",
            ),
        ),
        (
            "CreateRegistryRecord",
            deferred(
                "Model requires registryId, name, descriptorType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new registry record within the specified registry. A registry record represents an individual AI resource's metadata in the registry. This could be an MCP server (and associated tools), A2A agent, agent skill, or a…",
            ),
        ),
        (
            "CreateWorkloadIdentity",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new workload identity.",
            ),
        ),
        (
            "DeleteAgentRuntime",
            deferred(
                "Model requires agentRuntimeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Bedrock AgentCore Runtime, or a single version of an AgentCore Runtime when you provide the version qualifier.",
            ),
        ),
        (
            "DeleteAgentRuntimeEndpoint",
            deferred(
                "Model requires agentRuntimeId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an AgentCore Runtime endpoint.",
            ),
        ),
        (
            "DeleteApiKeyCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an API key credential provider.",
            ),
        ),
        (
            "DeleteBrowser",
            deferred(
                "Model requires browserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom browser.",
            ),
        ),
        (
            "DeleteBrowserProfile",
            deferred(
                "Model requires profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a browser profile.",
            ),
        ),
        (
            "DeleteCapacityProvider",
            deferred(
                "Model requires capacityProviderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a capacity provider. Before you delete a capacity provider, disassociate all agent runtimes and runtime versions that reference it. If any references remain, the operation fails.",
            ),
        ),
        (
            "DeleteCodeInterpreter",
            deferred(
                "Model requires codeInterpreterId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom code interpreter.",
            ),
        ),
        (
            "DeleteConfigurationBundle",
            deferred(
                "Model requires bundleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configuration bundle and all of its versions.",
            ),
        ),
        (
            "DeleteConsentPortal",
            deferred(
                "Model requires consentPortalIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a consent portal.",
            ),
        ),
        (
            "DeleteDataset",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a dataset version or an entire dataset asynchronously. If datasetVersion is absent, deletes all versions and the dataset record itself. If provided, deletes only that specific version.",
            ),
        ),
        (
            "DeleteDatasetExamples",
            deferred(
                "Model requires datasetId, exampleIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes specific examples by ID from DRAFT. All example IDs are validated before any deletes occur. If any ID does not exist in DRAFT, the entire batch is rejected (all-or-nothing semantics).",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteEvaluator",
            deferred(
                "Model requires evaluatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom evaluator. Builtin evaluators cannot be deleted. The evaluator must not be referenced by any active online evaluation configurations.",
            ),
        ),
        (
            "DeleteGateway",
            deferred(
                "Model requires gatewayIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a gateway.",
            ),
        ),
        (
            "DeleteGatewayRateLimit",
            deferred(
                "Model requires gatewayIdentifier, rateLimitId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a gateway rate limit.",
            ),
        ),
        (
            "DeleteGatewayRule",
            deferred(
                "Model requires gatewayIdentifier, ruleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a gateway rule.",
            ),
        ),
        (
            "DeleteGatewayTarget",
            deferred(
                "Model requires gatewayIdentifier, targetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a gateway target. You cannot delete a target that is in a pending authorization state ( CREATE_PENDING_AUTH , UPDATE_PENDING_AUTH , or SYNCHRONIZE_PENDING_AUTH ). Wait for the authorization to complete or fail before…",
            ),
        ),
        (
            "DeleteHarness",
            deferred(
                "Model requires harnessId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to delete a Harness.",
            ),
        ),
        (
            "DeleteHarnessEndpoint",
            deferred(
                "Model requires harnessId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to delete a harness endpoint.",
            ),
        ),
        (
            "DeleteMemory",
            deferred(
                "Model requires memoryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Bedrock AgentCore Memory resource. When you delete a memory resource, it is permanently removed.",
            ),
        ),
        (
            "DeleteOauth2CredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an OAuth2 credential provider.",
            ),
        ),
        (
            "DeleteOnlineEvaluationConfig",
            deferred(
                "Model requires onlineEvaluationConfigId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an online evaluation configuration and stops any ongoing evaluation processes associated with it.",
            ),
        ),
        (
            "DeletePaymentConnector",
            deferred(
                "Model requires paymentManagerId, paymentConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a payment connector.",
            ),
        ),
        (
            "DeletePaymentCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a payment credential provider and its associated stored credentials.",
            ),
        ),
        (
            "DeletePaymentManager",
            deferred(
                "Model requires paymentManagerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a payment manager. All payment connectors associated with the payment manager must be deleted before the payment manager can be deleted. This operation initiates the deletion process asynchronously.",
            ),
        ),
        (
            "DeletePolicy",
            deferred(
                "Model requires policyEngineId, policyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing policy from the AgentCore Policy system. Once deleted, the policy can no longer be used for agent behavior control and all references to it become invalid. This is an asynchronous operation. Use the GetPolicy…",
            ),
        ),
        (
            "DeletePolicyEngine",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing policy engine from the AgentCore Policy system. The policy engine must not have any associated policies before deletion. Once deleted, the policy engine and all its configurations become unavailable for policy…",
            ),
        ),
        (
            "DeleteRegistry",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a registry. The registry must contain zero records before it can be deleted. This operation initiates the deletion process asynchronously.",
            ),
        ),
        (
            "DeleteRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a registry record. The record's status transitions to DELETING and the record is removed asynchronously.",
            ),
        ),
        (
            "DeleteResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the resource-based policy for a specified resource. This feature is currently available only for AgentCore Runtime and Gateway.",
            ),
        ),
        (
            "DeleteWorkloadIdentity",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a workload identity.",
            ),
        ),
        (
            "GetAgentRuntime",
            deferred(
                "Model requires agentRuntimeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an Amazon Bedrock AgentCore Runtime.",
            ),
        ),
        (
            "GetAgentRuntimeEndpoint",
            deferred(
                "Model requires agentRuntimeId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an Amazon Secure AgentEndpoint.",
            ),
        ),
        (
            "GetApiKeyCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an API key credential provider.",
            ),
        ),
        // Model: Gets information about a custom browser.
        (
            "GetBrowser",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("browserId")?,
                    Binding::From {
                        operation: name("ListBrowsers")?,
                        path: path("browserId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("browserArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("browserArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetBrowserProfile",
            deferred(
                "Model requires profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a browser profile.",
            ),
        ),
        (
            "GetCapacityProvider",
            deferred(
                "Model requires capacityProviderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a capacity provider, including its status, permissions configuration, and compute configuration.",
            ),
        ),
        // Model: Gets information about a custom code interpreter.
        (
            "GetCodeInterpreter",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("codeInterpreterId")?,
                    Binding::From {
                        operation: name("ListCodeInterpreters")?,
                        path: path("codeInterpreterId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("codeInterpreterArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("codeInterpreterArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetConfigurationBundle",
            deferred(
                "Model requires bundleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the latest version of a configuration bundle. By default, returns the latest version on the mainline branch. Use GetConfigurationBundleVersion to retrieve a specific historical version.",
            ),
        ),
        (
            "GetConfigurationBundleVersion",
            deferred(
                "Model requires bundleId, versionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a specific version of a configuration bundle by its version identifier.",
            ),
        ),
        (
            "GetConsentPortal",
            deferred(
                "Model requires consentPortalIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a consent portal.",
            ),
        ),
        (
            "GetDataset",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves dataset metadata. Use the datasetVersion query parameter to retrieve a specific version's metadata. If absent, defaults to DRAFT. For paginated example content, use ListDatasetExamples .",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetEvaluator",
            deferred(
                "Model requires evaluatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about an evaluator, including its configuration, status, and metadata. Works with both built-in and custom evaluators.",
            ),
        ),
        (
            "GetGateway",
            deferred(
                "Model requires gatewayIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific Gateway.",
            ),
        ),
        (
            "GetGatewayRateLimit",
            deferred(
                "Model requires gatewayIdentifier, rateLimitId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a gateway rate limit.",
            ),
        ),
        (
            "GetGatewayRule",
            deferred(
                "Model requires gatewayIdentifier, ruleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific gateway rule.",
            ),
        ),
        (
            "GetGatewayTarget",
            deferred(
                "Model requires gatewayIdentifier, targetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific gateway target.",
            ),
        ),
        (
            "GetHarness",
            deferred(
                "Model requires harnessId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to get a single harness.",
            ),
        ),
        (
            "GetHarnessEndpoint",
            deferred(
                "Model requires harnessId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to get a single harness endpoint.",
            ),
        ),
        // Model: Retrieve an existing Amazon Bedrock AgentCore Memory resource.
        ("GetMemory", denied(DenyReason::Content)),
        (
            "GetOauth2CredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an OAuth2 credential provider.",
            ),
        ),
        (
            "GetOnlineEvaluationConfig",
            deferred(
                "Model requires onlineEvaluationConfigId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about an online evaluation configuration, including its rules, data sources, evaluators, and execution status.",
            ),
        ),
        (
            "GetPaymentConnector",
            deferred(
                "Model requires paymentManagerId, paymentConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific payment connector.",
            ),
        ),
        (
            "GetPaymentCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific payment credential provider.",
            ),
        ),
        (
            "GetPaymentManager",
            deferred(
                "Model requires paymentManagerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific payment manager.",
            ),
        ),
        (
            "GetPolicy",
            deferred(
                "Model requires policyEngineId, policyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific policy within the AgentCore Policy system. This operation returns the complete policy definition, metadata, and current status, allowing administrators to review and manage policy…",
            ),
        ),
        (
            "GetPolicyEngine",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific policy engine within the AgentCore Policy system. This operation returns the complete policy engine configuration, metadata, and current status, allowing administrators to review…",
            ),
        ),
        (
            "GetPolicyEngineSummary",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a metadata-only summary of a specific policy engine without decrypting customer content. This lightweight read operation returns resource identifiers, status, timestamps, and the encryption key ARN, but does not include…",
            ),
        ),
        (
            "GetPolicyGeneration",
            deferred(
                "Model requires policyGenerationId, policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a policy generation request within the AgentCore Policy system. Policy generation converts natural language descriptions into Dogwood policy statements using AI-powered translation, enabling…",
            ),
        ),
        (
            "GetPolicyGenerationSummary",
            deferred(
                "Model requires policyGenerationId, policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a metadata-only summary of a specific policy generation request without decrypting customer content. This lightweight read operation returns resource identifiers, status, timestamps, and findings, but does not include…",
            ),
        ),
        (
            "GetPolicySummary",
            deferred(
                "Model requires policyEngineId, policyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a metadata-only summary of a specific policy without decrypting customer content. This lightweight read operation returns resource identifiers, status, and timestamps, but does not include the policy definition,…",
            ),
        ),
        (
            "GetRegistry",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific registry.",
            ),
        ),
        (
            "GetRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific registry record.",
            ),
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the resource-based policy for a specified resource. This feature is currently available only for AgentCore Runtime and Gateway.",
            ),
        ),
        (
            "GetTokenVault",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is kmsConfiguration, lastModifiedDate; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves information about a token vault.",
            ),
        ),
        (
            "GetWorkloadIdentity",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a workload identity.",
            ),
        ),
        (
            "ListAgentRuntimeEndpoints",
            deferred(
                "Model requires agentRuntimeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all endpoints for a specific Amazon Secure Agent.",
            ),
        ),
        (
            "ListAgentRuntimeVersions",
            deferred(
                "Model requires agentRuntimeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all versions of a specific Amazon Secure Agent.",
            ),
        ),
        (
            "ListAgentRuntimeVersionsByCapacityProvider",
            deferred(
                "Model requires capacityProviderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the agent runtime versions that are associated with a capacity provider. Use this operation to identify the runtimes you must disassociate before you can delete the capacity provider. Results are paginated; use the…",
            ),
        ),
        // Model: Lists all Amazon Secure Agents in your account.
        (
            "ListAgentRuntimes",
            OperationOverlay {
                queryable: vec![path("agentRuntimeArn")?],
                ..read(
                    "agentRuntimes",
                    Identity::Arn(path("agentRuntimeArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListApiKeyCredentialProviders",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is credentialProviders; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all API key credential providers in your account.",
            ),
        ),
        // Model: Lists all browser profiles in your account.
        (
            "ListBrowserProfiles",
            OperationOverlay {
                queryable: vec![path("profileArn")?],
                ..read(
                    "profileSummaries",
                    Identity::Arn(path("profileArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists all custom browsers in your account.
        (
            "ListBrowsers",
            OperationOverlay {
                queryable: vec![path("browserArn")?],
                fixed: [(path("type")?, FixedValue::String("CUSTOM".to_owned()))].into_iter().collect(),
                ..read(
                    "browserSummaries",
                    Identity::Arn(path("browserArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists the capacity providers in your account and returns summary information for each one. To retrieve the full configuration for a specific capacity provider, use GetCapacityProvider . Results are…
        (
            "ListCapacityProviders",
            OperationOverlay {
                queryable: vec![path("capacityProviderArn")?],
                ..read(
                    "capacityProviders",
                    Identity::Arn(path("capacityProviderArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all custom code interpreters in your account.
        (
            "ListCodeInterpreters",
            OperationOverlay {
                queryable: vec![path("codeInterpreterArn")?],
                fixed: [(path("type")?, FixedValue::String("CUSTOM".to_owned()))].into_iter().collect(),
                ..read(
                    "codeInterpreterSummaries",
                    Identity::Arn(path("codeInterpreterArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConfigurationBundleVersions",
            deferred(
                "Model requires bundleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all versions of a configuration bundle, with optional filtering by branch name or creation source.",
            ),
        ),
        // Model: Lists all configuration bundles in the account.
        (
            "ListConfigurationBundles",
            OperationOverlay {
                queryable: vec![path("bundleArn")?],
                ..read(
                    "bundles",
                    Identity::Arn(path("bundleArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all of the consent portals in your account.
        (
            "ListConsentPortals",
            OperationOverlay {
                queryable: vec![path("consentPortalArn")?],
                ..read(
                    "consentPortals",
                    Identity::Arn(path("consentPortalArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListDatasetExamples",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns paginated examples from the dataset. The server embeds the resolved version in the pagination token. Once pagination begins, all subsequent pages are pinned to that version regardless of concurrent mutations.",
            ),
        ),
        (
            "ListDatasetVersions",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all published versions of a dataset, sorted by version number descending (newest first). Does not include the DRAFT working copy.",
            ),
        ),
        // Model: Lists all datasets in the caller's account, paginated.
        (
            "ListDatasets",
            OperationOverlay {
                queryable: vec![path("datasetArn")?],
                ..read(
                    "datasets",
                    Identity::Arn(path("datasetArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all available evaluators, including both builtin evaluators provided by the service and custom evaluators created by the user.
        (
            "ListEvaluators",
            deferred(
                "The modeled collection mixes builtin, third-party and custom evaluators without an ownership request filter; no owned-only selector is available.",
            ),
        ),
        (
            "ListGatewayRateLimits",
            deferred(
                "Model requires gatewayIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all rate limits for a gateway. Results are paginated. Use the nextToken parameter to retrieve additional results.",
            ),
        ),
        (
            "ListGatewayRules",
            deferred(
                "Model requires gatewayIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all rules for a gateway.",
            ),
        ),
        (
            "ListGatewayTargets",
            deferred(
                "Model requires gatewayIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all targets for a specific gateway.",
            ),
        ),
    ])
}

pub(super) fn part_4() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists all gateways in the account.
        (
            "ListGateways",
            OperationOverlay {
                queryable: vec![path("gatewayId")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("gatewayId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListHarnessEndpoints",
            deferred(
                "Model requires harnessId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to list the endpoints of a harness.",
            ),
        ),
        (
            "ListHarnessVersions",
            deferred(
                "Model requires harnessId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to list the versions of a Harness.",
            ),
        ),
        // Model: Operation to list harnesses.
        (
            "ListHarnesses",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "harnesses",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists the available Amazon Bedrock AgentCore Memory resources in the current Amazon Web Services Region.
        (
            "ListMemories",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "memories",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListOauth2CredentialProviders",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is credentialProviders; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all OAuth2 credential providers in your account.",
            ),
        ),
        (
            "ListOnlineEvaluationConfigs",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is onlineEvaluationConfigs; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all online evaluation configurations in the account, providing summary information about each configuration's status and settings.",
            ),
        ),
        (
            "ListPaymentConnectors",
            deferred(
                "Model requires paymentManagerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all payment connectors for a specified payment manager.",
            ),
        ),
        (
            "ListPaymentCredentialProviders",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is credentialProviders; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all payment credential providers in the account.",
            ),
        ),
        (
            "ListPaymentManagers",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is paymentManagers; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all payment managers in the account.",
            ),
        ),
        (
            "ListPolicies",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of policies within the AgentCore Policy engine. This operation supports pagination and filtering to help administrators manage and discover policies across policy engines. Results can be filtered by policy engine…",
            ),
        ),
        // Model: Retrieves a paginated list of metadata-only policy engine summaries without decrypting customer content. This lightweight read operation returns resource identifiers, status, and timestamps for each…
        (
            "ListPolicyEngineSummaries",
            OperationOverlay {
                queryable: vec![path("policyEngineArn")?],
                ..read(
                    "policyEngines",
                    Identity::Arn(path("policyEngineArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPolicyEngines",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is policyEngines; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves a list of policy engines within the AgentCore Policy system. This operation supports pagination to help administrators discover and manage policy engines across their account. Each policy engine serves as a container…",
            ),
        ),
        (
            "ListPolicyGenerationAssets",
            deferred(
                "Model requires policyGenerationId, policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of generated policy assets from a policy generation request within the AgentCore Policy system. This operation returns the actual Dogwood policies and related artifacts produced by the AI-powered policy…",
            ),
        ),
        (
            "ListPolicyGenerationSummaries",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of metadata-only policy generation summaries within a policy engine without decrypting customer content. This lightweight read operation returns resource identifiers, status, timestamps, and findings…",
            ),
        ),
        (
            "ListPolicyGenerations",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of policy generation requests within the AgentCore Policy system. This operation supports pagination and filtering to help track and manage AI-powered policy generation operations.",
            ),
        ),
        (
            "ListPolicySummaries",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of metadata-only policy summaries within a policy engine without decrypting customer content. This lightweight read operation returns resource identifiers, status, and timestamps for each policy, but…",
            ),
        ),
        // Model: Lists all registries in the account. You can optionally filter results by status using the status parameter, or by authorizer type using the authorizerType parameter.
        (
            "ListRegistries",
            OperationOverlay {
                queryable: vec![path("registryArn")?],
                ..read(
                    "registries",
                    Identity::Arn(path("registryArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListRegistryRecords",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists registry records within a registry. You can optionally filter results using the name , status , and descriptorType parameters. When multiple filters are specified, they are combined using AND logic.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags associated with the specified resource. This feature is currently available only for AgentCore Runtime, Browser, Browser Profile, Code Interpreter tool, and Gateway.",
            ),
        ),
    ])
}
