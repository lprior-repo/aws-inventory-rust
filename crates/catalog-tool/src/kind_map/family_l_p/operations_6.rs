use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "mwaa", operation: "ListEnvironments", kind: "AWS::MWAA::Environment" },
    OperationMapping { service: "mwaa-serverless", operation: "ListWorkflows", kind: "AWS::ApiOnly::MwaaServerlessWorkflow" },
    OperationMapping { service: "neptune-graph", operation: "ListGraphs", kind: "AWS::NeptuneGraph::Graph" },
    OperationMapping { service: "neptune-graph", operation: "ListGraphSnapshots", kind: "AWS::NeptuneGraph::GraphSnapshot" },
    OperationMapping { service: "neptune", operation: "DescribeDBClusters", kind: "AWS::Neptune::DBCluster" },
    OperationMapping { service: "neptune", operation: "DescribeDBClusterEndpoints", kind: "AWS::ApiOnly::NeptuneClusterEndpoint" },
    OperationMapping { service: "neptune", operation: "DescribeDBClusterParameterGroups", kind: "AWS::Neptune::DBClusterParameterGroup" },
    OperationMapping { service: "neptune", operation: "DescribeDBParameterGroups", kind: "AWS::Neptune::DBParameterGroup" },
    OperationMapping { service: "neptune", operation: "DescribeDBSubnetGroups", kind: "AWS::Neptune::DBSubnetGroup" },
    OperationMapping { service: "neptune", operation: "DescribeDBClusterSnapshots", kind: "AWS::ApiOnly::NeptuneClusterSnapshot" },
    OperationMapping { service: "neptune", operation: "DescribeEventSubscriptions", kind: "AWS::Neptune::EventSubscription" },
    OperationMapping { service: "neptune", operation: "DescribeGlobalClusters", kind: "AWS::Neptune::GlobalCluster" },
    OperationMapping { service: "network-firewall", operation: "ListFirewalls", kind: "AWS::NetworkFirewall::Firewall" },
    OperationMapping { service: "network-firewall", operation: "ListFirewallPolicies", kind: "AWS::NetworkFirewall::FirewallPolicy" },
    OperationMapping { service: "network-firewall", operation: "ListRuleGroups", kind: "AWS::NetworkFirewall::RuleGroup" },
    OperationMapping { service: "network-firewall", operation: "ListTLSInspectionConfigurations", kind: "AWS::NetworkFirewall::TLSInspectionConfiguration" },
    OperationMapping { service: "network-firewall", operation: "ListVpcEndpointAssociations", kind: "AWS::NetworkFirewall::VpcEndpointAssociation" },
    OperationMapping { service: "network-security-manager", operation: "ListScopes", kind: "AWS::NetworkSecurityManager::Scope" },
    OperationMapping { service: "network-security-manager", operation: "ListPolicies", kind: "AWS::NetworkSecurityManager::Policy" },
    OperationMapping { service: "network-security-manager", operation: "ListRules", kind: "AWS::NetworkSecurityManager::Rule" },
];
