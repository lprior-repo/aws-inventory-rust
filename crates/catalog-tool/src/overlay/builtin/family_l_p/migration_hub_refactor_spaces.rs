// Pinned Smithy source: .local/aws-models/models/migration-hub-refactor-spaces/service/2021-10-26/migration-hub-refactor-spaces-2021-10-26.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateApplication", "CreateEnvironment", "CreateRoute", "CreateService", 
            "DeleteApplication", "DeleteEnvironment", "DeleteResourcePolicy", "DeleteRoute", 
            "DeleteService", "PutResourcePolicy", "TagResource", "UntagResource", 
            "UpdateRoute",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetApplication", "ListRoutes",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EnvironmentIdentifier, ApplicationIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetEnvironment", "ListEnvironmentVpcs",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EnvironmentIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourcePolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Identifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetRoute",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EnvironmentIdentifier, ApplicationIdentifier, RouteIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetService",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EnvironmentIdentifier, ApplicationIdentifier, ServiceIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists Amazon Web Services Migration Hub Refactor Spaces environments owned by a caller account or shared with the caller account.
    Resource {
        operation: "ListEnvironments",
        items: "EnvironmentSummaryList", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "EnvironmentId", "NetworkFabricType", "OwnerAccountId", "TransitGatewayId", "State"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists all the Amazon Web Services Migration Hub Refactor Spaces applications within an environment.
    Resource {
        operation: "ListApplications",
        items: "ApplicationSummaryList", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "OwnerAccountId", "CreatedByAccountId", "ApplicationId", "EnvironmentId", "VpcId", "ProxyType", "State"],
        tags: Some("Tags"),
        bindings: &[("EnvironmentIdentifier", "ListEnvironments", "EnvironmentId")],
        fixed: &[],
    },
    // Lists all the Amazon Web Services Migration Hub Refactor Spaces services within an application.
    Resource {
        operation: "ListServices",
        items: "ServiceSummaryList", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ServiceId", "Name", "OwnerAccountId", "CreatedByAccountId", "EnvironmentId", "ApplicationId", "VpcId", "EndpointType", "State"],
        tags: Some("Tags"),
        bindings: &[("EnvironmentIdentifier", "ListApplications", "EnvironmentId"), ("ApplicationIdentifier", "ListApplications", "ApplicationId")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("migration-hub-refactor-spaces", GROUPS, RESOURCES)
}
