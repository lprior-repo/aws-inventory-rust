use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "appstream", operation: "ListExportImageTasks", kind: "AWS::ApiOnly::AppStreamExportImageTask" },
    OperationMapping { service: "appsync", operation: "GetApi", kind: "AWS::AppSync::Api" },
    OperationMapping { service: "appsync", operation: "GetDomainName", kind: "AWS::AppSync::DomainName" },
    OperationMapping { service: "appsync", operation: "GetGraphqlApi", kind: "AWS::AppSync::GraphQLApi" },
    OperationMapping { service: "appsync", operation: "ListApis", kind: "AWS::AppSync::Api" },
    OperationMapping { service: "appsync", operation: "ListChannelNamespaces", kind: "AWS::AppSync::ChannelNamespace" },
    OperationMapping { service: "appsync", operation: "ListDomainNames", kind: "AWS::AppSync::DomainName" },
    OperationMapping { service: "appsync", operation: "ListGraphqlApis", kind: "AWS::AppSync::GraphQLApi" },
    OperationMapping { service: "arc-region-switch", operation: "ListPlans", kind: "AWS::ARCRegionSwitch::Plan" },
    OperationMapping { service: "arc-zonal-shift", operation: "ListManagedResources", kind: "AWS::ApiOnly::ARCZonalShiftManagedResource" },
    OperationMapping { service: "arc-zonal-shift", operation: "ListZonalShifts", kind: "AWS::ApiOnly::ARCZonalShiftZonalShift" },
    OperationMapping { service: "athena", operation: "ListCapacityReservations", kind: "AWS::Athena::CapacityReservation" },
    OperationMapping { service: "athena", operation: "ListDataCatalogs", kind: "AWS::Athena::DataCatalog" },
    OperationMapping { service: "athena", operation: "ListWorkGroups", kind: "AWS::Athena::WorkGroup" },
    OperationMapping { service: "auditmanager", operation: "ListAssessmentReports", kind: "AWS::ApiOnly::AuditManagerAssessmentReport" },
    OperationMapping { service: "auditmanager", operation: "ListAssessments", kind: "AWS::AuditManager::Assessment" },
    OperationMapping { service: "auto-scaling-plans", operation: "DescribeScalingPlans", kind: "AWS::AutoScalingPlans::ScalingPlan" },
    OperationMapping { service: "auto-scaling", operation: "DescribeAutoScalingGroups", kind: "AWS::AutoScaling::AutoScalingGroup" },
    OperationMapping { service: "auto-scaling", operation: "DescribePolicies", kind: "AWS::AutoScaling::ScalingPolicy" },
    OperationMapping { service: "auto-scaling", operation: "DescribeScheduledActions", kind: "AWS::AutoScaling::ScheduledAction" },
];
