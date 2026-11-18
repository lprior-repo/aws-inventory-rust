use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "bedrock",
        operation: "ListModelCopyJobs",
        kind: "AWS::ApiOnly::BedrockModelCopyJob",
    },
    OperationMapping {
        service: "bedrock",
        operation: "ListModelCustomizationJobs",
        kind: "AWS::ApiOnly::BedrockModelCustomizationJob",
    },
    OperationMapping {
        service: "bedrock",
        operation: "ListModelImportJobs",
        kind: "AWS::ApiOnly::BedrockModelImportJob",
    },
    OperationMapping {
        service: "bedrock",
        operation: "ListModelInvocationJobs",
        kind: "AWS::ApiOnly::BedrockModelInvocationJob",
    },
    OperationMapping {
        service: "bedrock",
        operation: "ListProvisionedModelThroughputs",
        kind: "AWS::ApiOnly::BedrockProvisionedModelThroughput",
    },
    OperationMapping {
        service: "billing",
        operation: "ListBillingViews",
        kind: "AWS::Billing::BillingView",
    },
    OperationMapping {
        service: "billingconductor",
        operation: "ListBillingGroups",
        kind: "AWS::BillingConductor::BillingGroup",
    },
    OperationMapping {
        service: "billingconductor",
        operation: "ListCustomLineItems",
        kind: "AWS::BillingConductor::CustomLineItem",
    },
    OperationMapping {
        service: "billingconductor",
        operation: "ListPricingPlans",
        kind: "AWS::BillingConductor::PricingPlan",
    },
    OperationMapping {
        service: "billingconductor",
        operation: "ListPricingRules",
        kind: "AWS::BillingConductor::PricingRule",
    },
    OperationMapping {
        service: "budgets",
        operation: "DescribeBudgets",
        kind: "AWS::Budgets::Budget",
    },
    OperationMapping {
        service: "chatbot",
        operation: "DescribeChimeWebhookConfigurations",
        kind: "AWS::ApiOnly::ChatbotChimeWebhookConfiguration",
    },
    OperationMapping {
        service: "chatbot",
        operation: "DescribeSlackChannelConfigurations",
        kind: "AWS::Chatbot::SlackChannelConfiguration",
    },
    OperationMapping {
        service: "chatbot",
        operation: "ListCustomActions",
        kind: "AWS::Chatbot::CustomAction",
    },
    OperationMapping {
        service: "chatbot",
        operation: "ListMicrosoftTeamsChannelConfigurations",
        kind: "AWS::Chatbot::MicrosoftTeamsChannelConfiguration",
    },
    OperationMapping {
        service: "chime-sdk-media-pipelines",
        operation: "ListMediaInsightsPipelineConfigurations",
        kind: "AWS::ApiOnly::ChimeSDKMediaPipelinesMediaInsightsPipelineConfiguration",
    },
    OperationMapping {
        service: "chime-sdk-media-pipelines",
        operation: "ListMediaPipelineKinesisVideoStreamPools",
        kind: "AWS::ApiOnly::ChimeSDKMediaPipelinesMediaPipelineKinesisVideoStreamPool",
    },
    OperationMapping {
        service: "chime-sdk-media-pipelines",
        operation: "ListMediaPipelines",
        kind: "AWS::ApiOnly::ChimeSDKMediaPipelinesMediaPipeline",
    },
    OperationMapping {
        service: "chime-sdk-voice",
        operation: "GetSipMediaApplication",
        kind: "AWS::Chime::SipMediaApplication",
    },
    OperationMapping {
        service: "chime-sdk-voice",
        operation: "GetSipRule",
        kind: "AWS::ApiOnly::ChimeSipRule",
    },
];
