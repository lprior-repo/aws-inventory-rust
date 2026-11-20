// DMS and migration hub operation mappings.

use crate::kind_map::{ApiOnlyMapping, OperationMapping};

pub(super) const API_ONLY: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSReplication",
        reason: "Provisioned DMS serverless execution status keyed by configuration ARN is distinct from replication configuration, mappings and settings and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSConnection",
        reason: "Existing endpoint and replication-instance connection-test status is not an endpoint definition or secret-bearing connection payload; no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSAssessmentRun",
        reason: "Replication-task assessment-run control summary is distinct from assessment report contents and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSIndividualAssessment",
        reason: "Individual replication-task assessment status is distinct from assessment report contents and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSCollector",
        reason: "Fleet Advisor collector status metadata is not a collector configuration object and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSScannedDatabase",
        reason: "Fleet Advisor scanned-database discovery summary is not a database resource and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSPendingMaintenanceAction",
        reason: "Pending maintenance action status is not a configuration object and has no pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::DMSDataMigration",
        reason: "Historical migration project summary (WithoutSettings=true) is not a project definition and has no pinned CFN type.",
    },
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeDataProviders",
        kind: "AWS::DMS::DataProvider",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeDataMigrations",
        kind: "AWS::ApiOnly::DMSDataMigration",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeReplications",
        kind: "AWS::ApiOnly::DMSReplication",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeConnections",
        kind: "AWS::ApiOnly::DMSConnection",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeReplicationTaskAssessmentRuns",
        kind: "AWS::ApiOnly::DMSAssessmentRun",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeReplicationTaskIndividualAssessments",
        kind: "AWS::ApiOnly::DMSIndividualAssessment",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeFleetAdvisorCollectors",
        kind: "AWS::ApiOnly::DMSCollector",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribeFleetAdvisorDatabases",
        kind: "AWS::ApiOnly::DMSScannedDatabase",
    },
    OperationMapping {
        service: "database-migration-service",
        operation: "DescribePendingMaintenanceActions",
        kind: "AWS::ApiOnly::DMSPendingMaintenanceAction",
    },
];