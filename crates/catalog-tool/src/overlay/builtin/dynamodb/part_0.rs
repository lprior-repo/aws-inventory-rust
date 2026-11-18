use super::super::{empty, name, path};
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListBackups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("BackupSummaries")?),
        fixed: [(path("BackupType")?, FixedValue::String("ALL".to_owned())), (path("Limit")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("ExclusiveStartBackupArn")?, output: path("LastEvaluatedBackupArn")?, optional: false })),
        identity: Some(Identity::Arn(path("BackupArn")?)),
        ..empty()
    }),
("DescribeBackup", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("BackupDescription.BackupDetails")?),
        bind: [(path("BackupArn")?, Binding::From { operation: name("ListBackups")?, path: path("BackupArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Arn(path("BackupArn")?)),
        ..empty()
    }),
("ListBulkOperations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("BulkOperationsSummaryList")?),
        page_size: Some(25),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        identity: Some(Identity::Arn(path("BulkOperationArn")?)),
        ..empty()
    }),
("DescribeBulkOperation", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("BulkOperationDescription")?),
        bind: [(path("BulkOperationArn")?, Binding::From { operation: name("ListBulkOperations")?, path: path("BulkOperationArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Arn(path("BulkOperationArn")?)),
        ..empty()
    }),
("ListExports", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ExportSummaries")?),
        page_size: Some(25),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        identity: Some(Identity::Arn(path("ExportArn")?)),
        ..empty()
    }),
("ListGlobalTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("GlobalTables")?),
        fixed: [(path("Limit")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("ExclusiveStartGlobalTableName")?, output: path("LastEvaluatedGlobalTableName")?, optional: false })),
        identity: Some(Identity::Key(vec![path("GlobalTableName")?])),
        ..empty()
    }),
("DescribeGlobalTable", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("GlobalTableDescription")?),
        bind: [(path("GlobalTableName")?, Binding::From { operation: name("ListGlobalTables")?, path: path("GlobalTableName")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Key(vec![path("GlobalTableName")?])),
        ..empty()
    }),
("DescribeGlobalTableSettings", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        bind: [(path("GlobalTableName")?, Binding::From { operation: name("ListGlobalTables")?, path: path("GlobalTableName")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Key(vec![path("GlobalTableName")?])),
        ..empty()
    }),
("ListImports", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ImportSummaryList")?),
        page_size: Some(25),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        identity: Some(Identity::Arn(path("ImportArn")?)),
        ..empty()
    }),
("DescribeImport", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ImportTableDescription")?),
        bind: [(path("ImportArn")?, Binding::From { operation: name("ListImports")?, path: path("ImportArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Arn(path("ImportArn")?)),
        ..empty()
    }),
("ListTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TableNames")?),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("ExclusiveStartTableName")?, output: path("LastEvaluatedTableName")?, optional: false })),
        identity: Some(Identity::Template("arn:{partition}:dynamodb:{region}:{account}:table/{$}".to_owned())),
        ..empty()
    }),
("DescribeTable", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Table")?),
        bind: [(path("TableName")?, Binding::From { operation: name("ListTables")?, path: path("$")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Arn(path("TableArn")?)),
        ..empty()
    }),
("DescribeContinuousBackups", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Reads PITR status/window for required TableName, now discoverable via ListTables. Missing request/parent-bound identity: ContinuousBackupsDescription does not echo a table identifier. Do not identify by mutable status or recovery timestamps.".to_owned()),
        ..empty()
    }),
("DescribeTimeToLive", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Reads TTL status/attribute for required TableName, now discoverable via ListTables. Missing request/parent-bound identity: TimeToLiveDescription has no table identifier. AttributeName and status are not unique resource identities.".to_owned()),
        ..empty()
    }),
("DescribeKinesisStreamingDestination", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        bind: [(path("TableName")?, Binding::From { operation: name("ListTables")?, path: path("$")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Key(vec![path("TableName")?])),
        ..empty()
    }),
("DescribeTableReplicaAutoScaling", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        bind: [(path("TableName")?, Binding::From { operation: name("ListTables")?, path: path("$")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        identity: Some(Identity::Key(vec![path("TableAutoScalingDescription.TableName")?])),
        ..empty()
    }),
("ListContributorInsights", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Paged safe configuration summaries use NextToken and MaxResults<=100; TableName is optional. Each summary has TableName and optional IndexName (absent for the table). Needs optional-safe composite identity to distinguish table from same-table indexes without missing-identity failure or collisions. TableName-only identity loses index records.".to_owned()),
        ..empty()
    }),
("DescribeContributorInsights", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Reads table/index insight configuration and rule names, not insight event/key contents. Requires TableName plus optional IndexName from summaries. Needs correlated optional binding/identity for table versus index records; ListContributorInsights is deferred. Omitting IndexName would silently miss indexes.".to_owned()),
        ..empty()
    }),
("ListTagsOfResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Safe tag read requires ResourceArn, now discoverable from DescribeTable.Table.TableArn. Tags and NextToken output do not echo resource ARN. Needs parent-bound resource identity and paginated tag-enrichment merge (NextToken in/out); identifying by tag Key would collide across tables.".to_owned()),
        ..empty()
    }),
("GetResourcePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Policy is access-control configuration, not table item data; table/stream ARNs are discoverable from DescribeTable. Output Policy/RevisionId has no resource identity. Needs request-bound identity, optional absent-policy handling and reviewed policy presence/projection without runtime JSON parsing. No policy-body collection is approved.".to_owned()),
        ..empty()
    })
    ])
}
