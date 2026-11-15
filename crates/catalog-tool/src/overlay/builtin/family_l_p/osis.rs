// Pinned Smithy source: .local/aws-models/models/osis/service/2022-01-01/osis-2022-01-01.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreatePipeline", "CreatePipelineEndpoint", "DeletePipeline", 
            "DeletePipelineEndpoint", "DeleteResourcePolicy", "PutResourcePolicy", 
            "RevokePipelineEndpointConnections", "StartPipeline", "StopPipeline", 
            "TagResource", "UntagResource", "UpdatePipeline",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetPipeline", "GetPipelineBlueprint",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "GetPipelineChangeProgress",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PipelineName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourcePolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListPipelineBlueprints", "ListPipelineEndpointConnections",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ValidatePipeline",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PipelineConfigurationBody); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists all OpenSearch Ingestion pipelines in the current Amazon Web Services account and Region. For more information, see Viewing Amazon OpenSearch Ingestion pipelines .
    Resource {
        operation: "ListPipelines",
        items: "Pipelines", id: Id::Arn("PipelineArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["PipelineArn", "Status", "PipelineName"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists all pipeline endpoints in your account.
    Resource {
        operation: "ListPipelineEndpoints",
        items: "PipelineEndpoints", id: Id::Key(&["EndpointId"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["EndpointId", "PipelineArn", "Status", "VpcId"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("osis", GROUPS, RESOURCES)
}
