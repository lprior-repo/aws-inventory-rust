use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "sesv2",
        operation: "ListDeliverabilityTestReports",
        kind: "AWS::ApiOnly::SESDeliverabilityTestReport",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListRecommendations",
        kind: "AWS::ApiOnly::SESRecommendation",
    },
    OperationMapping {
        service: "shield",
        operation: "ListProtectionGroups",
        kind: "AWS::Shield::ProtectionGroup",
    },
    OperationMapping {
        service: "shield",
        operation: "ListProtections",
        kind: "AWS::Shield::Protection",
    },
    OperationMapping {
        service: "shield",
        operation: "ListAttacks",
        kind: "AWS::ApiOnly::ShieldAttack",
    },
    OperationMapping {
        service: "signer",
        operation: "ListSigningProfiles",
        kind: "AWS::Signer::SigningProfile",
    },
    OperationMapping {
        service: "signer",
        operation: "ListSigningJobs",
        kind: "AWS::ApiOnly::SignerSigningJob",
    },
    OperationMapping {
        service: "simpledbv2",
        operation: "ListExports",
        kind: "AWS::ApiOnly::SimpleDBExport",
    },
    OperationMapping {
        service: "snow-device-management",
        operation: "ListDevices",
        kind: "AWS::ApiOnly::SnowDevice",
    },
    OperationMapping {
        service: "snow-device-management",
        operation: "ListTasks",
        kind: "AWS::ApiOnly::SnowDeviceTask",
    },
    OperationMapping {
        service: "snowball",
        operation: "ListClusters",
        kind: "AWS::ApiOnly::SnowballCluster",
    },
    OperationMapping {
        service: "snowball",
        operation: "ListJobs",
        kind: "AWS::ApiOnly::SnowballJob",
    },
    OperationMapping {
        service: "snowball",
        operation: "ListLongTermPricing",
        kind: "AWS::ApiOnly::SnowballLongTermRental",
    },
    OperationMapping {
        service: "socialmessaging",
        operation: "ListLinkedWhatsAppBusinessAccounts",
        kind: "AWS::ApiOnly::WhatsAppBusinessAccount",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribeInstanceInformation",
        kind: "AWS::ApiOnly::SSMManagedInstance",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribeMaintenanceWindows",
        kind: "AWS::SSM::MaintenanceWindow",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribeParameters",
        kind: "AWS::SSM::Parameter",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribePatchBaselines",
        kind: "AWS::SSM::PatchBaseline",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListAssociations",
        kind: "AWS::SSM::Association",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListOpsMetadata",
        kind: "AWS::ApiOnly::SSMOpsMetadata",
    },
];
