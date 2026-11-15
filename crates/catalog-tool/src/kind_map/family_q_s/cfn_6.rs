use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::SageMaker::Workforce"),
    CfnMapping::canonical("AWS::SageMaker::Workteam"),
    CfnMapping::canonical("AWS::Scheduler::Schedule"),
    CfnMapping::canonical("AWS::Scheduler::ScheduleGroup"),
    CfnMapping::canonical("AWS::SecretsManager::Secret"),
    CfnMapping::canonical("AWS::SecurityHub::ActionTarget"),
    CfnMapping::canonical("AWS::SecurityHub::AutomationRule"),
    CfnMapping::canonical("AWS::SecurityHub::ConfigurationPolicy"),
    CfnMapping::canonical("AWS::SecurityHub::FindingAggregator"),
    CfnMapping::canonical("AWS::SecurityHub::Hub"),
    CfnMapping::canonical("AWS::SecurityHub::Insight"),
    CfnMapping::canonical("AWS::SecurityHub::ProductSubscription"),
    CfnMapping::canonical("AWS::SecurityHub::Standard"),
    CfnMapping::canonical("AWS::SecurityLake::Subscriber"),
    CfnMapping::canonical("AWS::ServiceCatalog::Portfolio"),
    CfnMapping::canonical("AWS::ServiceCatalog::ServiceAction"),
    CfnMapping::canonical("AWS::ServiceCatalog::TagOption"),
    CfnMapping::canonical("AWS::ServiceCatalogAppRegistry::Application"),
    CfnMapping::canonical("AWS::ServiceCatalogAppRegistry::AttributeGroup"),
    CfnMapping::canonical("AWS::ServiceDiscovery::Service"),
];
