// Pinned Smithy source: .local/aws-models/models/migrationhubstrategy/service/2020-02-19/migrationhubstrategy-2020-02-19.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "GetApplicationComponentDetails", "GetApplicationComponentStrategies",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (applicationComponentId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetAssessment", "GetImportFileTask", "GetRecommendationReportDetails",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAnalyzableServers", "ListApplicationComponents", "ListImportFileTask", 
            "ListServers",
        ],
        class: OverlayClass::Enumerate,
        reason: None,
    },
    Group {
        operations: &[
            "GetLatestAssessmentId", "GetPortfolioPreferences", "GetPortfolioSummary",
        ],
        class: OverlayClass::Defer,
        reason: Some("Settings and reference data, not a resource collection. Pinned operations: Gets settings, references, or state."),
    },
    Group {
        operations: &[
            "GetServerDetails", "GetServerStrategies",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (serverId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "PutPortfolioPreferences", "StartAssessment", "StartImportFileTask", 
            "StartRecommendationReportGeneration", "StopAssessment", 
            "UpdateApplicationComponentConfig", "UpdateServerConfig",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieves a list of all the installed collectors.
    Resource {
        operation: "ListCollectors",
        items: "Collectors", id: Id::Key(&["collectorId"]),
        scope: Scope::HomeRegion, tokens: Some(("nextToken", "nextToken")),
        query: &["collectorId", "hostName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("migrationhubstrategy", GROUPS, RESOURCES)
}
