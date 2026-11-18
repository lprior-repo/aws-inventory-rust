//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iotsitewise/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{
    Binding, ContextSource, ExecutionBranch, FixedValue, Identity, IdentityField, MemberPath,
    MissingSelected, NameError, OperationName, OperationOverlay, OverlayClass, RowOperand,
    RowPolicy, RowPredicate, RowValueType, Scope, ServiceOverlay,
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListAssetModels",
            policy::query(
                policy::enumerate(
                    "assetModelSummaries",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn", "id"],
            )?,
        ),
        (
            "ListAssets",
            policy::query(
                policy::fixed(
                    policy::bind(
                        policy::enumerate(
                            "assetSummaries",
                            policy::arn("arn")?,
                            Some(("nextToken", "nextToken")),
                        )?,
                        "assetModelId",
                        "ListAssetModels",
                        "id",
                    )?,
                    "filter",
                    "ALL",
                )?,
                &["arn", "assetModelId"],
            )?,
        ),
        (
            "ListGateways",
            policy::query(
                policy::enumerate(
                    "gatewaySummaries",
                    policy::key("gatewayId")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["gatewayId"],
            )?,
        ),
        (
            "ListPortals",
            policy::query(
                policy::enumerate(
                    "portalSummaries",
                    policy::key("id")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["id"],
            )?,
        ),
        (
            "ListComputationModels",
            policy::query(
                policy::enumerate(
                    "computationModelSummaries",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListProjects",
            monitor_child("projectSummaries", "portalId", "ListPortals")?,
        ),
        (
            "ListDashboards",
            monitor_child("dashboardSummaries", "projectId", "ListProjects")?,
        ),
        ("ListAccessPolicies", access_policies()?),
        (
            "ListWorkspaces",
            policy::query(
                policy::enumerate(
                    "workspaceSummaries",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn", "name"],
            )?,
        ),
        (
            "ListApplications",
            policy::query(
                policy::enumerate(
                    "applications",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn", "id", "workspaceName"],
            )?,
        ),
        ("ListDatasets", datasets()?),
        (
            "ListPipelines",
            workspace_child("pipelineSummaries", "pipelineArn")?,
        ),
        ("ListTasks", workspace_child("taskSummaries", "taskArn")?),
        (
            "ListTimeSeries",
            policy::query(
                policy::enumerate(
                    "TimeSeriesSummaries",
                    policy::arn("timeSeriesArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["timeSeriesArn", "timeSeriesId", "assetId", "propertyId"],
            )?,
        ),
    ];
    policy::finish(
        "iotsitewise",
        approved,
        DEFERRED,
        "Pinned complete summary shapes are metadata, including nested status/error/enrichment metadata and policy identity references, not definitions, samples or credentials. Monitor projects/dashboards retain their bound parent in identity; workspace pipeline/task definitions use their own ARN, never execution identity. Dataset branches cover both KENDRA and SITEWISE without optional narrowing filters. Access-policy PORTAL and PROJECT branches bind only their corresponding producer, retaining resource type and bound resource identity; native resource IDs must equal the bound request parent and the opposite resource variant must be absent. Remaining detail/content families remain unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

fn monitor_child(items: &str, input: &str, parent: &str) -> Result<OperationOverlay, NameError> {
    policy::query(
        policy::bind(
            policy::enumerate(
                items,
                Identity::CorrelatedKey(vec![
                    IdentityField::Request(MemberPath::new(input)?),
                    IdentityField::Selected(MemberPath::new("id")?),
                ]),
                Some(("nextToken", "nextToken")),
            )?,
            input,
            parent,
            "id",
        )?,
        &["id", "name"],
    )
}

fn workspace_child(items: &str, arn: &str) -> Result<OperationOverlay, NameError> {
    policy::query(
        policy::bind(
            policy::enumerate(items, policy::arn(arn)?, Some(("nextToken", "nextToken")))?,
            "workspaceName",
            "ListWorkspaces",
            "name",
        )?,
        &[arn],
    )
}

fn access_policies() -> Result<OperationOverlay, NameError> {
    let mut value = policy::query(
        policy::enumerate(
            "accessPolicySummaries",
            Identity::CorrelatedKey(vec![
                IdentityField::Request(MemberPath::new("resourceType")?),
                IdentityField::Request(MemberPath::new("resourceId")?),
                IdentityField::Selected(MemberPath::new("id")?),
            ]),
            Some(("nextToken", "nextToken")),
        )?,
        &[
            "id",
            "permission",
            "resource.portal.id",
            "resource.project.id",
            "identity.user.id",
            "identity.group.id",
            "identity.iamUser.arn",
            "identity.iamRole.arn",
        ],
    )?;
    value.class = Some(OverlayClass::Expand);
    value.branches = [
        (
            "PORTAL",
            "ListPortals",
            "resource.portal.id",
            "resource.project",
        ),
        (
            "PROJECT",
            "ListProjects",
            "resource.project.id",
            "resource.portal",
        ),
    ]
    .into_iter()
    .map(|(resource, producer, selected, opposite)| {
        Ok(ExecutionBranch {
            discriminator: resource.to_owned(),
            fixed: [(
                MemberPath::new("resourceType")?,
                FixedValue::String(resource.to_owned()),
            )]
            .into_iter()
            .collect(),
            bind: [(
                MemberPath::new("resourceId")?,
                Binding::From {
                    operation: OperationName::new(producer)?,
                    path: MemberPath::new("id")?,
                },
            )]
            .into_iter()
            .collect(),
            emission: None,
            row_policy: Some(RowPolicy {
                assertions: vec![
                    RowPredicate::SelectedRequestEquals {
                        selected: MemberPath::selected(selected)?,
                        request: MemberPath::selected("resourceId")?,
                        value_type: RowValueType::String,
                        missing: MissingSelected::Reject,
                    },
                    RowPredicate::Absent(RowOperand {
                        source: ContextSource::SelectedItem,
                        path: MemberPath::selected(opposite)?,
                        value_type: RowValueType::Structure,
                    }),
                ],
                ..Default::default()
            }),
            scope: Scope::Region,
            request_region: None,
        })
    })
    .collect::<Result<_, NameError>>()?;
    Ok(value)
}

fn datasets() -> Result<OperationOverlay, NameError> {
    let mut value = policy::query(
        policy::enumerate(
            "datasetSummaries",
            policy::arn("arn")?,
            Some(("nextToken", "nextToken")),
        )?,
        &["arn", "id", "sourceType"],
    )?;
    value.branches = ["KENDRA", "SITEWISE"]
        .into_iter()
        .map(|source| {
            Ok(ExecutionBranch {
                discriminator: source.to_owned(),
                fixed: [(
                    MemberPath::new("sourceType")?,
                    FixedValue::String(source.to_owned()),
                )]
                .into_iter()
                .collect(),
                bind: Default::default(),
                emission: None,
                row_policy: None,
                scope: Scope::Region,
                request_region: None,
            })
        })
        .collect::<Result<_, NameError>>()?;
    Ok(value)
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeAccessPolicy",
    "DescribeAction",
    "DescribeApplication",
    "DescribeAsset",
    "DescribeAssetCompositeModel",
    "DescribeAssetModel",
    "DescribeAssetModelCompositeModel",
    "DescribeAssetModelInterfaceRelationship",
    "DescribeAssetProperty",
    "DescribeBulkImportJob",
    "DescribeComputationModel",
    "DescribeComputationModelExecutionSummary",
    "DescribeDashboard",
    "DescribeDataset",
    "DescribeDatasetExportJob",
    "DescribeDefaultEncryptionConfiguration",
    "DescribeEnrichmentJob",
    "DescribeExecution",
    "DescribeGateway",
    "DescribeGatewayCapabilityConfiguration",
    "DescribeLoggingOptions",
    "DescribePipeline",
    "DescribePipelineExecution",
    "DescribePortal",
    "DescribeProject",
    "DescribeQuery",
    "DescribeSearch",
    "DescribeStorageConfiguration",
    "DescribeTask",
    "DescribeTimeSeries",
    "DescribeWorkspace",
    "GetCaptureData",
    "GetSearchResults",
    "ListActions",
    "ListAssetModelCompositeModels",
    "ListAssetModelProperties",
    "ListAssetProperties",
    "ListAssetRelationships",
    "ListAssociatedAssets",
    "ListBulkImportJobs",
    "ListCompositionRelationships",
    "ListComputationModelDataBindingUsages",
    "ListComputationModelResolveToResources",
    "ListDatasetDataSegmentRelationships",
    "ListDatasetDataSegments",
    "ListDatasetExportJobs",
    "ListEnrichmentJobs",
    "ListExecutions",
    "ListInterfaceRelationships",
    "ListPipelineExecutions",
    "ListProjectAssets",
    "ListQueries",
    "ListSearches",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AssociateAssets",
    "AssociateTimeSeriesToAssetProperty",
    "BatchAssociateDataSegmentsToDataset",
    "BatchAssociateProjectAssets",
    "BatchDeleteDatasetDataSegments",
    "BatchDisassociateDataSegmentsFromDataset",
    "BatchDisassociateProjectAssets",
    "BatchPutAssetPropertyValue",
    "CancelEnrichmentJob",
    "CancelPipelineExecution",
    "CancelQuery",
    "CreateAccessPolicy",
    "CreateApplication",
    "CreateAsset",
    "CreateAssetModel",
    "CreateAssetModelCompositeModel",
    "CreateBulkImportJob",
    "CreateComputationModel",
    "CreateDashboard",
    "CreateDataset",
    "CreateDatasetExportJob",
    "CreateEnrichmentJob",
    "CreateGateway",
    "CreatePipeline",
    "CreatePortal",
    "CreateProject",
    "CreateTask",
    "CreateWorkspace",
    "DeleteAccessPolicy",
    "DeleteApplication",
    "DeleteAsset",
    "DeleteAssetModel",
    "DeleteAssetModelCompositeModel",
    "DeleteAssetModelInterfaceRelationship",
    "DeleteComputationModel",
    "DeleteDashboard",
    "DeleteDataset",
    "DeleteGateway",
    "DeletePipeline",
    "DeletePortal",
    "DeleteProject",
    "DeleteTask",
    "DeleteTimeSeries",
    "DeleteWorkspace",
    "DisassociateAssets",
    "DisassociateTimeSeriesFromAssetProperty",
    "ExecuteAction",
    "PutAssetModelInterfaceRelationship",
    "PutDefaultEncryptionConfiguration",
    "PutLoggingOptions",
    "PutStorageConfiguration",
    "StartPipelineExecution",
    "StartQuery",
    "StartSearch",
    "TagResource",
    "UntagResource",
    "UpdateAccessPolicy",
    "UpdateAsset",
    "UpdateAssetModel",
    "UpdateAssetModelCompositeModel",
    "UpdateAssetProperty",
    "UpdateComputationModel",
    "UpdateDashboard",
    "UpdateDataset",
    "UpdateGateway",
    "UpdateGatewayCapabilityConfiguration",
    "UpdatePipeline",
    "UpdatePortal",
    "UpdateProject",
    "UpdateTask",
    "UpdateWorkspace",
];

const CONTENT: &[&str] = &[
    "BatchGetAssetPropertyAggregates",
    "BatchGetAssetPropertyValue",
    "BatchGetAssetPropertyValueHistory",
    "ExecuteQuery",
    "GetAssetPropertyAggregates",
    "GetAssetPropertyValue",
    "GetAssetPropertyValueHistory",
    "GetInterpolatedAssetPropertyValues",
    "GetQueryResults",
    "InvokeAssistant",
];
