use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "mgn",
        operation: "DescribeSourceServers",
        kind: "AWS::ApiOnly::MigrationSourceServer",
    },
    OperationMapping {
        service: "mgn",
        operation: "DescribeReplicationConfigurationTemplates",
        kind: "AWS::MGN::ReplicationConfigurationTemplate",
    },
    OperationMapping {
        service: "mgn",
        operation: "DescribeLaunchConfigurationTemplates",
        kind: "AWS::MGN::LaunchConfigurationTemplate",
    },
    OperationMapping {
        service: "mgn",
        operation: "DescribeVcenterClients",
        kind: "AWS::ApiOnly::MigrationVcenterClient",
    },
    OperationMapping {
        service: "mgn",
        operation: "ListApplications",
        kind: "AWS::MGN::Application",
    },
    OperationMapping {
        service: "mgn",
        operation: "ListConnectors",
        kind: "AWS::MGN::Connector",
    },
    OperationMapping {
        service: "mgn",
        operation: "ListWaves",
        kind: "AWS::MGN::Wave",
    },
    OperationMapping {
        service: "migration-hub-refactor-spaces",
        operation: "ListEnvironments",
        kind: "AWS::RefactorSpaces::Environment",
    },
    OperationMapping {
        service: "migration-hub-refactor-spaces",
        operation: "ListApplications",
        kind: "AWS::RefactorSpaces::Application",
    },
    OperationMapping {
        service: "migration-hub-refactor-spaces",
        operation: "ListServices",
        kind: "AWS::RefactorSpaces::Service",
    },
    OperationMapping {
        service: "migration-hub",
        operation: "ListProgressUpdateStreams",
        kind: "AWS::ApiOnly::MigrationProgressUpdateStream",
    },
    OperationMapping {
        service: "migration-hub",
        operation: "ListApplicationStates",
        kind: "AWS::ApiOnly::MigrationApplicationState",
    },
    OperationMapping {
        service: "migrationhub-config",
        operation: "DescribeHomeRegionControls",
        kind: "AWS::ApiOnly::MigrationHomeRegionControl",
    },
    OperationMapping {
        service: "migrationhuborchestrator",
        operation: "ListWorkflows",
        kind: "AWS::ApiOnly::MigrationOrchestratorWorkflow",
    },
    OperationMapping {
        service: "migrationhubstrategy",
        operation: "ListCollectors",
        kind: "AWS::ApiOnly::MigrationStrategyCollector",
    },
    OperationMapping {
        service: "mpa",
        operation: "ListApprovalTeams",
        kind: "AWS::MPA::ApprovalTeam",
    },
    OperationMapping {
        service: "mpa",
        operation: "ListIdentitySources",
        kind: "AWS::MPA::IdentitySource",
    },
    OperationMapping {
        service: "mq",
        operation: "ListBrokers",
        kind: "AWS::AmazonMQ::Broker",
    },
    OperationMapping {
        service: "mq",
        operation: "ListConfigurations",
        kind: "AWS::AmazonMQ::Configuration",
    },
    OperationMapping {
        service: "mq",
        operation: "DescribeBroker",
        kind: "AWS::AmazonMQ::Broker",
    },
];
