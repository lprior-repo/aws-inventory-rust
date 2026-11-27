use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    ContextSource, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, RowOperand, RowPolicy, RowPredicate, RowValueType, Scope, TokenPair,
};

fn owned_rds_engine() -> Result<RowPolicy, NameError> {
    Ok(RowPolicy {
        owned: Some(RowPredicate::StringNotIn {
            operand: RowOperand {
                source: ContextSource::SelectedItem,
                path: path("Engine")?,
                value_type: RowValueType::String,
            },
            values: vec!["docdb".to_owned(), "neptune".to_owned()],
        }),
        ..RowPolicy::default()
    })
}
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeDBCapacityReservations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBCapacityReservations")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBCapacityReservationArn")?)),
        ..empty()
    }),
("DescribeDBClusterAutomatedBackups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBClusterAutomatedBackups")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBClusterAutomatedBackupsArn")?)),
        ..empty()
    }),
("DescribeDBClusterBacktracks", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires an Aurora-MySQL-capable parent predicate; the API applies only to Aurora MySQL clusters. Backtrack identity must include DBClusterIdentifier and BacktrackIdentifier.".to_owned()),
        ..empty()
    }),
("DescribeDBClusterEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBClusterEndpoints")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBClusterEndpointArn")?)),
        ..empty()
    }),
("DescribeDBClusterParameterGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBClusterParameterGroups")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBClusterParameterGroupArn")?)),
        ..empty()
    }),
("DescribeDBClusterParameters", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeDBClusterSnapshotAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("The API documents attributes for manual cluster snapshots only. DescribeDBClusterSnapshots also returns automated snapshots; the schema has no parent SnapshotType=manual predicate or merge of DBClusterSnapshotAttributesResult onto the snapshot identity.".to_owned()),
        ..empty()
    }),
("DescribeDBClusterSnapshots", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBClusterSnapshots")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        fixed: [(path("IncludeShared")?, FixedValue::Boolean(false)), (path("IncludePublic")?, FixedValue::Boolean(false))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBClusterSnapshotArn")?)),
        ..empty()
    }),
("DescribeDBClusters", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBClusters")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        row_policy: owned_rds_engine()?,
        identity: Some(Identity::Key(vec![path("DbClusterResourceId")?])),
        exclude_response_members: vec![path("DBClusters.PendingModifiedValues.MasterUserPassword")?],
        ..empty()
    }),
("DescribeDBEngineVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeDBInstanceAutomatedBackups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBInstanceAutomatedBackups")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Key(vec![path("Region")?, path("DbiResourceId")?])),
        ..empty()
    }),
("DescribeDBInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBInstances")?),
        page_size: Some(100),
        tags: Some(path("TagList")?),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        row_policy: owned_rds_engine()?,
        identity: Some(Identity::Key(vec![path("DbiResourceId")?])),
        exclude_response_members: vec![path("DBInstances.PendingModifiedValues.MasterUserPassword")?],
        ..empty()
    }),
("DescribeDBLogFiles", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("The response contains log metadata (LogFileName, LastWritten, Size), not log bytes. Parent-qualified (DBInstanceIdentifier, LogFileName) identity is required; output lacks DBInstanceIdentifier.".to_owned()),
        ..empty()
    }),
("DescribeDBMajorEngineVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeDBParameterGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBParameterGroups")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBParameterGroupArn")?)),
        ..empty()
    }),
("DescribeDBParameters", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeDBProxies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBProxies")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBProxyArn")?)),
        ..empty()
    }),
("DescribeDBProxyEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DBProxyEndpoints")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false })),
        identity: Some(Identity::Arn(path("DBProxyEndpointArn")?)),
        ..empty()
    }),
("DescribeDBProxyTargetGroups", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeDBProxyTargets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Targets describe memberships, not unique standalone DB resources. TargetArn is the DB instance/cluster ARN and can occur under multiple proxies. Needs parent-qualified (DBProxyName, TargetGroupName, TargetArn) identity and membership edges; proxy/group context is absent from output.".to_owned()),
        ..empty()
    })
    ])
}
