// Pinned Smithy source: .local/aws-models/models/mediaconvert/service/2017-08-29/mediaconvert-2017-08-29.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateCertificate", "CancelJob", "CreateJob", "CreateJobTemplate", 
            "CreatePreset", "CreateQueue", "CreateResourceShare", "DeleteJobTemplate", 
            "DeletePolicy", "DeletePreset", "DeleteQueue", "DisassociateCertificate", 
            "PutPolicy", "StartJobsQuery", "TagResource", "UntagResource", "UpdateJobTemplate", 
            "UpdatePreset", "UpdateQueue",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeEndpoints", "GetPolicy", "ListVersions", "Probe", "SearchJobs",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "GetJob", "GetJobTemplate", "GetPreset", "ListJobTemplates", "ListJobs", 
            "ListPresets",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "GetJobsQueryResults",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Id); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieve a JSON array of up to twenty of your queues. This will return the queues themselves, not just a list of them. To retrieve the next twenty queues, use the nextToken string 
    Resource {
        operation: "ListQueues",
        items: "Queues", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "Status", "Type"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Same pinned response resource shape as ListQueues; no content-bearing expansion.
    Resource {
        operation: "GetQueue",
        items: "Queue", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: None,
        query: &["Arn", "Name", "Status", "Type"],
        tags: None,
        bindings: &[("Name", "ListQueues", "Name")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediaconvert", GROUPS, RESOURCES)
}
