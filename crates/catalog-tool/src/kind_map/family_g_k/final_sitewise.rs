use crate::kind_map::{
    ApiOnlyMapping, CfnMapping, OperationMapping, RelationMapping, RelationPath, RelationSource,
    RelationTarget,
};

pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::IoTSiteWise::Project"),
    CfnMapping::canonical("AWS::IoTSiteWise::Dashboard"),
    CfnMapping::canonical("AWS::IoTSiteWise::AccessPolicy"),
    CfnMapping::canonical("AWS::IoTSiteWise::Workspace"),
    CfnMapping::canonical("AWS::IoTSiteWise::Application"),
    CfnMapping::canonical("AWS::IoTSiteWise::Dataset"),
    CfnMapping::canonical("AWS::IoTSiteWise::Pipeline"),
    CfnMapping::canonical("AWS::IoTSiteWise::Task"),
    CfnMapping::canonical("AWS::IoTTwinMaker::Scene"),
    CfnMapping::canonical("AWS::IoTTwinMaker::SyncJob"),
];

pub(super) const API: &[ApiOnlyMapping<'static>] = &[ApiOnlyMapping {
    kind: "AWS::ApiOnly::IoTSiteWiseTimeSeries",
    reason: "Pinned CFN registry has no time-series definition type. ListTimeSeries enumerates metadata by its own timeSeriesArn even when no asset/property is attached; it never reads sample values.",
}];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "iotsitewise",
        operation: "ListProjects",
        kind: "AWS::IoTSiteWise::Project",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListDashboards",
        kind: "AWS::IoTSiteWise::Dashboard",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListAccessPolicies",
        kind: "AWS::IoTSiteWise::AccessPolicy",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListWorkspaces",
        kind: "AWS::IoTSiteWise::Workspace",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListApplications",
        kind: "AWS::IoTSiteWise::Application",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListDatasets",
        kind: "AWS::IoTSiteWise::Dataset",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListPipelines",
        kind: "AWS::IoTSiteWise::Pipeline",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListTasks",
        kind: "AWS::IoTSiteWise::Task",
    },
    OperationMapping {
        service: "iotsitewise",
        operation: "ListTimeSeries",
        kind: "AWS::ApiOnly::IoTSiteWiseTimeSeries",
    },
    OperationMapping {
        service: "iottwinmaker",
        operation: "ListScenes",
        kind: "AWS::IoTTwinMaker::Scene",
    },
    OperationMapping {
        service: "iottwinmaker",
        operation: "ListSyncJobs",
        kind: "AWS::IoTTwinMaker::SyncJob",
    },
];

pub(super) const RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::Application",
        source: RelationSource::SelectedOutput(RelationPath::Fields("workspaceName")),
        target_kind: "AWS::IoTSiteWise::Workspace",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("name")),
        relation: "IN_WORKSPACE",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::IoTSiteWiseTimeSeries",
        source: RelationSource::SelectedOutput(RelationPath::Fields("assetId")),
        target_kind: "AWS::IoTSiteWise::Asset",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
        relation: "IN_ASSET",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::AccessPolicy",
        source: RelationSource::SelectedOutput(RelationPath::Fields("resource.portal.id")),
        target_kind: "AWS::IoTSiteWise::Portal",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
        relation: "IN_PORTAL",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::AccessPolicy",
        source: RelationSource::SelectedOutput(RelationPath::Fields("resource.project.id")),
        target_kind: "AWS::IoTSiteWise::Project",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
        relation: "IN_PROJECT",
        scope: "account_region",
        guard: None,
    },
];

pub(super) const REQUEST_RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::Project",
        source: RelationSource::Request(RelationPath::Fields("portalId")),
        target_kind: "AWS::IoTSiteWise::Portal",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
        relation: "IN_PORTAL",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::Dashboard",
        source: RelationSource::Request(RelationPath::Fields("projectId")),
        target_kind: "AWS::IoTSiteWise::Project",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
        relation: "IN_PROJECT",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::Pipeline",
        source: RelationSource::Request(RelationPath::Fields("workspaceName")),
        target_kind: "AWS::IoTSiteWise::Workspace",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("name")),
        relation: "IN_WORKSPACE",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTSiteWise::Task",
        source: RelationSource::Request(RelationPath::Fields("workspaceName")),
        target_kind: "AWS::IoTSiteWise::Workspace",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("name")),
        relation: "IN_WORKSPACE",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTTwinMaker::Scene",
        source: RelationSource::Request(RelationPath::Fields("workspaceId")),
        target_kind: "AWS::IoTTwinMaker::Workspace",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("workspaceId")),
        relation: "IN_WORKSPACE",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::IoTTwinMaker::SyncJob",
        source: RelationSource::Request(RelationPath::Fields("workspaceId")),
        target_kind: "AWS::IoTTwinMaker::Workspace",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("workspaceId")),
        relation: "IN_WORKSPACE",
        scope: "account_region",
        guard: None,
    },
];
