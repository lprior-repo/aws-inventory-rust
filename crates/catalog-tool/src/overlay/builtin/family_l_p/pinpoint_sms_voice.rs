use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateConfigurationSet",
            "CreateConfigurationSetEventDestination",
            "DeleteConfigurationSet",
            "DeleteConfigurationSetEventDestination",
            "SendVoiceMessage",
            "UpdateConfigurationSetEventDestination",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetConfigurationSetEventDestinations"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConfigurationSetName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListConfigurationSets",
    items: "ConfigurationSets",
    id: Id::Key(&["$"]),
    scope: Scope::Region,
    tokens: Some(("NextToken", "NextToken")),
    query: &["$"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pinpoint-sms-voice", GROUPS, RESOURCES)
}
