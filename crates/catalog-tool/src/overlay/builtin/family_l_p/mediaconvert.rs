use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateCertificate",
            "CancelJob",
            "CreateJob",
            "CreateJobTemplate",
            "CreatePreset",
            "CreateQueue",
            "CreateResourceShare",
            "DeleteJobTemplate",
            "DeletePolicy",
            "DeletePreset",
            "DeleteQueue",
            "DisassociateCertificate",
            "PutPolicy",
            "StartJobsQuery",
            "TagResource",
            "UntagResource",
            "UpdateJobTemplate",
            "UpdatePreset",
            "UpdateQueue",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeEndpoints",
            "GetPolicy",
            "ListVersions",
            "Probe",
            "SearchJobs",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "GetJob",
            "GetJobTemplate",
            "GetPreset",
            "ListJobTemplates",
            "ListJobs",
            "ListPresets",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["GetJobsQueryResults"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Arn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListQueues",
        items: "Queues",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "Status", "Type"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetQueue",
        items: "Queue",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: None,
        query: &["Arn", "Name", "Status", "Type"],
        tags: None,
        bindings: &[("Name", "ListQueues", "Name")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediaconvert", GROUPS, RESOURCES)
}
