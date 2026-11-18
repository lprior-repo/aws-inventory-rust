use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "backup",
        operation: "ListIndexedRecoveryPoints",
        kind: "AWS::ApiOnly::BackupRecoveryPoint",
    },
    OperationMapping {
        service: "backup",
        operation: "ListLegalHolds",
        kind: "AWS::ApiOnly::BackupLegalHold",
    },
    OperationMapping {
        service: "backup",
        operation: "ListProtectedResources",
        kind: "AWS::ApiOnly::BackupProtectedResource",
    },
    OperationMapping {
        service: "backup",
        operation: "ListRecoveryPointsByBackupVault",
        kind: "AWS::ApiOnly::BackupRecoveryPoint",
    },
    OperationMapping {
        service: "backup",
        operation: "ListReportJobs",
        kind: "AWS::ApiOnly::BackupReportJob",
    },
    OperationMapping {
        service: "backup",
        operation: "ListReportPlans",
        kind: "AWS::Backup::ReportPlan",
    },
    OperationMapping {
        service: "backup",
        operation: "ListRestoreJobs",
        kind: "AWS::ApiOnly::BackupRestoreJob",
    },
    OperationMapping {
        service: "backup",
        operation: "ListRestoreTestingPlans",
        kind: "AWS::Backup::RestoreTestingPlan",
    },
    OperationMapping {
        service: "backup",
        operation: "ListScanJobs",
        kind: "AWS::ApiOnly::BackupScanJob",
    },
    OperationMapping {
        service: "backup",
        operation: "ListTieringConfigurations",
        kind: "AWS::Backup::TieringConfiguration",
    },
    OperationMapping {
        service: "backupsearch",
        operation: "ListSearchJobs",
        kind: "AWS::ApiOnly::BackupSearchSearchJob",
    },
    OperationMapping {
        service: "batch",
        operation: "DescribeComputeEnvironments",
        kind: "AWS::Batch::ComputeEnvironment",
    },
    OperationMapping {
        service: "batch",
        operation: "DescribeJobQueues",
        kind: "AWS::Batch::JobQueue",
    },
    OperationMapping {
        service: "batch",
        operation: "DescribeServiceEnvironments",
        kind: "AWS::Batch::ServiceEnvironment",
    },
    OperationMapping {
        service: "batch",
        operation: "ListConsumableResources",
        kind: "AWS::Batch::ConsumableResource",
    },
    OperationMapping {
        service: "batch",
        operation: "ListSchedulingPolicies",
        kind: "AWS::Batch::SchedulingPolicy",
    },
    OperationMapping {
        service: "bcm-dashboards",
        operation: "ListDashboards",
        kind: "AWS::ApiOnly::BCMDashboardsDashboard",
    },
    OperationMapping {
        service: "bcm-dashboards",
        operation: "ListScheduledReports",
        kind: "AWS::ApiOnly::BCMDashboardsScheduledReport",
    },
    OperationMapping {
        service: "bcm-data-exports",
        operation: "ListExports",
        kind: "AWS::BCMDataExports::Export",
    },
    OperationMapping {
        service: "bcm-pricing-calculator",
        operation: "ListBillEstimates",
        kind: "AWS::ApiOnly::BCMPricingCalculatorBillEstimate",
    },
];
