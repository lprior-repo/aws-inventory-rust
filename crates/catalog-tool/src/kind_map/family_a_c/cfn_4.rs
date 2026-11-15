use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Bedrock::AutomatedReasoningPolicy"),
    CfnMapping::canonical("AWS::Bedrock::Blueprint"),
    CfnMapping::canonical("AWS::Bedrock::DataAutomationLibrary"),
    CfnMapping::canonical("AWS::Bedrock::DataAutomationProject"),
    CfnMapping::canonical("AWS::Bedrock::Flow"),
    CfnMapping::canonical("AWS::Bedrock::Guardrail"),
    CfnMapping::canonical("AWS::Bedrock::KnowledgeBase"),
    CfnMapping::canonical("AWS::Bedrock::Prompt"),
    CfnMapping::canonical("AWS::BedrockAgentCore::BrowserCustom"),
    CfnMapping::canonical("AWS::BedrockAgentCore::BrowserProfile"),
    CfnMapping::canonical("AWS::BedrockAgentCore::CapacityProvider"),
    CfnMapping::canonical("AWS::BedrockAgentCore::CodeInterpreterCustom"),
    CfnMapping::canonical("AWS::BedrockAgentCore::ConfigurationBundle"),
    CfnMapping::canonical("AWS::BedrockAgentCore::ConsentPortal"),
    CfnMapping::canonical("AWS::BedrockAgentCore::Dataset"),
    CfnMapping::canonical("AWS::BedrockAgentCore::Evaluator"),
    CfnMapping::canonical("AWS::BedrockAgentCore::Gateway"),
    CfnMapping::canonical("AWS::BedrockAgentCore::Harness"),
    CfnMapping::canonical("AWS::BedrockAgentCore::Memory"),
    CfnMapping::canonical("AWS::BedrockAgentCore::PolicyEngine"),
];
