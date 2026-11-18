use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeAnomalyDetectors", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AnomalyDetectors")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("StateValue")?],
        queryable: vec![path("AnomalyDetectorId")?, path("Namespace")?, path("MetricName")?, path("Stat")?],
        fixed: [(path("AnomalyDetectorTypes")?, FixedValue::Strings(vec!["SINGLE_METRIC".to_owned(), "METRIC_MATH".to_owned()]))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("AnomalyDetectorId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInsightRules", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InsightRules")?),
        page_size: Some(500),
        queryable: vec![path("Name")?, path("State")?, path("ManagedRule")?, path("ApplyOnTransformedLogs")?],
        identity: Some(Identity::Template("arn:{partition}:cloudwatch:{region}:{account}:insight-rule/{Name}".to_owned())),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListAlarmMuteRules", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AlarmMuteRuleSummaries")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("Status")?, path("LastUpdatedTimestamp")?],
        queryable: vec![path("AlarmMuteRuleArn")?, path("MuteType")?],
        identity: Some(Identity::Arn(path("AlarmMuteRuleArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListDashboards", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DashboardEntries")?),
        exclude_from_hash: vec![path("LastModified")?],
        queryable: vec![path("DashboardName")?, path("DashboardArn")?, path("Size")?],
        identity: Some(Identity::Arn(path("DashboardArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetDashboard", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("$")?),
        queryable: vec![path("DashboardName")?, path("DashboardArn")?],
        identity: Some(Identity::Arn(path("DashboardArn")?)),
        bind: [(path("DashboardName")?, Binding::From { operation: name("ListDashboards")?, path: path("DashboardName")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetDataset", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        queryable: vec![path("DatasetId")?, path("Arn")?, path("KmsKeyArn")?],
        fixed: [(path("DatasetIdentifier")?, FixedValue::String("default".to_owned()))].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetOTelEnrichment", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        exclude_from_hash: vec![path("CreatedAt")?, path("UpdatedAt")?],
        queryable: vec![path("Status")?],
        identity: Some(Identity::Template("cloudwatch:otel-enrichment:{partition}:{account}:{region}".to_owned())),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListMetricDataQueryDefinitions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("UpdatedAt")?],
        queryable: vec![path("Id")?, path("Path")?, path("Type")?],
        identity: Some(Identity::Template("arn:{partition}:cloudwatch:{region}:{account}:metric-query-definition/{Id}".to_owned())),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetMetricDataQueryDefinition", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListMetricStreams", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Entries")?),
        page_size: Some(500),
        exclude_from_hash: vec![path("CreationDate")?, path("LastUpdateDate")?],
        queryable: vec![path("Arn")?, path("Name")?, path("FirehoseArn")?, path("State")?, path("OutputFormat")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetMetricStream", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        exclude_from_hash: vec![path("CreationDate")?, path("LastUpdateDate")?],
        queryable: vec![path("Arn")?, path("Name")?, path("FirehoseArn")?, path("RoleArn")?, path("State")?, path("OutputFormat")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        bind: [(path("Name")?, Binding::From { operation: name("ListMetricStreams")?, path: path("Name")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeAlarms", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("The response contains MetricAlarms, CompositeAlarms and LogAlarms; one items selector cannot emit all three, and complete requests must explicitly include all three AlarmTypes. StateReason and StateReasonData also need an evaluated-value safety decision or safe projection; exclude_from_hash does not remove content.".to_owned()),
        ..empty()
    }),
("DescribeAlarmsForMetric", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires MetricName and Namespace with the complete Dimensions identity from ListMetrics, whose collection-valued identity is not expressible. Returns only standard single-metric alarms, not metric-math or composite alarms, and MetricAlarms.StateReason/StateReasonData require the same evaluated-value safety decision as DescribeAlarms.".to_owned()),
        ..empty()
    }),
("GetAlarmMuteRule", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires AlarmMuteRuleName, but ListAlarmMuteRules.AlarmMuteRuleSummaries supplies AlarmMuteRuleArn and no name. Binding the complete ARN as a name is not documented; a reviewed ARN-resource extraction binding is missing.".to_owned()),
        ..empty()
    }),
("GetResourceMetricsConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires the external resource ResourceArn, but CloudWatch exposes no resource-metrics-configuration list operation. Needs reviewed cross-service resource discovery and bindings, including ResourceNotFoundException absence semantics; current bindings name same-service operation items only.".to_owned()),
        ..empty()
    }),
("ListMetrics", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns metric descriptors, not values, but unique identity requires Namespace, MetricName and the entire unordered Dimensions list. Scalar identity paths cannot encode that identity without collisions. Linked-account results also require joining Metrics with the parallel OwningAccounts array; the default same-account listing still needs canonical dimension identity and omits metrics inactive for two weeks.".to_owned()),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags have no resource ARN in the response. Correct ownership requires request/parent identity inheritance and fanout across alarm, dashboard, metric-stream and Contributor Insights rule parents; the current single-source binding and output-only identity cannot express that complete plan.".to_owned()),
        ..empty()
    }),
("DescribeAlarmContributors", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeAlarmHistory", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetInsightRuleReport", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    })
    ])
}
