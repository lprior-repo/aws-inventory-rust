use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "cloudtrail",
        operation: "ListEventDataStores",
        kind: "AWS::CloudTrail::EventDataStore",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "ListImports",
        kind: "AWS::ApiOnly::CloudTrailImport",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "DescribeApiDestination",
        kind: "AWS::Events::ApiDestination",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "DescribeArchive",
        kind: "AWS::Events::Archive",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "DescribeEventBus",
        kind: "AWS::Events::EventBus",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListApiDestinations",
        kind: "AWS::Events::ApiDestination",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListArchives",
        kind: "AWS::Events::Archive",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListConnections",
        kind: "AWS::Events::Connection",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListEventBuses",
        kind: "AWS::Events::EventBus",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListEventSources",
        kind: "AWS::ApiOnly::EventsEventSource",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListReplays",
        kind: "AWS::ApiOnly::EventsReplay",
    },
    OperationMapping {
        service: "cloudwatch-events",
        operation: "ListRules",
        kind: "AWS::Events::Rule",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeDeliveries",
        kind: "AWS::Logs::Delivery",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeDeliveryDestinations",
        kind: "AWS::Logs::DeliveryDestination",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeDeliverySources",
        kind: "AWS::Logs::DeliverySource",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeDestinations",
        kind: "AWS::Logs::Destination",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeExportTasks",
        kind: "AWS::ApiOnly::LogsExportTask",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeImportTasks",
        kind: "AWS::ApiOnly::LogsImportTask",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeLogGroups",
        kind: "AWS::Logs::LogGroup",
    },
    OperationMapping {
        service: "cloudwatch-logs",
        operation: "DescribeLogStreams",
        kind: "AWS::Logs::LogStream",
    },
];
