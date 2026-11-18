use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "bcm-pricing-calculator",
        operation: "ListBillScenarios",
        kind: "AWS::ApiOnly::BCMPricingCalculatorBillScenario",
    },
    OperationMapping {
        service: "bcm-pricing-calculator",
        operation: "ListWorkloadEstimates",
        kind: "AWS::ApiOnly::BCMPricingCalculatorWorkloadEstimate",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "GetKnowledgeBase",
        kind: "AWS::Bedrock::KnowledgeBase",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "ListAgents",
        kind: "AWS::Bedrock::Agent",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "ListFlows",
        kind: "AWS::Bedrock::Flow",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "ListKnowledgeBases",
        kind: "AWS::Bedrock::KnowledgeBase",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "ListPrompts",
        kind: "AWS::Bedrock::Prompt",
    },
    OperationMapping {
        service: "bedrock-agent",
        operation: "ListRetrievalInstructions",
        kind: "AWS::ApiOnly::BedrockRetrievalInstruction",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "GetBrowser",
        kind: "AWS::BedrockAgentCore::BrowserCustom",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "GetCodeInterpreter",
        kind: "AWS::BedrockAgentCore::CodeInterpreterCustom",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListAgentRuntimes",
        kind: "AWS::BedrockAgentCore::Runtime",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListBrowserProfiles",
        kind: "AWS::BedrockAgentCore::BrowserProfile",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListBrowsers",
        kind: "AWS::BedrockAgentCore::BrowserCustom",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListCapacityProviders",
        kind: "AWS::BedrockAgentCore::CapacityProvider",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListCodeInterpreters",
        kind: "AWS::BedrockAgentCore::CodeInterpreterCustom",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListConfigurationBundles",
        kind: "AWS::BedrockAgentCore::ConfigurationBundle",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListConsentPortals",
        kind: "AWS::BedrockAgentCore::ConsentPortal",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListDatasets",
        kind: "AWS::BedrockAgentCore::Dataset",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListEvaluators",
        kind: "AWS::BedrockAgentCore::Evaluator",
    },
    OperationMapping {
        service: "bedrock-agentcore-control",
        operation: "ListGateways",
        kind: "AWS::BedrockAgentCore::Gateway",
    },
];
