use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "securityhub", operation: "ListEnabledProductsForImport", kind: "AWS::SecurityHub::ProductSubscription" },
    OperationMapping { service: "securityhub", operation: "DescribeHub", kind: "AWS::SecurityHub::Hub" },
    OperationMapping { service: "securityhub", operation: "GetInsights", kind: "AWS::SecurityHub::Insight" },
    OperationMapping { service: "securityhub", operation: "ListAggregatorsV2", kind: "AWS::ApiOnly::SecurityHubAggregator" },
    OperationMapping { service: "securityhub", operation: "ListConnectors", kind: "AWS::ApiOnly::SecurityHubConnector" },
    OperationMapping { service: "securityhub", operation: "ListConnectorsV2", kind: "AWS::ApiOnly::SecurityHubConnector" },
    OperationMapping { service: "securityhub", operation: "ListAutomationRulesV2", kind: "AWS::ApiOnly::SecurityHubAutomationRuleV2" },
    OperationMapping { service: "securityhub", operation: "ListExportJobsV2", kind: "AWS::ApiOnly::SecurityHubExportJob" },
    OperationMapping { service: "securityhub", operation: "ListFreeTrialStatusesV2", kind: "AWS::ApiOnly::SecurityHubAccountFreeTrialStatus" },
    OperationMapping { service: "securityhub", operation: "ListInvitations", kind: "AWS::ApiOnly::SecurityHubInvitation" },
    OperationMapping { service: "securityhub", operation: "ListOrganizationAdminAccounts", kind: "AWS::ApiOnly::SecurityHubOrganizationAdministrator" },
    OperationMapping { service: "securitylake", operation: "ListSubscribers", kind: "AWS::SecurityLake::Subscriber" },
    OperationMapping { service: "serverlessapplicationrepository", operation: "ListApplications", kind: "AWS::ApiOnly::ServerlessApplication" },
    OperationMapping { service: "service-catalog", operation: "ListPortfolios", kind: "AWS::ServiceCatalog::Portfolio" },
    OperationMapping { service: "service-catalog", operation: "ListTagOptions", kind: "AWS::ServiceCatalog::TagOption" },
    OperationMapping { service: "service-catalog", operation: "ListServiceActions", kind: "AWS::ServiceCatalog::ServiceAction" },
    OperationMapping { service: "service-catalog", operation: "ListProvisionedProductPlans", kind: "AWS::ApiOnly::ServiceCatalogProvisionedProductPlan" },
    OperationMapping { service: "service-catalog-appregistry", operation: "ListApplications", kind: "AWS::ServiceCatalogAppRegistry::Application" },
    OperationMapping { service: "service-catalog-appregistry", operation: "ListAttributeGroups", kind: "AWS::ServiceCatalogAppRegistry::AttributeGroup" },
    OperationMapping { service: "service-quotas", operation: "ListRequestedServiceQuotaChangeHistory", kind: "AWS::ApiOnly::ServiceQuotaChangeRequest" },
];
