// Pinned Smithy source: .local/aws-models/models/polly/service/2016-06-10/polly-2016-06-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "DeleteLexicon", "PutLexicon", "StartSpeechSynthesisStream", "TagResource", 
            "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeVoices",
        ],
        class: OverlayClass::Deny(DenyReason::ReferenceData),
        reason: None,
    },
    Group {
        operations: &[
            "GetLexicon",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "GetSpeechSynthesisTask",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (TaskId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListSpeechSynthesisTasks",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "StartSpeechSynthesisTask", "SynthesizeSpeech",
        ],
        class: OverlayClass::Deny(DenyReason::Paid),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of pronunciation lexicons stored in an Amazon Web Services Region. For more information, see Managing Lexicons .
    Resource {
        operation: "ListLexicons",
        items: "Lexicons", id: Id::Arn("Attributes.LexiconArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Attributes.LexiconArn", "Name", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("polly", GROUPS, RESOURCES)
}
