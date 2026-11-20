use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "bedrock",
        operation: "ListInferenceProfiles",
        kind: "AWS::Bedrock::ApplicationInferenceProfile",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "ListPortals",
        kind: "AWS::ApiOnly::ApiGatewayV2Portal",
    },
    OperationMapping {
        service: "connect",
        operation: "ListSecurityProfiles",
        kind: "AWS::Connect::SecurityProfile",
    },
    OperationMapping {
        service: "connect",
        operation: "ListTrafficDistributionGroups",
        kind: "AWS::Connect::TrafficDistributionGroup",
    },
    OperationMapping {
        service: "connectcampaigns",
        operation: "DescribeCampaign",
        kind: "AWS::ConnectCampaigns::Campaign",
    },
    OperationMapping {
        service: "connectcampaigns",
        operation: "ListCampaigns",
        kind: "AWS::ConnectCampaigns::Campaign",
    },
    OperationMapping {
        service: "connectcampaignsv2",
        operation: "DescribeCampaign",
        kind: "AWS::ConnectCampaignsV2::Campaign",
    },
    OperationMapping {
        service: "connectcampaignsv2",
        operation: "ListCampaigns",
        kind: "AWS::ConnectCampaignsV2::Campaign",
    },
    OperationMapping {
        service: "connectcases",
        operation: "GetDomain",
        kind: "AWS::Cases::Domain",
    },
    OperationMapping {
        service: "connectcases",
        operation: "ListDomains",
        kind: "AWS::Cases::Domain",
    },
    OperationMapping {
        service: "connecthealth",
        operation: "ListDomains",
        kind: "AWS::ApiOnly::ConnectHealthDomain",
    },
    OperationMapping {
        service: "continuum",
        operation: "ListCollectors",
        kind: "AWS::ApiOnly::ContinuumCollector",
    },
    OperationMapping {
        service: "continuum",
        operation: "ListEnvironments",
        kind: "AWS::ApiOnly::ContinuumEnvironment",
    },
    OperationMapping {
        service: "continuum",
        operation: "ListIntegrations",
        kind: "AWS::ApiOnly::ContinuumIntegration",
    },
    OperationMapping {
        service: "controltower",
        operation: "ListControlOperations",
        kind: "AWS::ApiOnly::ControlTowerControlOperation",
    },
    OperationMapping {
        service: "controltower",
        operation: "ListEnabledBaselines",
        kind: "AWS::ControlTower::EnabledBaseline",
    },
    OperationMapping {
        service: "controltower",
        operation: "ListEnabledControls",
        kind: "AWS::ControlTower::EnabledControl",
    },
    OperationMapping {
        service: "controltower",
        operation: "ListLandingZoneOperations",
        kind: "AWS::ApiOnly::ControlTowerLandingZoneOperation",
    },
    OperationMapping {
        service: "cost-and-usage-report-service",
        operation: "DescribeReportDefinitions",
        kind: "AWS::CUR::ReportDefinition",
    },
    OperationMapping {
        service: "customer-profiles",
        operation: "GetDomain",
        kind: "AWS::CustomerProfiles::Domain",
    },
    OperationMapping {
        service: "customer-profiles",
        operation: "ListDomains",
        kind: "AWS::CustomerProfiles::Domain",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "ListLogGroups",
        kind: "AWS::Logs::LogGroup",
    },
    OperationMapping {
        service: "config-service",
        operation: "ListConfigurationRecorders",
        kind: "AWS::Config::ConfigurationRecorder",
    },
    OperationMapping {
        service: "config-service",
        operation: "DescribeConfigRuleEvaluationStatus",
        kind: "AWS::ApiOnly::ConfigRuleEvaluationStatus",
    },
    OperationMapping {
        service: "config-service",
        operation: "DescribeConfigurationRecorderStatus",
        kind: "AWS::ApiOnly::ConfigurationRecorderStatus",
    },
    OperationMapping {
        service: "config-service",
        operation: "DescribeConformancePackStatus",
        kind: "AWS::ApiOnly::ConformancePackStatus",
    },
    OperationMapping {
        service: "arc-region-switch",
        operation: "ListPlansInRegion",
        kind: "AWS::ARCRegionSwitch::Plan",
    },
    OperationMapping {
        service: "batch",
        operation: "ListServiceJobs",
        kind: "AWS::ApiOnly::BatchServiceJob",
    },
    OperationMapping {
        service: "chime-sdk-media-pipelines",
        operation: "ListMediaCapturePipelines",
        kind: "AWS::ApiOnly::ChimeSDKMediaPipelinesMediaPipeline",
    },
    OperationMapping {
        service: "chime-sdk-voice",
        operation: "ListPhoneNumbers",
        kind: "AWS::ApiOnly::ChimeSDKVoicePhoneNumber",
    },
    OperationMapping {
        service: "chime-sdk-voice",
        operation: "ListVoiceProfileDomains",
        kind: "AWS::ApiOnly::ChimeSDKVoiceProfileDomain",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListTypes",
        kind: "AWS::ApiOnly::CloudFormationRegistryType",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListTypeVersions",
        kind: "AWS::ApiOnly::CloudFormationRegistryTypeVersion",
    },
    OperationMapping {
        service: "codebuild",
        operation: "ListSourceCredentials",
        kind: "AWS::CodeBuild::SourceCredential",
    },
];
