use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::AuditManagerAssessmentReport",
        reason: "An existing Audit Manager assessment-report metadata object is not the assessment definition; no report document is downloaded.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupAccessPoint",
        reason: "Backup access-point metadata identifies an access point over recovery data, not an S3 access point or backup vault; no exact pinned Backup access-point type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupBackupJob",
        reason: "Backup BackupJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupCopyJob",
        reason: "Backup CopyJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupGatewayGateway",
        reason: "Backup Gateway gateway appliance metadata keyed by GatewayArn is distinct from a hypervisor; the pinned CFN set has no gateway appliance type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupGatewayVirtualMachine",
        reason: "Backup Gateway discovered virtual machine metadata describes an external hypervisor VM, not an EC2 instance or hypervisor definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupLegalHold",
        reason: "Backup legal hold metadata identifies a retention hold over recovery points, not a framework or vault; no matching pinned legal-hold type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupProtectedResource",
        reason: "Backup protected-resource metadata is a backup-protection observation of an existing AWS resource, not its underlying EC2/RDS/EFS resource definition.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupRecoveryPoint",
        reason: "A Backup recovery point is a retained backup snapshot identified by RecoveryPointArn, not a live protected resource or backup vault.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupReportJob",
        reason: "Backup ReportJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupRestoreJob",
        reason: "Backup RestoreJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupScanJob",
        reason: "Backup ScanJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BackupSearchSearchJob",
        reason: "Existing Backup Search job metadata is an execution of a recovery-data search, not a recovery point or vault; no result content is fetched.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BCMDashboardsDashboard",
        reason: "Billing and Cost Management dashboard summaries are not CloudWatch dashboards; no pinned billing-dashboard definition type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BCMDashboardsScheduledReport",
        reason: "Billing and Cost Management scheduled dashboard-report metadata is not a CUR report or dashboard; no exact pinned scheduled-report type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BCMPricingCalculatorBillEstimate",
        reason: "A saved Pricing Calculator bill estimate is a hypothetical billing scenario result, not a charge or provisioned AWS resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BCMPricingCalculatorBillScenario",
        reason: "A saved Pricing Calculator bill scenario represents modeling inputs, not actual deployed AWS resources or a billing group.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BCMPricingCalculatorWorkloadEstimate",
        reason: "A saved Pricing Calculator workload estimate is a hypothetical workload-cost model, not actual capacity or a provisioned workload.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BedrockAdvancedPromptOptimizationJob",
        reason: "Bedrock AdvancedPromptOptimizationJob metadata identifies an already-created asynchronous execution, not a declarative service configuration or underlying data; no job input/output contents are read and no job is started.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::BedrockAgentCoreRegistry",
        reason: "AgentCore registry summaries identify agent capability registries, not runtime deployments or workload identities; no pinned CFN Registry resource.",
    },
];
