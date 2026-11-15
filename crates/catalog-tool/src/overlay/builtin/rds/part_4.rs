use crate::overlay::{NameError, OperationOverlay, ExecutionBranch, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeDBRecommendations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Diagnostic performance Analysis and opaque RecommendedActions.Parameters.Value/ContextAttributes.Value have no metadata-only safety guarantee in the model. Needs reviewed projection or authoritative exclusion of customer SQL/secret content before enabling this diagnostic read.".to_owned()),
        ..empty()
    }),
("DescribeDBSecurityGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBSecurityGroups")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBSecurityGroupArn")?)),
        ..empty()
    }),
("DescribeDBShardGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBShardGroups")?),
        tags: Some(path("TagList")?),
        fixed: [(path("MaxRecords")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBShardGroupArn")?)),
        ..empty()
    }),
("DescribeDBSnapshotAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires a manual-only snapshot source and merge of DBSnapshotAttributesResult onto the snapshot identity. DescribeDBSnapshots needs request variants for complete owned snapshots, and the schema cannot filter parent SnapshotType=manual.".to_owned()),
        ..empty()
    }),
("DescribeDBSnapshotTenantDatabases", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("SnapshotType supports automated, manual and awsbackup owned snapshots, but the model does not document the omitted-type coverage. Needs explicit owned-type request variants and deduplication; shared/public reference snapshots must not be enabled to approximate complete inventory.".to_owned()),
        ..empty()
    }),
("DescribeDBSnapshots", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBSnapshots")?),
        identity: Some(Identity::Arn(path("DBSnapshotArn")?)),
        tags: Some(path("TagList")?),
        queryable: vec![path("DBSnapshotArn")?],
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        branches: ["manual", "automated", "awsbackup"].into_iter().map(|kind| Ok(ExecutionBranch {
            discriminator: kind.to_owned(),
            fixed: [(path("SnapshotType")?, FixedValue::String(kind.to_owned()))].into(),
            scope: Scope::Region, request_region: None,
            bind: Default::default(), emission: None, row_policy: None,
        })).collect::<Result<_, NameError>>()?,
        ..empty()
    }),
("DescribeDBSubnetGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBSubnetGroups")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBSubnetGroupArn")?)),
        ..empty()
    }),
("DescribeEngineDefaultClusterParameters", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeEngineDefaultParameters", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeEventCategories", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeEventSubscriptions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("EventSubscriptionsList")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("EventSubscriptionArn")?)),
        ..empty()
    }),
("DescribeEvents", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeExportTasks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ExportTasks")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Key(vec![path("ExportTaskIdentifier")?])),
        ..empty()
    }),
("DescribeGlobalClusters", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("GlobalClusters")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("GlobalClusterArn")?)),
        ..empty()
    }),
("DescribeIntegrations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Integrations")?),
        page_size: Some(100),
        tags: Some(path("Tags")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("IntegrationArn")?)),
        ..empty()
    }),
("DescribeOptionGroupOptions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeOptionGroups", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeOrderableDBInstanceOptions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribePendingMaintenanceActions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PendingMaintenanceActions")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("ResourceIdentifier")?)),
        ..empty()
    }),
("DescribeReservedDBInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ReservedDBInstances")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("ReservedDBInstanceArn")?)),
        ..empty()
    })
    ])
}
