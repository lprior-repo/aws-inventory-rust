use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "servicediscovery",
        operation: "ListServices",
        kind: "AWS::ServiceDiscovery::Service",
    },
    OperationMapping {
        service: "servicediscovery",
        operation: "ListOperations",
        kind: "AWS::ApiOnly::ServiceDiscoveryOperation",
    },
    OperationMapping {
        service: "ses",
        operation: "ListConfigurationSets",
        kind: "AWS::SES::ConfigurationSet",
    },
    OperationMapping {
        service: "ses",
        operation: "ListIdentities",
        kind: "AWS::SES::EmailIdentity",
    },
    OperationMapping {
        service: "ses",
        operation: "ListReceiptRuleSets",
        kind: "AWS::SES::ReceiptRuleSet",
    },
    OperationMapping {
        service: "ses",
        operation: "ListTemplates",
        kind: "AWS::SES::Template",
    },
    OperationMapping {
        service: "ses",
        operation: "ListReceiptFilters",
        kind: "AWS::SES::ReceiptFilter",
    },
    OperationMapping {
        service: "ses",
        operation: "ListCustomVerificationEmailTemplates",
        kind: "AWS::ApiOnly::SESCustomVerificationEmailTemplate",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListConfigurationSets",
        kind: "AWS::SES::ConfigurationSet",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListContactLists",
        kind: "AWS::SES::ContactList",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListDedicatedIpPools",
        kind: "AWS::SES::DedicatedIpPool",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListEmailIdentities",
        kind: "AWS::SES::EmailIdentity",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListEmailTemplates",
        kind: "AWS::SES::Template",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListMultiRegionEndpoints",
        kind: "AWS::SES::MultiRegionEndpoint",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListTenants",
        kind: "AWS::SES::Tenant",
    },
    OperationMapping {
        service: "sesv2",
        operation: "GetDedicatedIps",
        kind: "AWS::ApiOnly::SESDedicatedIp",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListExportJobs",
        kind: "AWS::ApiOnly::SESExportJob",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListImportJobs",
        kind: "AWS::ApiOnly::SESImportJob",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListReputationEntities",
        kind: "AWS::ApiOnly::SESReputationEntity",
    },
    OperationMapping {
        service: "sesv2",
        operation: "ListCustomVerificationEmailTemplates",
        kind: "AWS::ApiOnly::SESCustomVerificationEmailTemplate",
    },
];
