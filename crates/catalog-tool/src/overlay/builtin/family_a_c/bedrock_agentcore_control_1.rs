// Pinned source: .local/aws-models/models/bedrock-agentcore-control/service/2023-06-05/bedrock-agentcore-control-2023-06-05.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists all workload identities in your account.
        (
            "ListWorkloadIdentities",
            OperationOverlay {
                queryable: vec![path("workloadIdentityArn")?],
                ..read(
                    "workloadIdentities",
                    Identity::Arn(path("workloadIdentityArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "PutResourcePolicy",
            deferred(
                "Model requires resourceArn, policy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a resource-based policy for a resource with the specified resourceArn. This feature is currently available only for AgentCore Runtime and Gateway.",
            ),
        ),
        (
            "ResetTokenVaultKeyMaterial",
            deferred(
                "Model requires tokenVaultId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resets the key material for a token vault whose customer managed KMS key has been deleted, and re-encrypts the token vault with an Amazon Web Services managed key so that you can use the token vault again. This operation can't be…",
            ),
        ),
        (
            "RotatePaymentConnectorCredentials",
            deferred(
                "Model requires paymentManagerId, paymentConnectorId, credentialsToRotate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Replaces the service-managed credentials of a payment connector with newly issued credentials. Use this operation only for payment connectors with a provisionMode of QUICK_CREATE . For payment connectors with a provisionMode of…",
            ),
        ),
        (
            "SetTokenVaultCMK",
            deferred(
                "Model requires kmsConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the customer master key (CMK) for a token vault.",
            ),
        ),
        (
            "StartPolicyGeneration",
            deferred(
                "Model requires policyEngineId, resource, content, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates the AI-powered generation of Dogwood policies from natural language descriptions within the AgentCore Policy system. This feature enables both technical and non-technical users to create policies by describing their…",
            ),
        ),
        (
            "SubmitRegistryRecordForApproval",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a registry record for approval. This transitions the record from DRAFT status to PENDING_APPROVAL status. If the registry has auto-approval enabled, the record is automatically approved.",
            ),
        ),
        (
            "SynchronizeGatewayTargets",
            deferred(
                "Model requires gatewayIdentifier, targetIdList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Synchronizes the gateway targets by fetching the latest tool definitions from the target endpoints. You cannot synchronize a target that is in a pending authorization state ( CREATE_PENDING_AUTH , UPDATE_PENDING_AUTH , or…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified tags to a resource with the specified resourceArn. If existing tags on a resource are not specified in the request parameters, they are not changed. When a resource is deleted, the tags associated with…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified resource. This feature is currently available only for AgentCore Runtime, Browser, Browser Profile, Code Interpreter tool, and Gateway.",
            ),
        ),
        (
            "UpdateAgentRuntime",
            deferred(
                "Model requires agentRuntimeId, agentRuntimeArtifact, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amazon Secure Agent.",
            ),
        ),
        (
            "UpdateAgentRuntimeEndpoint",
            deferred(
                "Model requires agentRuntimeId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amazon Bedrock AgentCore Runtime endpoint.",
            ),
        ),
        (
            "UpdateApiKeyCredentialProvider",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing API key credential provider.",
            ),
        ),
        (
            "UpdateCapacityProvider",
            deferred(
                "Model requires capacityProviderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a capacity provider. Only the description can be changed. To change other configuration, such as instance types, networking, or storage, create a new capacity provider.",
            ),
        ),
        (
            "UpdateConfigurationBundle",
            deferred(
                "Model requires bundleId, parentVersionIds, commitMessage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a configuration bundle by creating a new version with the specified changes. Each update creates a new version in the version history.",
            ),
        ),
        (
            "UpdateConsentPortal",
            deferred(
                "Model requires consentPortalIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing consent portal.",
            ),
        ),
        (
            "UpdateDataset",
            deferred(
                "Model requires datasetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a dataset's metadata. Synchronous operation. Only provided fields are updated; omitted fields remain unchanged. To modify dataset content, use AddDatasetExamples , UpdateDatasetExamples , or DeleteDatasetExamples .",
            ),
        ),
        (
            "UpdateDatasetExamples",
            deferred(
                "Model requires datasetId, examples. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates multiple existing examples in-place on DRAFT. All examples are validated against the dataset's schema type before any writes occur. If any example fails validation, the entire batch is rejected (all-or-nothing semantics).",
            ),
        ),
        (
            "UpdateEvaluator",
            deferred(
                "Model requires evaluatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom evaluator's configuration, description, or evaluation level. Built-in evaluators cannot be updated. The evaluator must not be locked for modification.",
            ),
        ),
        (
            "UpdateGateway",
            deferred(
                "Model requires gatewayIdentifier, name, roleArn, authorizerType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing gateway. Replaces the entire gateway configuration. Any optional field that you omit, including fields in protocolConfiguration , is reset. For information about retrieving the current configuration, see…",
            ),
        ),
        (
            "UpdateGatewayRateLimit",
            deferred(
                "Model requires gatewayIdentifier, rateLimitId, entries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the entries of a gateway rate limit. The dimension keys are immutable after creation.",
            ),
        ),
        (
            "UpdateGatewayRule",
            deferred(
                "Model requires gatewayIdentifier, ruleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a gateway rule's priority, conditions, actions, or description.",
            ),
        ),
        (
            "UpdateGatewayTarget",
            deferred(
                "Model requires gatewayIdentifier, targetId, targetConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing gateway target. You cannot update a target that is in a pending authorization state ( CREATE_PENDING_AUTH , UPDATE_PENDING_AUTH , or SYNCHRONIZE_PENDING_AUTH ). Wait for the authorization to complete or fail…",
            ),
        ),
        (
            "UpdateHarness",
            deferred(
                "Model requires harnessId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to update a harness.",
            ),
        ),
        (
            "UpdateHarnessEndpoint",
            deferred(
                "Model requires harnessId, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Operation to update a harness endpoint.",
            ),
        ),
        (
            "UpdateMemory",
            deferred(
                "Model requires memoryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update an Amazon Bedrock AgentCore Memory resource memory.",
            ),
        ),
        (
            "UpdateOauth2CredentialProvider",
            deferred(
                "Model requires name, credentialProviderVendor, oauth2ProviderConfigInput. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing OAuth2 credential provider.",
            ),
        ),
        (
            "UpdateOnlineEvaluationConfig",
            deferred(
                "Model requires onlineEvaluationConfigId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an online evaluation configuration's settings, including rules, data sources, evaluators, and execution status. Changes take effect immediately for ongoing evaluations.",
            ),
        ),
        (
            "UpdatePaymentConnector",
            deferred(
                "Model requires paymentManagerId, paymentConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing payment connector. This operation uses PATCH semantics, so you only need to specify the fields you want to change.",
            ),
        ),
        (
            "UpdatePaymentCredentialProvider",
            deferred(
                "Model requires name, credentialProviderVendor, providerConfigurationInput. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing payment credential provider with new authentication credentials.",
            ),
        ),
        (
            "UpdatePaymentManager",
            deferred(
                "Model requires paymentManagerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing payment manager. This operation uses PATCH semantics, so you only need to specify the fields you want to change.",
            ),
        ),
        (
            "UpdatePolicy",
            deferred(
                "Model requires policyEngineId, policyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing policy within the AgentCore Policy system. This operation allows modification of the policy description and definition while maintaining the policy's identity. The updated policy is validated against the Cedar…",
            ),
        ),
        (
            "UpdatePolicyEngine",
            deferred(
                "Model requires policyEngineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing policy engine within the AgentCore Policy system. This operation allows modification of the policy engine description while maintaining its identity. This is an asynchronous operation. Use the GetPolicyEngine…",
            ),
        ),
        (
            "UpdateRegistry",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing registry. This operation uses PATCH semantics, so you only need to specify the fields you want to change.",
            ),
        ),
        (
            "UpdateRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing registry record. This operation uses PATCH semantics, so you only need to specify the fields you want to change. The update is processed asynchronously and returns HTTP 202 Accepted.",
            ),
        ),
        (
            "UpdateRegistryRecordStatus",
            deferred(
                "Model requires registryId, recordId, status, statusReason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of a registry record. Use this operation to approve, reject, or deprecate a registry record.",
            ),
        ),
        (
            "UpdateWorkloadIdentity",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing workload identity.",
            ),
        ),
    ])
}
