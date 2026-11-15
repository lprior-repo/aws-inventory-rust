use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "bedrock-agentcore-control", operation: "ListHarnesses", kind: "AWS::BedrockAgentCore::Harness" },
    OperationMapping { service: "bedrock-agentcore-control", operation: "ListMemories", kind: "AWS::BedrockAgentCore::Memory" },
    OperationMapping { service: "bedrock-agentcore-control", operation: "ListPolicyEngineSummaries", kind: "AWS::BedrockAgentCore::PolicyEngine" },
    OperationMapping { service: "bedrock-agentcore-control", operation: "ListRegistries", kind: "AWS::ApiOnly::BedrockAgentCoreRegistry" },
    OperationMapping { service: "bedrock-agentcore-control", operation: "ListWorkloadIdentities", kind: "AWS::BedrockAgentCore::WorkloadIdentity" },
    OperationMapping { service: "bedrock-data-automation", operation: "ListBlueprints", kind: "AWS::Bedrock::Blueprint" },
    OperationMapping { service: "bedrock-data-automation", operation: "ListDataAutomationLibraries", kind: "AWS::Bedrock::DataAutomationLibrary" },
    OperationMapping { service: "bedrock-data-automation", operation: "ListDataAutomationProjects", kind: "AWS::Bedrock::DataAutomationProject" },
    OperationMapping { service: "bedrock-runtime", operation: "ListAsyncInvokes", kind: "AWS::ApiOnly::BedrockAsyncInvocation" },
    OperationMapping { service: "bedrock", operation: "GetCustomModel", kind: "AWS::ApiOnly::BedrockCustomModel" },
    OperationMapping { service: "bedrock", operation: "GetImportedModel", kind: "AWS::ApiOnly::BedrockImportedModel" },
    OperationMapping { service: "bedrock", operation: "GetProvisionedModelThroughput", kind: "AWS::ApiOnly::BedrockProvisionedModelThroughput" },
    OperationMapping { service: "bedrock", operation: "ListAdvancedPromptOptimizationJobs", kind: "AWS::ApiOnly::BedrockAdvancedPromptOptimizationJob" },
    OperationMapping { service: "bedrock", operation: "ListAutomatedReasoningPolicies", kind: "AWS::Bedrock::AutomatedReasoningPolicy" },
    OperationMapping { service: "bedrock", operation: "ListCustomModelDeployments", kind: "AWS::ApiOnly::BedrockCustomModelDeployment" },
    OperationMapping { service: "bedrock", operation: "ListCustomModels", kind: "AWS::ApiOnly::BedrockCustomModel" },
    OperationMapping { service: "bedrock", operation: "ListEvaluationJobs", kind: "AWS::ApiOnly::BedrockEvaluationJob" },
    OperationMapping { service: "bedrock", operation: "ListGuardrails", kind: "AWS::Bedrock::Guardrail" },
    OperationMapping { service: "bedrock", operation: "ListImportedModels", kind: "AWS::ApiOnly::BedrockImportedModel" },
    OperationMapping { service: "bedrock", operation: "ListMarketplaceModelEndpoints", kind: "AWS::ApiOnly::BedrockMarketplaceModelEndpoint" },
];
