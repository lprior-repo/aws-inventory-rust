// Pinned Smithy source: .local/aws-models/models/lex-models-v2/service/2020-08-07/lex-models-v2-2020-08-07.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "BatchCreateCustomVocabularyItem", "BatchDeleteCustomVocabularyItem", 
            "BatchUpdateCustomVocabularyItem", "BuildBotLocale", "CreateBot", "CreateBotAlias", 
            "CreateBotLocale", "CreateBotReplica", "CreateBotVersion", "CreateExport", 
            "CreateIntent", "CreateResourcePolicy", "CreateResourcePolicyStatement", 
            "CreateSlot", "CreateSlotType", "CreateTestSetDiscrepancyReport", 
            "CreateUploadUrl", "DeleteBot", "DeleteBotAlias", 
            "DeleteBotAnalyzerRecommendation", "DeleteBotLocale", "DeleteBotReplica", 
            "DeleteBotVersion", "DeleteCustomVocabulary", "DeleteExport", "DeleteImport", 
            "DeleteIntent", "DeleteResourcePolicy", "DeleteResourcePolicyStatement", 
            "DeleteSlot", "DeleteSlotType", "DeleteTestSet", "DeleteUtterances", 
            "GenerateBotElement", "StartBotAnalyzer", "StartBotRecommendation", 
            "StartBotResourceGeneration", "StartImport", "StartTestExecution", 
            "StartTestSetGeneration", "StopBotAnalyzer", "StopBotRecommendation", 
            "TagResource", "UntagResource", "UpdateBot", "UpdateBotAlias", "UpdateBotLocale", 
            "UpdateBotRecommendation", "UpdateExport", "UpdateIntent", "UpdateResourcePolicy", 
            "UpdateSlot", "UpdateSlotType", "UpdateTestSet",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeBot", "ListBotAnalyzerHistory", "ListBotReplicas",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotAlias",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botAliasId, botId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotAnalyzerRecommendation",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botAnalyzerRequestId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotLocale", "DescribeCustomVocabularyMetadata", "ListBotRecommendations", 
            "ListBotResourceGenerations", "ListCustomVocabularyItems", "ListIntents", 
            "ListSlotTypes",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion, localeId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotRecommendation", "ListRecommendedIntents",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion, localeId, botRecommendationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotReplica", "ListBotAliasReplicas", "ListBotVersionReplicas",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, replicaRegion); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotResourceGeneration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion, localeId, generationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeBotVersion", "ListBotLocales",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeExport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (exportId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeImport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (importId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeIntent",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (intentId, botId, botVersion, localeId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeResourcePolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeSlot",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (slotId, botId, botVersion, localeId, intentId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeSlotType",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (slotTypeId, botId, botVersion, localeId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeTestExecution", "GetTestExecutionArtifactsUrl",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (testExecutionId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeTestSet", "ListTestSetRecords",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (testSetId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeTestSetDiscrepancyReport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (testSetDiscrepancyReportId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeTestSetGeneration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (testSetGenerationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAggregatedUtterances",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, localeId, aggregationDuration); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListBuiltInIntents", "ListBuiltInSlotTypes",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (localeId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListExports", "ListImports", "ListTestExecutions", "ListTestSets",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListIntentMetrics", "ListIntentStageMetrics", "ListSessionMetrics", 
            "ListUtteranceMetrics",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, startDateTime, endDateTime, metrics); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListIntentPaths",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, startDateTime, endDateTime, intentPath); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListSessionAnalyticsData", "ListUtteranceAnalyticsData",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, startDateTime, endDateTime); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListSlots",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion, localeId, intentId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (resourceARN); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTestExecutionResultItems",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (testExecutionId, resultFilterBy); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "SearchAssociatedTranscripts",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (botId, botVersion, localeId, botRecommendationId, filters); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Gets a list of available bots.
    Resource {
        operation: "ListBots",
        items: "botSummaries", id: Id::Key(&["botId"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["botId", "botName", "botStatus", "botType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListBotAliases", items: "botAliasSummaries",
        id: Id::Correlated(&["botId"], &["botAliasId"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["botAliasId", "botAliasName", "botVersion", "botAliasStatus"], tags: None,
        bindings: &[("botId", "ListBots", "botId")], fixed: &[],
    },
    Resource {
        operation: "ListBotVersions", items: "botVersionSummaries",
        id: Id::Correlated(&["botId"], &["botVersion"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["botVersion", "botName", "botStatus"], tags: None,
        bindings: &[("botId", "ListBots", "botId")], fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lex-models-v2", GROUPS, RESOURCES)
}
