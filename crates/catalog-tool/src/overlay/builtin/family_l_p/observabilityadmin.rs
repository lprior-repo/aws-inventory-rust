// Pinned Smithy source: .local/aws-models/models/observabilityadmin/service/2018-05-10/observabilityadmin-2018-05-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateCentralizationRuleForOrganization", "CreateDatasetIntegration", 
            "CreateS3TableIntegration", "CreateTelemetryPipeline", "CreateTelemetryRule", 
            "CreateTelemetryRuleForOrganization", "DeleteCentralizationRuleForOrganization", 
            "DeleteDatasetIntegration", "DeleteS3TableIntegration", "DeleteTelemetryPipeline", 
            "DeleteTelemetryRule", "DeleteTelemetryRuleForOrganization", 
            "StartTelemetryEnrichment", "StartTelemetryEvaluation", 
            "StartTelemetryEvaluationForOrganization", "StopTelemetryEnrichment", 
            "StopTelemetryEvaluation", "StopTelemetryEvaluationForOrganization", "TagResource", 
            "UntagResource", "UpdateCentralizationRuleForOrganization", 
            "UpdateDatasetIntegration", "UpdateTelemetryPipeline", "UpdateTelemetryRule", 
            "UpdateTelemetryRuleForOrganization",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetCentralizationRuleForOrganization", "GetTelemetryRule", 
            "GetTelemetryRuleForOrganization",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (RuleIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDatasetIntegration", "GetS3TableIntegration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetTelemetryEnrichmentStatus", "GetTelemetryEvaluationStatus", 
            "GetTelemetryEvaluationStatusForOrganization", "ListResourceTelemetry", 
            "ListResourceTelemetryForOrganization",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "GetTelemetryPipeline",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PipelineIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceARN); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "TestTelemetryPipeline",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Records, Configuration); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ValidateTelemetryPipelineConfiguration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Configuration); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists all centralization rules in your organization. This operation can only be called by the organization's management account or a delegated administrator account.
    Resource {
        operation: "ListCentralizationRulesForOrganization",
        items: "CentralizationRuleSummaries", id: Id::Arn("RuleArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["RuleArn", "RuleName", "CreatorAccountId", "CreatedRegion", "TagPropagationStatus", "ContextGraphStatus", "DestinationAccountId", "DestinationRegion"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists all telemetry rules in your account. You can filter the results by specifying a rule name prefix.
    Resource {
        operation: "ListTelemetryRules",
        items: "TelemetryRuleSummaries", id: Id::Arn("RuleArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["RuleArn", "RuleName", "ResourceType", "TelemetryType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists all telemetry rules in your organization. This operation can only be called by the organization's management account or a delegated administrator account.
    Resource {
        operation: "ListTelemetryRulesForOrganization",
        items: "TelemetryRuleSummaries", id: Id::Arn("RuleArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["RuleArn", "RuleName", "ResourceType", "TelemetryType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of telemetry pipelines in your account. Returns up to 100 results. If more than 100 telemetry pipelines exist, include the NextToken value from the response to retri
    Resource {
        operation: "ListTelemetryPipelines",
        items: "PipelineSummaries", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "Status"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists all S3 Table integrations in your account. We recommend using pagination to ensure that the operation returns quickly and successfully.
    Resource {
        operation: "ListS3TableIntegrations",
        items: "IntegrationSummaries", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns the dataset integrations in your account.
    Resource {
        operation: "ListDatasetIntegrations",
        items: "DatasetIntegrationSummaries", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "RoleArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("observabilityadmin", GROUPS, RESOURCES)
}
