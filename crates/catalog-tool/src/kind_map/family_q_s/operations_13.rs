use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "ssm",
        operation: "ListResourceDataSync",
        kind: "AWS::SSM::ResourceDataSync",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribeActivations",
        kind: "AWS::ApiOnly::SSMActivation",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListCloudConnectors",
        kind: "AWS::ApiOnly::SSMCloudConnector",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribeInventoryDeletions",
        kind: "AWS::ApiOnly::SSMInventoryDeletion",
    },
    OperationMapping {
        service: "ssm",
        operation: "DescribePatchGroups",
        kind: "AWS::ApiOnly::SSMPatchGroup",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListComplianceSummaries",
        kind: "AWS::ApiOnly::SSMComplianceSummary",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListOpsItemRelatedItems",
        kind: "AWS::ApiOnly::SSMOpsItemAssociation",
    },
    OperationMapping {
        service: "ssm",
        operation: "ListResourceComplianceSummaries",
        kind: "AWS::ApiOnly::SSMResourceComplianceSummary",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListContacts",
        kind: "AWS::SSMContacts::Contact",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListRotations",
        kind: "AWS::SSMContacts::Rotation",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListEngagements",
        kind: "AWS::ApiOnly::SSMContactEngagement",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListContactChannels",
        kind: "AWS::ApiOnly::SSMContactChannel",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListPageReceipts",
        kind: "AWS::ApiOnly::SSMContactPageReceipt",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListPageResolutions",
        kind: "AWS::ApiOnly::SSMContactPageResolution",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListPagesByContact",
        kind: "AWS::ApiOnly::SSMContactPage",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListPagesByEngagement",
        kind: "AWS::ApiOnly::SSMContactPage",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListRotationOverrides",
        kind: "AWS::ApiOnly::SSMContactRotationOverride",
    },
    OperationMapping {
        service: "ssm-contacts",
        operation: "ListRotationShifts",
        kind: "AWS::ApiOnly::SSMContactRotationShift",
    },
    OperationMapping {
        service: "ssm-incidents",
        operation: "ListResponsePlans",
        kind: "AWS::SSMIncidents::ResponsePlan",
    },
    OperationMapping {
        service: "ssm-incidents",
        operation: "ListReplicationSets",
        kind: "AWS::SSMIncidents::ReplicationSet",
    },
    OperationMapping {
        service: "ssm-incidents",
        operation: "ListIncidentRecords",
        kind: "AWS::ApiOnly::SSMIncidentRecord",
    },
    OperationMapping {
        service: "ssm-sap",
        operation: "ListApplications",
        kind: "AWS::SystemsManagerSAP::Application",
    },
    OperationMapping {
        service: "ssm-sap",
        operation: "ListComponents",
        kind: "AWS::ApiOnly::SAPComponent",
    },
    OperationMapping {
        service: "ssm-sap",
        operation: "ListDatabases",
        kind: "AWS::ApiOnly::SAPDatabase",
    },
    OperationMapping {
        service: "sso-admin",
        operation: "ListInstances",
        kind: "AWS::SSO::Instance",
    },
    OperationMapping {
        service: "sso-admin",
        operation: "ListPermissionSets",
        kind: "AWS::SSO::PermissionSet",
    },
    OperationMapping {
        service: "sso-admin",
        operation: "ListApplications",
        kind: "AWS::SSO::Application",
    },
];
