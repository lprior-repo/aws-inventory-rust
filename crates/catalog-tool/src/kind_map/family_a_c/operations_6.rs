use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "b2bi",
        operation: "GetPartnership",
        kind: "AWS::B2BI::Partnership",
    },
    OperationMapping {
        service: "b2bi",
        operation: "GetProfile",
        kind: "AWS::B2BI::Profile",
    },
    OperationMapping {
        service: "b2bi",
        operation: "ListCapabilities",
        kind: "AWS::B2BI::Capability",
    },
    OperationMapping {
        service: "b2bi",
        operation: "ListPartnerships",
        kind: "AWS::B2BI::Partnership",
    },
    OperationMapping {
        service: "b2bi",
        operation: "ListProfiles",
        kind: "AWS::B2BI::Profile",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "GetGateway",
        kind: "AWS::ApiOnly::BackupGatewayGateway",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "GetHypervisor",
        kind: "AWS::BackupGateway::Hypervisor",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "GetVirtualMachine",
        kind: "AWS::ApiOnly::BackupGatewayVirtualMachine",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "ListGateways",
        kind: "AWS::ApiOnly::BackupGatewayGateway",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "ListHypervisors",
        kind: "AWS::BackupGateway::Hypervisor",
    },
    OperationMapping {
        service: "backup-gateway",
        operation: "ListVirtualMachines",
        kind: "AWS::ApiOnly::BackupGatewayVirtualMachine",
    },
    OperationMapping {
        service: "backup",
        operation: "DescribeBackupVault",
        kind: "AWS::Backup::BackupVault",
    },
    OperationMapping {
        service: "backup",
        operation: "DescribeFramework",
        kind: "AWS::Backup::Framework",
    },
    OperationMapping {
        service: "backup",
        operation: "DescribeReportPlan",
        kind: "AWS::Backup::ReportPlan",
    },
    OperationMapping {
        service: "backup",
        operation: "ListBackupAccessPoints",
        kind: "AWS::ApiOnly::BackupAccessPoint",
    },
    OperationMapping {
        service: "backup",
        operation: "ListBackupJobs",
        kind: "AWS::ApiOnly::BackupBackupJob",
    },
    OperationMapping {
        service: "backup",
        operation: "ListBackupPlans",
        kind: "AWS::Backup::BackupPlan",
    },
    OperationMapping {
        service: "backup",
        operation: "ListBackupVaults",
        kind: "AWS::Backup::BackupVault",
    },
    OperationMapping {
        service: "backup",
        operation: "ListCopyJobs",
        kind: "AWS::ApiOnly::BackupCopyJob",
    },
    OperationMapping {
        service: "backup",
        operation: "ListFrameworks",
        kind: "AWS::Backup::Framework",
    },
];
