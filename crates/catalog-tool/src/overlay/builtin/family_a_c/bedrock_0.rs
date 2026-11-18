// Pinned source: .local/aws-models/models/bedrock/service/2023-04-20/bedrock-2023-04-20.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchDeleteAdvancedPromptOptimizationJob",
            deferred(
                "Model requires jobIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes one or more advanced prompt optimization jobs.",
            ),
        ),
        (
            "BatchDeleteEvaluationJob",
            deferred(
                "Model requires jobIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a batch of evaluation jobs. An evaluation job can only be deleted if it has following status FAILED , COMPLETED , and STOPPED . You can request up to 25 model evaluation jobs be deleted in a single request.",
            ),
        ),
        (
            "CancelAutomatedReasoningPolicyBuildWorkflow",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels a running Automated Reasoning policy build workflow. This stops the policy generation process and prevents further processing of the source documents.",
            ),
        ),
        (
            "CreateAdvancedPromptOptimizationJob",
            deferred(
                "Model requires jobName, inputConfig, outputConfig, modelConfigurations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an advanced prompt optimization job. The job optimizes your prompt templates for specific models using your evaluation dataset and criteria.",
            ),
        ),
        (
            "CreateAutomatedReasoningPolicy",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Automated Reasoning policy for Amazon Bedrock Guardrails. Automated Reasoning policies use mathematical techniques to detect hallucinations, suggest corrections, and highlight unstated assumptions in the responses of…",
            ),
        ),
        (
            "CreateAutomatedReasoningPolicyTestCase",
            deferred(
                "Model requires policyArn, guardContent, expectedAggregatedFindingsResult. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a test for an Automated Reasoning policy. Tests validate that your policy works as expected by providing sample inputs and expected outcomes. Use tests to verify policy behavior before deploying to production.",
            ),
        ),
        (
            "CreateAutomatedReasoningPolicyVersion",
            deferred(
                "Model requires policyArn, lastUpdatedDefinitionHash. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new version of an existing Automated Reasoning policy. This allows you to iterate on your policy rules while maintaining previous versions for rollback or comparison purposes.",
            ),
        ),
        (
            "CreateCustomModel",
            deferred(
                "Model requires modelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new custom model in Amazon Bedrock. After the model is active, you can use it for inference. You can provide the model data source in one of the following ways: customModelDataSource — Specify a SageMaker AI model…",
            ),
        ),
        (
            "CreateCustomModelDeployment",
            deferred(
                "Model requires modelDeploymentName, modelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deploys a custom model for on-demand inference in Amazon Bedrock. After you deploy your custom model, you use the deployment's Amazon Resource Name (ARN) as the modelId parameter when you submit prompts and generate responses…",
            ),
        ),
        (
            "CreateEvaluationJob",
            deferred(
                "Model requires jobName, roleArn, evaluationConfig, inferenceConfig, outputDataConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an evaluation job.",
            ),
        ),
        (
            "CreateFoundationModelAgreement",
            deferred(
                "Model requires offerToken, modelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Request a model access agreement for the specified model.",
            ),
        ),
        (
            "CreateGuardrail",
            deferred(
                "Model requires name, blockedInputMessaging, blockedOutputsMessaging. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a guardrail to block topics and to implement safeguards for your generative AI applications. You can configure the following policies in a guardrail to avoid undesirable and harmful content, filter out denied topics and…",
            ),
        ),
        (
            "CreateGuardrailVersion",
            deferred(
                "Model requires guardrailIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a version of the guardrail. Use this API to create a snapshot of the guardrail when you are satisfied with a configuration, or to compare the configuration with another version.",
            ),
        ),
        (
            "CreateInferenceProfile",
            deferred(
                "Model requires inferenceProfileName, modelSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an application inference profile to track metrics and costs when invoking a model. To create an application inference profile for a foundation model in one region, specify the ARN of the model in that region. To create an…",
            ),
        ),
        (
            "CreateMarketplaceModelEndpoint",
            deferred(
                "Model requires modelSourceIdentifier, endpointConfig, endpointName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an endpoint for a model from Amazon Bedrock Marketplace. The endpoint is hosted by Amazon SageMaker.",
            ),
        ),
        (
            "CreateModelCopyJob",
            deferred(
                "Model requires sourceModelArn, targetModelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Copies a model to another region so that it can be used there. For more information, see Copy models to be used in other regions in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "CreateModelCustomizationJob",
            deferred(
                "Model requires jobName, customModelName, roleArn, baseModelIdentifier, trainingDataConfig, outputDataConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a fine-tuning job to customize a base model. You specify the base foundation model and the location of the training data. After the model-customization job completes successfully, your custom model resource will be ready…",
            ),
        ),
        (
            "CreateModelImportJob",
            deferred(
                "Model requires jobName, importedModelName, roleArn, modelDataSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a model import job to import model that you have customized in other environments, such as Amazon SageMaker. For more information, see Import a customized model",
            ),
        ),
        (
            "CreateModelInvocationJob",
            deferred(
                "Model requires jobName, roleArn, modelId, inputDataConfig, outputDataConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a batch inference job to invoke a model on multiple prompts. Format your data according to Format your inference data and upload it to an Amazon S3 bucket. For more information, see Process multiple prompts with batch…",
            ),
        ),
        (
            "CreatePromptRouter",
            deferred(
                "Model requires promptRouterName, models, routingCriteria, fallbackModel. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a prompt router that manages the routing of requests between multiple foundation models based on the routing criteria.",
            ),
        ),
        (
            "CreateProvisionedModelThroughput",
            deferred(
                "Model requires modelUnits, provisionedModelName, modelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates dedicated throughput for a base or custom model with the model units and for the duration that you specify. For pricing details, see Amazon Bedrock Pricing . For more information, see Provisioned Throughput in the Amazon…",
            ),
        ),
        (
            "DeleteAutomatedReasoningPolicy",
            deferred(
                "Model requires policyArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Automated Reasoning policy or policy version. This operation is idempotent. If you delete a policy more than once, each call succeeds. Deleting a policy removes it permanently and cannot be undone.",
            ),
        ),
        (
            "DeleteAutomatedReasoningPolicyBuildWorkflow",
            deferred(
                "Model requires policyArn, buildWorkflowId, lastUpdatedAt. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Automated Reasoning policy build workflow and its associated artifacts. This permanently removes the workflow history and any generated assets.",
            ),
        ),
        (
            "DeleteAutomatedReasoningPolicyTestCase",
            deferred(
                "Model requires policyArn, testCaseId, lastUpdatedAt. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Automated Reasoning policy test. This operation is idempotent; if you delete a test more than once, each call succeeds.",
            ),
        ),
        (
            "DeleteCustomModel",
            deferred(
                "Model requires modelIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom model that you created earlier. For more information, see Custom models in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "DeleteCustomModelDeployment",
            deferred(
                "Model requires customModelDeploymentIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom model deployment. This operation stops the deployment and removes it from your account. After deletion, the deployment ARN can no longer be used for inference requests. The following actions are related to the…",
            ),
        ),
        (
            "DeleteEnforcedGuardrailConfiguration",
            deferred(
                "Model requires configId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the account-level enforced guardrail configuration.",
            ),
        ),
        (
            "DeleteFoundationModelAgreement",
            deferred(
                "Model requires modelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete the model access agreement for the specified model.",
            ),
        ),
        (
            "DeleteGuardrail",
            deferred(
                "Model requires guardrailIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a guardrail. To delete a guardrail, only specify the ARN of the guardrail in the guardrailIdentifier field. If you delete a guardrail, all of its versions will be deleted. To delete a version of a guardrail, specify the…",
            ),
        ),
        (
            "DeleteImportedModel",
            deferred(
                "Model requires modelIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom model that you imported earlier. For more information, see Import a customized model in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "DeleteInferenceProfile",
            deferred(
                "Model requires inferenceProfileIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an application inference profile. For more information, see Increase throughput and resilience with cross-region inference in Amazon Bedrock . in the Amazon Bedrock User Guide.",
            ),
        ),
        (
            "DeleteMarketplaceModelEndpoint",
            deferred(
                "Model requires endpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an endpoint for a model from Amazon Bedrock Marketplace.",
            ),
        ),
        (
            "DeleteModelInvocationLoggingConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Delete the invocation logging.",
            ),
        ),
        (
            "DeletePromptRouter",
            deferred(
                "Model requires promptRouterArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified prompt router. This action cannot be undone.",
            ),
        ),
        (
            "DeleteProvisionedModelThroughput",
            deferred(
                "Model requires provisionedModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Provisioned Throughput. You can't delete a Provisioned Throughput before the commitment term is over. For more information, see Provisioned Throughput in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "DeleteResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a previously created Bedrock resource policy.",
            ),
        ),
        (
            "DeregisterMarketplaceModelEndpoint",
            deferred(
                "Model requires endpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deregisters an endpoint for a model from Amazon Bedrock Marketplace. This operation removes the endpoint's association with Amazon Bedrock but does not delete the underlying Amazon SageMaker endpoint.",
            ),
        ),
        (
            "ExportAutomatedReasoningPolicyVersion",
            deferred(
                "Model requires policyArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports the policy definition for an Automated Reasoning policy version. Returns the complete policy definition including rules, variables, and custom variable types in a structured format.",
            ),
        ),
        (
            "GetAccountDataRetention",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is mode, updatedAt; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the account-wide data retention mode for Amazon Bedrock.",
            ),
        ),
        (
            "GetAdvancedPromptOptimizationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an advanced prompt optimization job.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicy",
            deferred(
                "Model requires policyArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about an Automated Reasoning policy or policy version. Returns information including the policy definition, metadata, and timestamps.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicyAnnotations",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current annotations for an Automated Reasoning policy build workflow. Annotations contain corrections to the rules, variables and types to be applied to the policy.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetAutomatedReasoningPolicyBuildWorkflow",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about an Automated Reasoning policy build workflow, including its status, configuration, and metadata.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicyBuildWorkflowResultAssets",
            deferred(
                "Model requires policyArn, buildWorkflowId, assetType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the resulting assets from a completed Automated Reasoning policy build workflow, including build logs, quality reports, and generated policy artifacts.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicyNextScenario",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the next test scenario for validating an Automated Reasoning policy. This is used during the interactive policy refinement process to test policy behavior.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicyTestCase",
            deferred(
                "Model requires policyArn, testCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific Automated Reasoning policy test.",
            ),
        ),
        (
            "GetAutomatedReasoningPolicyTestResult",
            deferred(
                "Model requires policyArn, buildWorkflowId, testCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the test result for a specific Automated Reasoning policy test. Returns detailed validation findings and execution status.",
            ),
        ),
        // Model: Get the properties associated with a Amazon Bedrock custom model that you have created. For more information, see Custom models in the Amazon Bedrock User Guide .
        (
            "GetCustomModel",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("modelIdentifier")?,
                    Binding::From {
                        operation: name("ListCustomModels")?,
                        path: path("modelArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("modelArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("modelArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetCustomModelDeployment",
            deferred(
                "Model requires customModelDeploymentIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a custom model deployment, including its status, configuration, and metadata. Use this operation to monitor the deployment status and retrieve details needed for inference requests. The following…",
            ),
        ),
        (
            "GetEvaluationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an evaluation job, such as the status of the job.",
            ),
        ),
        (
            "GetFoundationModel",
            deferred(
                "Model requires modelIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get details about a Amazon Bedrock foundation model.",
            ),
        ),
        (
            "GetFoundationModelAvailability",
            deferred(
                "Model requires modelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get information about the Foundation model availability.",
            ),
        ),
        (
            "GetGuardrail",
            deferred(
                "Model requires guardrailIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details about a guardrail. If you don't specify a version, the response returns details for the DRAFT version.",
            ),
        ),
        // Model: Gets properties associated with a customized model you imported.
        (
            "GetImportedModel",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("modelIdentifier")?,
                    Binding::From {
                        operation: name("ListImportedModels")?,
                        path: path("modelArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("modelArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("modelArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetInferenceProfile",
            deferred(
                "Model requires inferenceProfileIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an inference profile. For more information, see Increase throughput and resilience with cross-region inference in Amazon Bedrock . in the Amazon Bedrock User Guide.",
            ),
        ),
        (
            "GetMarketplaceModelEndpoint",
            deferred(
                "Model requires endpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific endpoint for a model from Amazon Bedrock Marketplace.",
            ),
        ),
        (
            "GetModelCopyJob",
            deferred(
                "Model requires jobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a model copy job. For more information, see Copy models to be used in other regions in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "GetModelCustomizationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the properties associated with a model-customization job, including the status of the job. For more information, see Custom models in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "GetModelImportJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the properties associated with import model job, including the status of the job. For more information, see Import a customized model in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "GetModelInvocationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details about a batch inference job. For more information, see Monitor batch inference jobs",
            ),
        ),
        (
            "GetModelInvocationLoggingConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is loggingConfig; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Get the current configuration values for model invocation logging.",
            ),
        ),
        (
            "GetPromptRouter",
            deferred(
                "Model requires promptRouterArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a prompt router.",
            ),
        ),
        // Model: Returns details for a Provisioned Throughput. For more information, see Provisioned Throughput in the Amazon Bedrock User Guide .
        (
            "GetProvisionedModelThroughput",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("provisionedModelId")?,
                    Binding::From {
                        operation: name("ListProvisionedModelThroughputs")?,
                        path: path("provisionedModelArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("provisionedModelArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("provisionedModelArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the resource policy document for a Bedrock resource",
            ),
        ),
        (
            "GetUseCaseForModelAccess",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is formData; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Get usecase for model access.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists the advanced prompt optimization jobs in your account.
        (
            "ListAdvancedPromptOptimizationJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "jobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all Automated Reasoning policies in your account, with optional filtering by policy ARN. This helps you manage and discover existing policies.
        (
            "ListAutomatedReasoningPolicies",
            OperationOverlay {
                queryable: vec![path("policyArn")?],
                ..read(
                    "automatedReasoningPolicySummaries",
                    Identity::Arn(path("policyArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListAutomatedReasoningPolicyBuildWorkflows",
            deferred(
                "Model requires policyArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all build workflows for an Automated Reasoning policy, showing the history of policy creation and modification attempts.",
            ),
        ),
        (
            "ListAutomatedReasoningPolicyTestCases",
            deferred(
                "Model requires policyArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists tests for an Automated Reasoning policy. We recommend using pagination to ensure that the operation returns quickly and successfully.",
            ),
        ),
        (
            "ListAutomatedReasoningPolicyTestResults",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists test results for an Automated Reasoning policy, showing how the policy performed against various test scenarios and validation checks.",
            ),
        ),
        // Model: Lists custom model deployments in your account. You can filter the results by creation time, name, status, and associated model. Use this operation to manage and monitor your custom model…
        (
            "ListCustomModelDeployments",
            OperationOverlay {
                queryable: vec![path("customModelDeploymentArn")?],
                ..read(
                    "modelDeploymentSummaries",
                    Identity::Arn(path("customModelDeploymentArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of the custom models that you have created with the CreateModelCustomizationJob operation. For more information, see Custom models in the Amazon Bedrock User Guide .
        (
            "ListCustomModels",
            OperationOverlay {
                queryable: vec![path("modelArn")?],
                ..read(
                    "modelSummaries",
                    Identity::Arn(path("modelArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListEnforcedGuardrailsConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is guardrailsConfig; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the account-level enforced guardrail configurations.",
            ),
        ),
        // Model: Lists all existing evaluation jobs.
        (
            "ListEvaluationJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "jobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListFoundationModelAgreementOffers",
            deferred(
                "Model requires modelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the offers associated with the specified model.",
            ),
        ),
        (
            "ListFoundationModels",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is modelSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists Amazon Bedrock foundation models that you can use. You can filter the results with the request parameters. For more information, see Foundation models in the Amazon Bedrock User Guide .",
            ),
        ),
        // Model: Lists details about all the guardrails in an account. To list the DRAFT version of all your guardrails, don't specify the guardrailIdentifier field. To list all versions of a guardrail, specify the…
        (
            "ListGuardrails",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "guardrails",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of models you've imported. You can filter the results to return based on one or more criteria. For more information, see Import a customized model in the Amazon Bedrock User Guide .
        (
            "ListImportedModels",
            OperationOverlay {
                queryable: vec![path("modelArn")?],
                ..read(
                    "modelSummaries",
                    Identity::Arn(path("modelArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // APPLICATION profiles are customer-created resources; SYSTEM_DEFINED profiles are reference data.
        (
            "ListInferenceProfiles",
            OperationOverlay {
                fixed: [(
                    path("typeEquals")?,
                    FixedValue::String("APPLICATION".to_owned()),
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("inferenceProfileArn")?],
                ..read(
                    "inferenceProfileSummaries",
                    Identity::Arn(path("inferenceProfileArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists the endpoints for models from Amazon Bedrock Marketplace in your Amazon Web Services account.
        (
            "ListMarketplaceModelEndpoints",
            OperationOverlay {
                queryable: vec![path("endpointArn")?],
                ..read(
                    "marketplaceModelEndpoints",
                    Identity::Arn(path("endpointArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of model copy jobs that you have submitted. You can filter the jobs to return based on one or more criteria. For more information, see Copy models to be used in other regions in the…
        (
            "ListModelCopyJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "modelCopyJobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of model customization jobs that you have submitted. You can filter the jobs to return based on one or more criteria. For more information, see Custom models in the Amazon Bedrock User…
        (
            "ListModelCustomizationJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "modelCustomizationJobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of import jobs you've submitted. You can filter the results to return based on one or more criteria. For more information, see Import a customized model in the Amazon Bedrock User…
        (
            "ListModelImportJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "modelImportJobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all batch inference jobs in the account. For more information, see View details about a batch inference job .
        (
            "ListModelInvocationJobs",
            OperationOverlay {
                queryable: vec![path("jobArn")?],
                ..read(
                    "invocationJobSummaries",
                    Identity::Arn(path("jobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPromptRouters",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is promptRouterSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves a list of prompt routers.",
            ),
        ),
        // Model: Lists the Provisioned Throughputs in the account. For more information, see Provisioned Throughput in the Amazon Bedrock User Guide .
        (
            "ListProvisionedModelThroughputs",
            OperationOverlay {
                queryable: vec![path("provisionedModelArn")?],
                ..read(
                    "provisionedModelSummaries",
                    Identity::Arn(path("provisionedModelArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the tags associated with the specified resource. For more information, see Tagging resources in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "PutAccountDataRetention",
            deferred(
                "Model requires mode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the account-wide data retention mode for Amazon Bedrock.",
            ),
        ),
        (
            "PutEnforcedGuardrailConfiguration",
            deferred(
                "Model requires guardrailInferenceConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the account-level enforced guardrail configuration.",
            ),
        ),
        (
            "PutModelInvocationLoggingConfiguration",
            deferred(
                "Model requires loggingConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Set the configuration values for model invocation logging.",
            ),
        ),
        (
            "PutResourcePolicy",
            deferred(
                "Model requires resourceArn, resourcePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a resource policy for a Bedrock resource.",
            ),
        ),
        (
            "PutUseCaseForModelAccess",
            deferred(
                "Model requires formData. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Put usecase for model access.",
            ),
        ),
        (
            "RegisterMarketplaceModelEndpoint",
            deferred(
                "Model requires endpointIdentifier, modelSourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers an existing Amazon SageMaker endpoint with Amazon Bedrock Marketplace, allowing it to be used with Amazon Bedrock APIs.",
            ),
        ),
        (
            "StartAutomatedReasoningPolicyBuildWorkflow",
            deferred(
                "Model requires policyArn, buildWorkflowType, sourceContent. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a new build workflow for an Automated Reasoning policy. This initiates the process of analyzing source documents and generating policy rules, variables, and types.",
            ),
        ),
        (
            "StartAutomatedReasoningPolicyTestWorkflow",
            deferred(
                "Model requires policyArn, buildWorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a test workflow to validate Automated Reasoning policy tests. The workflow executes the specified tests against the policy and generates validation results.",
            ),
        ),
        (
            "StopAdvancedPromptOptimizationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an advanced prompt optimization job that is in progress.",
            ),
        ),
        (
            "StopEvaluationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an evaluation job that is current being created or running.",
            ),
        ),
    ])
}

pub(super) fn part_4() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "StopModelCustomizationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an active model customization job. For more information, see Custom models in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "StopModelInvocationJob",
            deferred(
                "Model requires jobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a batch inference job. You're only charged for tokens that were already processed. For more information, see Stop a batch inference job .",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceARN, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associate tags with a resource. For more information, see Tagging resources in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceARN, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove one or more tags from a resource. For more information, see Tagging resources in the Amazon Bedrock User Guide .",
            ),
        ),
        (
            "UpdateAutomatedReasoningPolicy",
            deferred(
                "Model requires policyArn, policyDefinition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Automated Reasoning policy with new rules, variables, or configuration. This creates a new version of the policy while preserving the previous version.",
            ),
        ),
        (
            "UpdateAutomatedReasoningPolicyAnnotations",
            deferred(
                "Model requires policyArn, buildWorkflowId, annotations, lastUpdatedAnnotationSetHash. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the annotations for an Automated Reasoning policy build workflow. This allows you to modify extracted rules, variables, and types before finalizing the policy.",
            ),
        ),
        (
            "UpdateAutomatedReasoningPolicyTestCase",
            deferred(
                "Model requires policyArn, testCaseId, guardContent, lastUpdatedAt, expectedAggregatedFindingsResult. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Automated Reasoning policy test. You can modify the content, query, expected result, and confidence threshold.",
            ),
        ),
        (
            "UpdateCustomModelDeployment",
            deferred(
                "Model requires modelArn, customModelDeploymentIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom model deployment with a new custom model. This allows you to deploy updated models without creating new deployment endpoints.",
            ),
        ),
        (
            "UpdateGuardrail",
            deferred(
                "Model requires guardrailIdentifier, name, blockedInputMessaging, blockedOutputsMessaging. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a guardrail with the values you specify. Specify a name and optional description . Specify messages for when the guardrail successfully blocks a prompt or a model response in the blockedInputMessaging and…",
            ),
        ),
        (
            "UpdateMarketplaceModelEndpoint",
            deferred(
                "Model requires endpointArn, endpointConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration of an existing endpoint for a model from Amazon Bedrock Marketplace.",
            ),
        ),
        (
            "UpdateProvisionedModelThroughput",
            deferred(
                "Model requires provisionedModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name or associated model for a Provisioned Throughput. For more information, see Provisioned Throughput in the Amazon Bedrock User Guide .",
            ),
        ),
    ])
}
