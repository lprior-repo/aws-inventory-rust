use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "savingsplans",
        operation: "DescribeSavingsPlans",
        kind: "AWS::ApiOnly::SavingsPlan",
    },
    OperationMapping {
        service: "scheduler",
        operation: "ListScheduleGroups",
        kind: "AWS::Scheduler::ScheduleGroup",
    },
    OperationMapping {
        service: "scheduler",
        operation: "ListSchedules",
        kind: "AWS::Scheduler::Schedule",
    },
    OperationMapping {
        service: "schemas",
        operation: "ListDiscoverers",
        kind: "AWS::EventSchemas::Discoverer",
    },
    OperationMapping {
        service: "schemas",
        operation: "ListRegistries",
        kind: "AWS::EventSchemas::Registry",
    },
    OperationMapping {
        service: "schemas",
        operation: "ListSchemas",
        kind: "AWS::EventSchemas::Schema",
    },
    OperationMapping {
        service: "secrets-manager",
        operation: "ListSecrets",
        kind: "AWS::SecretsManager::Secret",
    },
    OperationMapping {
        service: "security-ir",
        operation: "ListMemberships",
        kind: "AWS::ApiOnly::SecurityIncidentResponseMembership",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListAgentSpaces",
        kind: "AWS::ApiOnly::SecurityAgentSpace",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListApplications",
        kind: "AWS::ApiOnly::SecurityAgentApplication",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListIntegrations",
        kind: "AWS::ApiOnly::SecurityAgentIntegration",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListPrivateConnections",
        kind: "AWS::ApiOnly::SecurityAgentPrivateConnection",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListSecurityRequirementPacks",
        kind: "AWS::ApiOnly::SecurityAgentRequirementPack",
    },
    OperationMapping {
        service: "securityagent",
        operation: "ListTargetDomains",
        kind: "AWS::ApiOnly::SecurityAgentTargetDomain",
    },
    OperationMapping {
        service: "securityhub",
        operation: "DescribeActionTargets",
        kind: "AWS::SecurityHub::ActionTarget",
    },
    OperationMapping {
        service: "securityhub",
        operation: "GetEnabledStandards",
        kind: "AWS::SecurityHub::Standard",
    },
    OperationMapping {
        service: "securityhub",
        operation: "ListFindingAggregators",
        kind: "AWS::SecurityHub::FindingAggregator",
    },
    OperationMapping {
        service: "securityhub",
        operation: "ListMembers",
        kind: "AWS::ApiOnly::SecurityHubMember",
    },
    OperationMapping {
        service: "securityhub",
        operation: "ListConfigurationPolicies",
        kind: "AWS::SecurityHub::ConfigurationPolicy",
    },
    OperationMapping {
        service: "securityhub",
        operation: "ListAutomationRules",
        kind: "AWS::SecurityHub::AutomationRule",
    },
];
