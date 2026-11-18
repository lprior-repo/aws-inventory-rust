use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "clouddirectory",
        operation: "ListDirectories",
        kind: "AWS::ApiOnly::CloudDirectoryDirectory",
    },
    OperationMapping {
        service: "clouddirectory",
        operation: "ListPublishedSchemaArns",
        kind: "AWS::ApiOnly::CloudDirectorySchema",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListGeneratedTemplates",
        kind: "AWS::ApiOnly::CloudFormationGeneratedTemplate",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListResourceScans",
        kind: "AWS::ApiOnly::CloudFormationResourceScan",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListStacks",
        kind: "AWS::CloudFormation::Stack",
    },
    OperationMapping {
        service: "cloudformation",
        operation: "ListStackSets",
        kind: "AWS::CloudFormation::StackSet",
    },
    OperationMapping {
        service: "cloudhsm-v2",
        operation: "DescribeBackups",
        kind: "AWS::ApiOnly::CloudHSMV2Backup",
    },
    OperationMapping {
        service: "cloudhsm",
        operation: "DescribeHapg",
        kind: "AWS::ApiOnly::CloudHSMHAPG",
    },
    OperationMapping {
        service: "cloudhsm",
        operation: "DescribeHsm",
        kind: "AWS::ApiOnly::CloudHSMHSM",
    },
    OperationMapping {
        service: "cloudhsm",
        operation: "ListHapgs",
        kind: "AWS::ApiOnly::CloudHSMHAPG",
    },
    OperationMapping {
        service: "cloudhsm",
        operation: "ListHsms",
        kind: "AWS::ApiOnly::CloudHSMHSM",
    },
    OperationMapping {
        service: "cloudhsm",
        operation: "ListLunaClients",
        kind: "AWS::ApiOnly::CloudHSMClient",
    },
    OperationMapping {
        service: "cloudsearch",
        operation: "DescribeDomains",
        kind: "AWS::ApiOnly::CloudSearchDomain",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "DescribeTrails",
        kind: "AWS::CloudTrail::Trail",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "GetChannel",
        kind: "AWS::CloudTrail::Channel",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "GetEventDataStore",
        kind: "AWS::CloudTrail::EventDataStore",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "GetTrail",
        kind: "AWS::CloudTrail::Trail",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "ListChannels",
        kind: "AWS::CloudTrail::Channel",
    },
    OperationMapping {
        service: "cloudtrail",
        operation: "ListDashboards",
        kind: "AWS::CloudTrail::Dashboard",
    },
];
