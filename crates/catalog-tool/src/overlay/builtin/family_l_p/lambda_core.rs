// Pinned Smithy source: .local/aws-models/models/lambda-core/service/2026-04-30/lambda-core-2026-04-30.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateNetworkConnector", "CreateRuntime", "CreateRuntimeAlias", 
            "CreateRuntimeVersion", "DeleteNetworkConnector", "DeleteRuntime", 
            "DeleteRuntimeAlias", "DeleteRuntimeVersion", "RestoreRuntime", 
            "UpdateNetworkConnector", "UpdateRuntime", "UpdateRuntimeAlias", 
            "UpdateRuntimeVersion",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetNetworkConnector", "GetRuntime", "GetRuntimeAlias", "GetRuntimeVersion", 
            "ListNetworkConnectorVersions", "ListRuntimeAliases", "ListRuntimeVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some("Runtime and connector detail needs a discovered identity/version and a separate output safety/identity contract; only connector summaries are authorized."),
    },
    Group {
        operations: &[
            "ListRuntimes",
        ],
        class: OverlayClass::Deny(DenyReason::ReferenceData),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a paginated list of network connectors in your account for the current Region. You can optionally filter results by connector state. Use the Marker parameter from a previou
    Resource {
        operation: "ListNetworkConnectors",
        items: "NetworkConnectors", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("Marker", "NextMarker")),
        query: &["Arn", "Name", "Id", "Type", "State", "LatestVersionArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lambda-core", GROUPS, RESOURCES)
}
