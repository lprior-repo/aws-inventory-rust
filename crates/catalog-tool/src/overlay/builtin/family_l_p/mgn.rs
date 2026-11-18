// Pinned Smithy source: .local/aws-models/models/mgn/service/2020-02-26/mgn-2020-02-26.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "ArchiveApplication",
            "ArchiveWave",
            "AssociateApplications",
            "AssociateSourceServers",
            "ChangeServerLifeCycleState",
            "CreateApplication",
            "CreateConnector",
            "CreateLaunchConfigurationTemplate",
            "CreateNetworkMigrationDefinition",
            "CreateReplicationConfigurationTemplate",
            "CreateWave",
            "DeleteApplication",
            "DeleteConnector",
            "DeleteJob",
            "DeleteLaunchConfigurationTemplate",
            "DeleteNetworkMigrationDefinition",
            "DeleteReplicationConfigurationTemplate",
            "DeleteSourceServer",
            "DeleteVcenterClient",
            "DeleteWave",
            "DisassociateApplications",
            "DisassociateSourceServers",
            "DisconnectFromService",
            "FinalizeCutover",
            "InitializeService",
            "PauseReplication",
            "PutSourceServerAction",
            "PutTemplateAction",
            "RemoveSourceServerAction",
            "RemoveTemplateAction",
            "ResumeReplication",
            "RetryDataReplication",
            "StartCutover",
            "StartExport",
            "StartImport",
            "StartImportFileEnrichment",
            "StartNetworkMigrationAnalysis",
            "StartNetworkMigrationCodeGeneration",
            "StartNetworkMigrationDeployment",
            "StartNetworkMigrationMapping",
            "StartNetworkMigrationMappingUpdate",
            "StartReplication",
            "StartTest",
            "StopReplication",
            "TagResource",
            "TerminateTargetInstances",
            "UntagResource",
            "UpdateApplication",
            "UpdateConnector",
            "UpdateLaunchConfiguration",
            "UpdateLaunchConfigurationTemplate",
            "UpdateNetworkMigrationDefinition",
            "UpdateNetworkMigrationMapperSegment",
            "UpdateReplicationConfiguration",
            "UpdateReplicationConfigurationTemplate",
            "UpdateSourceServer",
            "UpdateSourceServerReplicationType",
            "UpdateWave",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeJobLogItems"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (jobID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DescribeJobs",
            "ListExports",
            "ListImportFileEnrichments",
            "ListImports",
            "ListManagedAccounts",
            "ListNetworkMigrationDefinitions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "GetLaunchConfiguration",
            "GetReplicationConfiguration",
            "ListSourceServerActions",
            "MarkAsArchived",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (sourceServerID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetNetworkMigrationDefinition",
            "ListNetworkMigrationExecutions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (networkMigrationDefinitionID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetNetworkMigrationMapperSegmentConstruct"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (networkMigrationDefinitionID, networkMigrationExecutionID, segmentID, constructID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListExportErrors"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (exportID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListImportErrors"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (importID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListNetworkMigrationAnalyses",
            "ListNetworkMigrationAnalysisResults",
            "ListNetworkMigrationCodeGenerationSegments",
            "ListNetworkMigrationCodeGenerations",
            "ListNetworkMigrationDeployedStacks",
            "ListNetworkMigrationDeployments",
            "ListNetworkMigrationMapperSegments",
            "ListNetworkMigrationMappingUpdates",
            "ListNetworkMigrationMappings",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (networkMigrationExecutionID, networkMigrationDefinitionID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListNetworkMigrationMapperSegmentConstructs"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (networkMigrationExecutionID, networkMigrationDefinitionID, segmentID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTemplateActions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (launchConfigurationTemplateID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["UnarchiveApplication"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (applicationID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["UnarchiveWave"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (waveID); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieves all SourceServers or multiple SourceServers by ID.
    Resource {
        operation: "DescribeSourceServers",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "sourceServerID",
            "replicationType",
            "vcenterClientID",
            "applicationID",
            "userProvidedID",
        ],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists all ReplicationConfigurationTemplates, filtered by replication configuration template IDs.
    Resource {
        operation: "DescribeReplicationConfigurationTemplates",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "replicationConfigurationTemplateID",
            "stagingAreaSubnetId",
            "replicationServerInstanceType",
            "defaultLargeStagingDiskType",
            "ebsEncryptionKeyArn",
        ],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists all Launch Configuration Templates, filtered by Launch Configuration Template IDs
    Resource {
        operation: "DescribeLaunchConfigurationTemplates",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "launchConfigurationTemplateID",
            "mapAutoTaggingMpeID",
            "ec2LaunchTemplateID",
        ],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of the installed vCenter clients.
    Resource {
        operation: "DescribeVcenterClients",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "vcenterClientID",
            "hostname",
            "vcenterUUID",
            "datacenterName",
        ],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // Retrieves all applications or multiple applications by ID.
    Resource {
        operation: "ListApplications",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "applicationID", "name", "waveID"],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // List Connectors.
    Resource {
        operation: "ListConnectors",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "connectorID", "name", "ssmInstanceID"],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    // Retrieves all waves or multiple waves by ID.
    Resource {
        operation: "ListWaves",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "waveID", "name"],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mgn", GROUPS, RESOURCES)
}
