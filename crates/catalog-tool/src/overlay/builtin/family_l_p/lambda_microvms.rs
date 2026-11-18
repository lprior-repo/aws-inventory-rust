// Pinned Smithy source: .local/aws-models/models/lambda-microvms/service/2025-09-09/lambda-microvms-2025-09-09.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateMicrovmAuthToken",
            "CreateMicrovmImage",
            "CreateMicrovmShellAuthToken",
            "DeleteMicrovmImage",
            "DeleteMicrovmImageVersion",
            "ResumeMicrovm",
            "RunMicrovm",
            "SuspendMicrovm",
            "TagResource",
            "TerminateMicrovm",
            "UntagResource",
            "UpdateMicrovmImage",
            "UpdateMicrovmImageVersion",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetMicrovm",
            "GetMicrovmImage",
            "GetMicrovmImageBuild",
            "GetMicrovmImageVersion",
            "ListMicrovmImageBuilds",
            "ListMicrovmImageVersions",
            "ListTags",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Image builds and versions require image identity/version context; microvm and customer-image summaries are the authorized resources.",
        ),
    },
    Group {
        operations: &[
            "ListManagedMicrovmImageVersions",
            "ListManagedMicrovmImages",
        ],
        class: OverlayClass::Deny(DenyReason::ReferenceData),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    // Lists MicroVMs in the account with optional filtering by image and version. We recommend using pagination to ensure that the operation returns quickly and successfully.
    Resource {
        operation: "ListMicrovms",
        items: "items",
        id: Id::Key(&["microvmId"]),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["microvmId", "state", "imageArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists MicroVM images in the account with optional name filtering. We recommend using pagination to ensure that the operation returns quickly and successfully.
    Resource {
        operation: "ListMicrovmImages",
        items: "items",
        id: Id::Arn("imageArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["imageArn", "name", "state"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lambda-microvms", GROUPS, RESOURCES)
}
