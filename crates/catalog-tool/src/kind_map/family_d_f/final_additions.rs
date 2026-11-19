use crate::kind_map::{
    ApiOnlyMapping, CfnMapping, MappingSource, OperationMapping, RelationMapping, RelationPath,
    RelationSource, RelationTarget,
};

pub(super) const SOURCE: MappingSource<'static> = MappingSource {
    cfn: &[
        CfnMapping::canonical("AWS::DMS::DataProvider"),
        CfnMapping::canonical("AWS::EMR::StudioSessionMapping"),
        CfnMapping::canonical("AWS::DirectConnect::DirectConnectGatewayAssociation"),
        CfnMapping::canonical("AWS::FMS::ApplicationsList"),
        CfnMapping::canonical("AWS::FMS::ProtocolsList"),
    ],
    api_only: &[
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DataExchangeEventAction",
            reason: "Revision-publication automation event action is distinct from a DataExchange grant and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DirectoryServiceEventTopicSubscription",
            reason: "DirectoryId and TopicName identify a directory event subscription association, not an SNS Topic; no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::ElastiCacheReservedCacheNode",
            reason: "Owned prepaid ElastiCache capacity reservation is not a cache cluster and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::ElasticsearchReservedInstance",
            reason: "Owned prepaid Elasticsearch capacity reservation is not a search domain and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::FraudDetectorBatchImportJob",
            reason: "Existing FraudDetector batch import job control metadata is distinct from imported records and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::FraudDetectorBatchPredictionJob",
            reason: "Existing FraudDetector batch prediction job control metadata is distinct from prediction result contents and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::FSxDataRepositoryTask",
            reason: "Existing FSx import, export or release execution metadata is not a data repository association definition; no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::EMRNotebookExecution",
            reason: "EMR notebook execution summary is distinct from notebook source and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DataExchangeRevision",
            reason: "Published dataset revision metadata is distinct from revision assets and has no pinned CFN type; asset specification download URLs are excluded.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::ECRImage",
            reason: "Private ECR digest summary keyed by registry, repository and digest is distinct from image layers or manifests and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::ECRPublicImage",
            reason: "Public ECR digest summary keyed by registry, repository and digest is distinct from image layers or manifests and has no pinned CFN type.",
        },
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
            kind: "AWS::ApiOnly::DMSCertificate",
            reason: "DMS endpoint encryption certificate metadata is not an endpoint definition or secret payload and has no pinned CFN type.",
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
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DetectiveInvitation",
            reason: "Detective graph invitation access metadata keyed by graph ARN and account is not a graph definition and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DetectiveOrganizationAdministrator",
            reason: "Detective delegated administrator account registration is distinct from a graph or Organizations account and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DirectConnectGatewayAssociationProposal",
            reason: "Global Direct Connect gateway association proposal is not an accepted gateway association and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DirectConnectGatewayAttachment",
            reason: "Direct Connect gateway-to-virtual-interface attachment is distinct from an EC2 transit gateway VPC attachment and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::EndUserMessagingJob",
            reason: "Asynchronous End User Messaging brand-synchronization job summary is distinct from message content and has no pinned CFN type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DSQLStream",
            reason: "DSQL cluster change-stream metadata has its own stream ARN and is distinct from a DSQL cluster; no pinned CFN stream type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::EMRServerlessJobRun",
            reason: "Existing EMR Serverless job-run summary is distinct from an application and excludes executable driver or command content; no pinned CFN job-run type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::EMRServerlessSession",
            reason: "Existing EMR Serverless session summary has its own ARN and is distinct from an application or executable session content; no pinned CFN session type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DetectiveGraphMembership",
            reason: "Graph and account identify Detective membership, including enabled organization accounts; this union is not the invitation-only AWS::Detective::MemberInvitation resource.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DRSRecoveryPlanStep",
            reason: "DRS recovery-plan step has its own ARN and is distinct from the containing recovery plan; no pinned CFN step type.",
        },
        ApiOnlyMapping {
            kind: "AWS::ApiOnly::DRSRecoveryPlanExecutionStep",
            reason: "DRS recovery-plan execution-step status has its own ARN and is distinct from a plan step definition; no pinned CFN execution-step type.",
        },
    ],
    operations: &[
        OperationMapping {
            service: "database-migration-service",
            operation: "DescribeDataProviders",
            kind: "AWS::DMS::DataProvider",
        },
        OperationMapping {
            service: "dataexchange",
            operation: "ListEventActions",
            kind: "AWS::ApiOnly::DataExchangeEventAction",
        },
        OperationMapping {
            service: "directory-service",
            operation: "DescribeEventTopics",
            kind: "AWS::ApiOnly::DirectoryServiceEventTopicSubscription",
        },
        OperationMapping {
            service: "elasticache",
            operation: "DescribeReservedCacheNodes",
            kind: "AWS::ApiOnly::ElastiCacheReservedCacheNode",
        },
        OperationMapping {
            service: "elasticsearch-service",
            operation: "DescribeReservedElasticsearchInstances",
            kind: "AWS::ApiOnly::ElasticsearchReservedInstance",
        },
        OperationMapping {
            service: "frauddetector",
            operation: "GetBatchImportJobs",
            kind: "AWS::ApiOnly::FraudDetectorBatchImportJob",
        },
        OperationMapping {
            service: "frauddetector",
            operation: "GetBatchPredictionJobs",
            kind: "AWS::ApiOnly::FraudDetectorBatchPredictionJob",
        },
        OperationMapping {
            service: "fsx",
            operation: "DescribeDataRepositoryTasks",
            kind: "AWS::ApiOnly::FSxDataRepositoryTask",
        },
        OperationMapping {
            service: "emr",
            operation: "ListNotebookExecutions",
            kind: "AWS::ApiOnly::EMRNotebookExecution",
        },
        OperationMapping {
            service: "emr",
            operation: "ListStudioSessionMappings",
            kind: "AWS::EMR::StudioSessionMapping",
        },
        OperationMapping {
            service: "dataexchange",
            operation: "ListDataSetRevisions",
            kind: "AWS::ApiOnly::DataExchangeRevision",
        },
        OperationMapping {
            service: "ecr",
            operation: "DescribeImages",
            kind: "AWS::ApiOnly::ECRImage",
        },
        OperationMapping {
            service: "ecr-public",
            operation: "DescribeImages",
            kind: "AWS::ApiOnly::ECRPublicImage",
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
            operation: "DescribeDataMigrations",
            kind: "AWS::ApiOnly::DMSDataMigration",
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
        OperationMapping {
            service: "detective",
            operation: "ListInvitations",
            kind: "AWS::ApiOnly::DetectiveInvitation",
        },
        OperationMapping {
            service: "detective",
            operation: "ListOrganizationAdminAccounts",
            kind: "AWS::ApiOnly::DetectiveOrganizationAdministrator",
        },
        OperationMapping {
            service: "direct-connect",
            operation: "DescribeDirectConnectGatewayAssociationProposals",
            kind: "AWS::ApiOnly::DirectConnectGatewayAssociationProposal",
        },
        OperationMapping {
            service: "direct-connect",
            operation: "DescribeDirectConnectGatewayAssociations",
            kind: "AWS::DirectConnect::DirectConnectGatewayAssociation",
        },
        OperationMapping {
            service: "direct-connect",
            operation: "DescribeDirectConnectGatewayAttachments",
            kind: "AWS::ApiOnly::DirectConnectGatewayAttachment",
        },
        OperationMapping {
            service: "endusermessaging",
            operation: "ListJobs",
            kind: "AWS::ApiOnly::EndUserMessagingJob",
        },
        OperationMapping {
            service: "dsql",
            operation: "ListStreams",
            kind: "AWS::ApiOnly::DSQLStream",
        },
        OperationMapping {
            service: "emr-serverless",
            operation: "ListJobRuns",
            kind: "AWS::ApiOnly::EMRServerlessJobRun",
        },
        OperationMapping {
            service: "emr-serverless",
            operation: "ListSessions",
            kind: "AWS::ApiOnly::EMRServerlessSession",
        },
        OperationMapping {
            service: "detective",
            operation: "ListMembers",
            kind: "AWS::ApiOnly::DetectiveGraphMembership",
        },
        OperationMapping {
            service: "fms",
            operation: "ListAppsLists",
            kind: "AWS::FMS::ApplicationsList",
        },
        OperationMapping {
            service: "fms",
            operation: "ListProtocolsLists",
            kind: "AWS::FMS::ProtocolsList",
        },
        OperationMapping {
            service: "drs",
            operation: "ListRecoveryPlanSteps",
            kind: "AWS::ApiOnly::DRSRecoveryPlanStep",
        },
        OperationMapping {
            service: "drs",
            operation: "ListRecoveryPlanExecutionSteps",
            kind: "AWS::ApiOnly::DRSRecoveryPlanExecutionStep",
        },
    ],
    relations: &[
        RelationMapping {
            source_kind: "AWS::ApiOnly::DSQLStream",
            source: RelationSource::SelectedOutput(RelationPath::Fields("clusterIdentifier")),
            target_kind: "AWS::DSQL::Cluster",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("identifier")),
            relation: "IN_CLUSTER",
            scope: "account_region",
            guard: None,
        },
        RelationMapping {
            source_kind: "AWS::ApiOnly::EMRServerlessJobRun",
            source: RelationSource::SelectedOutput(RelationPath::Fields("applicationId")),
            target_kind: "AWS::EMRServerless::Application",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
            relation: "IN_APPLICATION",
            scope: "account_region",
            guard: None,
        },
        RelationMapping {
            source_kind: "AWS::ApiOnly::EMRServerlessSession",
            source: RelationSource::SelectedOutput(RelationPath::Fields("applicationId")),
            target_kind: "AWS::EMRServerless::Application",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("id")),
            relation: "IN_APPLICATION",
            scope: "account_region",
            guard: None,
        },
        RelationMapping {
            source_kind: "AWS::ApiOnly::DetectiveGraphMembership",
            source: RelationSource::SelectedOutput(RelationPath::Fields("GraphArn")),
            target_kind: "AWS::Detective::Graph",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("Arn")),
            relation: "MEMBER_OF",
            scope: "account_region",
            guard: None,
        },
        RelationMapping {
            source_kind: "AWS::ApiOnly::EMRServerlessJobRun",
            source: RelationSource::SelectedOutput(RelationPath::Fields("executionRole")),
            target_kind: "AWS::IAM::Role",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("Arn")),
            relation: "ASSUMES_ROLE",
            scope: "account",
            guard: None,
        },
        RelationMapping {
            source_kind: "AWS::ApiOnly::EMRServerlessSession",
            source: RelationSource::SelectedOutput(RelationPath::Fields("executionRoleArn")),
            target_kind: "AWS::IAM::Role",
            target: RelationTarget::SelectedOutput(RelationPath::Fields("Arn")),
            relation: "ASSUMES_ROLE",
            scope: "account",
            guard: None,
        },
    ],
    ..super::super::EMPTY
};
