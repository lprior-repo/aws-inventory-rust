use crate::kind_map::{ApiOnlyMapping, CfnMapping, OperationMapping, RelationMapping, RelationPath, RelationSource, RelationTarget};

pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::GameLift::Script"),
    CfnMapping::canonical("AWS::GameLift::Location"),
    CfnMapping::canonical("AWS::ImageBuilder::Component"),
    CfnMapping::canonical("AWS::ImageBuilder::Workflow"),
    CfnMapping::canonical("AWS::Glue::Blueprint"),
    CfnMapping::canonical("AWS::Glue::DevEndpoint"),
    CfnMapping::canonical("AWS::Glue::MLTransform"),
    CfnMapping::canonical("AWS::Glue::Trigger"),
    CfnMapping::canonical("AWS::Glue::DataQualityRuleset"),
    CfnMapping::canonical("AWS::Glue::UsageProfile"),
    CfnMapping::canonical("AWS::Glue::SecurityConfiguration"),
    CfnMapping::canonical("AWS::InspectorV2::Connector"),
    CfnMapping::canonical("AWS::Invoicing::ProcurementPortalPreference"),
];

pub(super) const API_ONLY: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping { kind: "AWS::ApiOnly::ImageBuilderComponentSemanticVersion", reason: "A three-node semantic-version ARN represents a version family, not the four-node build ARN of CFN Component." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::ImageBuilderImageSemanticVersion", reason: "A three-node semantic-version ARN represents a version family, not the four-node build ARN of CFN Image." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::ImageBuilderWorkflowSemanticVersion", reason: "A three-node semantic-version ARN represents a version family, not the four-node build ARN of CFN Workflow." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::GlacierProvisionedCapacity", reason: "Existing account provisioned-capacity ID and reservation dates have no pinned CFN resource type; listing does not purchase capacity." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::Inspector2ConnectorScanConfiguration", reason: "Inspector scan durations associated with one AWS Config connector ARN are distinct from the Inspector Connector resource; no pinned CFN type exists." },
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping { service: "gamelift", operation: "ListScripts", kind: "AWS::GameLift::Script" },
    OperationMapping { service: "gamelift", operation: "ListLocations", kind: "AWS::GameLift::Location" },
    OperationMapping { service: "imagebuilder", operation: "ListComponentBuildVersions", kind: "AWS::ImageBuilder::Component" },
    OperationMapping { service: "imagebuilder", operation: "ListWorkflowBuildVersions", kind: "AWS::ImageBuilder::Workflow" },
    OperationMapping { service: "imagebuilder", operation: "ListComponents", kind: "AWS::ApiOnly::ImageBuilderComponentSemanticVersion" },
    OperationMapping { service: "imagebuilder", operation: "ListImages", kind: "AWS::ApiOnly::ImageBuilderImageSemanticVersion" },
    OperationMapping { service: "imagebuilder", operation: "ListWorkflows", kind: "AWS::ApiOnly::ImageBuilderWorkflowSemanticVersion" },
    OperationMapping { service: "glue", operation: "ListBlueprints", kind: "AWS::Glue::Blueprint" },
    OperationMapping { service: "glue", operation: "ListDevEndpoints", kind: "AWS::Glue::DevEndpoint" },
    OperationMapping { service: "glue", operation: "ListMLTransforms", kind: "AWS::Glue::MLTransform" },
    OperationMapping { service: "glue", operation: "ListTriggers", kind: "AWS::Glue::Trigger" },
    OperationMapping { service: "glue", operation: "ListDataQualityRulesets", kind: "AWS::Glue::DataQualityRuleset" },
    OperationMapping { service: "glue", operation: "ListUsageProfiles", kind: "AWS::Glue::UsageProfile" },
    OperationMapping { service: "glue", operation: "GetSecurityConfigurations", kind: "AWS::Glue::SecurityConfiguration" },
    OperationMapping { service: "glacier", operation: "ListProvisionedCapacity", kind: "AWS::ApiOnly::GlacierProvisionedCapacity" },
    OperationMapping { service: "inspector2", operation: "ListConnectors", kind: "AWS::InspectorV2::Connector" },
    OperationMapping { service: "inspector2", operation: "ListConnectorScanConfigurations", kind: "AWS::ApiOnly::Inspector2ConnectorScanConfiguration" },
    OperationMapping { service: "invoicing", operation: "ListProcurementPortalPreferences", kind: "AWS::Invoicing::ProcurementPortalPreference" },
];

pub(super) const RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping { source_kind: "AWS::InspectorV2::Connector", source: RelationSource::SelectedOutput(RelationPath::Fields("awsConfigConnectorArn")), target_kind: "AWS::ApiOnly::Inspector2ConnectorScanConfiguration", target: RelationTarget::SelectedOutput(RelationPath::Fields("awsConfigConnectorArn")), relation: "USES_SCAN_CONFIGURATION", scope: "account_region", guard: None },
];
