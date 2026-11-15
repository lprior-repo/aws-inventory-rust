// Pinned Smithy source: .local/aws-models/models/lex-model-building-service/service/2017-04-19/lex-model-building-service-2017-04-19.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateBotVersion", "CreateIntentVersion", "CreateSlotTypeVersion", "DeleteBot", 
            "DeleteBotAlias", "DeleteBotChannelAssociation", "DeleteBotVersion", 
            "DeleteIntent", "DeleteIntentVersion", "DeleteSlotType", "DeleteSlotTypeVersion", 
            "DeleteUtterances", "PutBot", "PutBotAlias", "PutIntent", "PutSlotType", 
            "StartImport", "StartMigration", "TagResource", "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetBot",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (name, versionOrAlias); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBotAlias",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (name, botName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBotAliases",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBotChannelAssociation",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (name, botName, botAlias); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBotChannelAssociations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botName, botAlias); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBotVersions", "GetIntentVersions", "GetSlotTypeVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBuiltinIntent",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (signature); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBuiltinIntents", "GetBuiltinSlotTypes", "GetIntents", "GetMigrations", 
            "GetSlotTypes",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "GetExport", "GetIntent", "GetSlotType", "GetUtterancesView",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "GetImport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (importId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetMigration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (migrationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
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
    // Returns bot information as follows: If you provide the nameContains field, the response includes information for the $LATEST version of all bots whose name contains the specified s
    Resource {
        operation: "GetBots",
        items: "bots", id: Id::Key(&["name"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lex-model-building-service", GROUPS, RESOURCES)
}
