use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "data-pipeline", operation: "ListPipelines", kind: "AWS::DataPipeline::Pipeline" },
    OperationMapping { service: "database-migration-service", operation: "DescribeEventSubscriptions", kind: "AWS::DMS::EventSubscription" },
    OperationMapping { service: "database-migration-service", operation: "DescribeInstanceProfiles", kind: "AWS::DMS::InstanceProfile" },
    OperationMapping { service: "database-migration-service", operation: "DescribeMigrationProjects", kind: "AWS::DMS::MigrationProject" },
    OperationMapping { service: "database-migration-service", operation: "DescribeReplicationInstances", kind: "AWS::DMS::ReplicationInstance" },
    OperationMapping { service: "database-migration-service", operation: "DescribeReplicationSubnetGroups", kind: "AWS::DMS::ReplicationSubnetGroup" },
    OperationMapping { service: "databrew", operation: "ListJobs", kind: "AWS::DataBrew::Job" },
    OperationMapping { service: "databrew", operation: "ListProjects", kind: "AWS::DataBrew::Project" },
    OperationMapping { service: "databrew", operation: "ListRulesets", kind: "AWS::DataBrew::Ruleset" },
    OperationMapping { service: "databrew", operation: "ListSchedules", kind: "AWS::DataBrew::Schedule" },
    OperationMapping { service: "dataexchange", operation: "ListDataGrants", kind: "AWS::ApiOnly::DataExchangeDataGrant" },
    OperationMapping { service: "dataexchange", operation: "ListDataSets", kind: "AWS::ApiOnly::DataExchangeDataSet" },
    OperationMapping { service: "dataexchange", operation: "ListReceivedDataGrants", kind: "AWS::ApiOnly::DataExchangeReceivedDataGrant" },
    OperationMapping { service: "datasync", operation: "ListAgents", kind: "AWS::DataSync::Agent" },
    OperationMapping { service: "datasync", operation: "ListLocations", kind: "AWS::ApiOnly::DataSyncLocation" },
    OperationMapping { service: "datasync", operation: "ListTasks", kind: "AWS::DataSync::Task" },
    OperationMapping { service: "datasync", operation: "ListTaskExecutions", kind: "AWS::ApiOnly::DataSyncTaskExecution" },
];
