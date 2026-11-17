// Pinned Smithy source: .local/aws-models/models/codepipeline/service/2015-07-09/codepipeline-2015-07-09.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcknowledgeJob", "AcknowledgeThirdPartyJob", "CreateCustomActionType",
            "CreatePipeline", "DeleteCustomActionType", "DeletePipeline", "DeleteWebhook",
            "DeregisterWebhookWithThirdParty", "DisableStageTransition", "EnableStageTransition",
            "PollForJobs", "PollForThirdPartyJobs", "PutActionRevision", "PutJobSuccessResult",
            "PutJobFailureResult", "PutThirdPartyJobSuccessResult", "PutThirdPartyJobFailureResult",
            "PutWebhook", "TagResource", "UntagResource", "UpdatePipeline",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetActionType", "ListActionTypes", "ListRuleTypes",
        ],
        class: OverlayClass::Defer,
        reason: Some("Action types and rule types are AWS-managed reference data, not customer account/region configuration. Enumerating these would catalog AWS infrastructure, not the customer's resources."),
    },
    Group {
        operations: &[
            "GetJobDetails", "GetThirdPartyJobDetails",
        ],
        class: OverlayClass::Defer,
        reason: Some("Job details require a jobId that is only obtained through the job worker API (PollForJobs/PollForThirdPartyJobs), which are mutating operations not available to the catalog. No enumerate path exists to discover jobIds."),
    },
    Group {
        operations: &[
            "GetPipelineExecution",
        ],
        class: OverlayClass::Defer,
        reason: Some("Requires pipelineExecutionId which is a runtime execution identifier. While pipeline executions can be listed via ListPipelineExecutions, individual execution details contain runtime state that is not part of persistent configuration inventory."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListPipelines",
        items: "pipelines",
        id: Id::Key(&["name"]),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetPipeline",
        items: "pipeline",
        id: Id::Key(&["name"]),
        scope: Scope::Region,
        tokens: None,
        query: &["name"],
        tags: None,
        bindings: &[("name", "ListPipelines", "name")],
        fixed: &[],
    },
    Resource {
        operation: "GetPipelineState",
        items: "stageStates",
        id: Id::Key(&["pipelineName"]),
        scope: Scope::Region,
        tokens: None,
        query: &["pipelineName"],
        tags: None,
        bindings: &[("pipelineName", "ListPipelines", "name")],
        fixed: &[],
    },
    Resource {
        operation: "ListWebhooks",
        items: "webhooks",
        id: Id::Key(&["name"]),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("codepipeline", GROUPS, RESOURCES)
}
