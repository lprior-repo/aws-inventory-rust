use super::{name, path, token};
use crate::overlay::{NameError, Paginator, RequestRouting, Scope, ServiceOverlay, TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector};

struct Tags {
    service: &'static str, input: &'static str, output: &'static str,
    fields: Option<(&'static str, &'static str)>, token: Option<(&'static str, &'static str)>,
    parents: &'static [(&'static str, &'static str)],
}

const TAGS: &[Tags] = &[
    Tags { service: "timestream-influxdb", input: "resourceArn", output: "tags", fields: None, token: None, parents: &[("ListDbBackups", "arn"), ("ListDbClusters", "arn"), ("ListDbInstances", "arn"), ("ListDbParameterGroups", "arn")] },
    Tags { service: "timestream-write", input: "ResourceARN", output: "Tags", fields: Some(("Key", "Value")), token: None, parents: &[("ListDatabases", "Arn"), ("ListTables", "Arn")] },
    Tags { service: "timestream-query", input: "ResourceARN", output: "Tags", fields: Some(("Key", "Value")), token: Some(("NextToken", "NextToken")), parents: &[("ListScheduledQueries", "Arn")] },
    Tags { service: "transfer", input: "Arn", output: "Tags", fields: Some(("Key", "Value")), token: Some(("NextToken", "NextToken")), parents: &[("ListServers", "Arn"), ("ListAgreements", "Arn"), ("ListCertificates", "Arn"), ("ListConnectors", "Arn"), ("ListProfiles", "Arn"), ("ListUsers", "Arn"), ("ListWebApps", "Arn"), ("ListWorkflows", "Arn")] },
    Tags { service: "translate", input: "ResourceArn", output: "Tags", fields: Some(("Key", "Value")), token: None, parents: &[("ListParallelData", "Arn"), ("ListTerminologies", "Arn")] },
    Tags { service: "verifiedpermissions", input: "resourceArn", output: "tags", fields: None, token: None, parents: &[("ListPolicyStores", "arn")] },
    Tags { service: "voice-id", input: "ResourceArn", output: "Tags", fields: Some(("Key", "Value")), token: None, parents: &[("ListDomains", "Arn")] },
    Tags { service: "vpc-lattice", input: "resourceArn", output: "tags", fields: None, token: None, parents: &[("ListServices", "arn"), ("ListServiceNetworks", "arn"), ("ListResourceConfigurations", "arn"), ("ListResourceGateways", "arn"), ("ListTargetGroups", "arn"), ("ListListeners", "arn"), ("ListServiceNetworkResourceAssociations", "arn"), ("ListServiceNetworkServiceAssociations", "arn"), ("ListServiceNetworkVpcAssociations", "arn")] },
    Tags { service: "wafv2", input: "ResourceARN", output: "TagInfoForResource.TagList", fields: Some(("Key", "Value")), token: Some(("NextMarker", "NextMarker")), parents: &[("ListWebACLs", "ARN"), ("ListIPSets", "ARN"), ("ListRegexPatternSets", "ARN"), ("ListRuleGroups", "ARN")] },
    Tags { service: "wellarchitected", input: "WorkloadArn", output: "Tags", fields: None, token: None, parents: &[("ListWorkloads", "WorkloadArn")] },
    Tags { service: "wisdom", input: "resourceArn", output: "tags", fields: None, token: None, parents: &[("ListAssistants", "assistantArn"), ("ListKnowledgeBases", "knowledgeBaseArn"), ("ListAssistantAssociations", "assistantAssociationArn")] },
    Tags { service: "workspaces-thin-client", input: "resourceArn", output: "tags", fields: None, token: None, parents: &[("ListDevices", "arn")] },
    Tags { service: "workspaces-web", input: "resourceArn", output: "tags", fields: Some(("Key", "Value")), token: None, parents: &[("ListPortals", "portalArn"), ("ListBrowserSettings", "browserSettingsArn"), ("ListDataProtectionSettings", "dataProtectionSettingsArn"), ("ListIpAccessSettings", "ipAccessSettingsArn"), ("ListNetworkSettings", "networkSettingsArn"), ("ListSessionLoggers", "sessionLoggerArn"), ("ListTrustStores", "trustStoreArn"), ("ListUserAccessLoggingSettings", "userAccessLoggingSettingsArn"), ("ListUserSettings", "userSettingsArn")] },
    Tags { service: "xray", input: "ResourceARN", output: "Tags", fields: Some(("Key", "Value")), token: Some(("NextToken", "NextToken")), parents: &[("GetGroups", "GroupARN"), ("GetSamplingRules", "SamplingRule.RuleARN")] },
];

pub(super) fn apply(mut services: Vec<ServiceOverlay>) -> Result<Vec<ServiceOverlay>, NameError> {
    for service in &mut services {
        for spec in TAGS.iter().filter(|spec| spec.service == service.service) {
            let parents = spec.parents.iter().map(|(operation, field)| Ok(TagEndpointParent {
                operation: name(operation)?, request: TagEndpointRequest::Member { input: path(spec.input)?, parent: path(field)? },
            })).collect::<Result<Vec<_>, NameError>>()?;
            let selector = match spec.fields {
                Some((key, value)) => TagEndpointSelector::List { path: path(spec.output)?, key: path(key)?, value: path(value)? },
                None => TagEndpointSelector::Map { path: path(spec.output)? },
            };
            let paginator = match spec.token { Some((input, output)) => token(input, output)?, None => Paginator::None };
            let mut plan = TagEndpoint { parents, selector }.operation(Scope::Region, paginator)?;
            if matches!(spec.service, "timestream-write" | "timestream-query") {
                plan.routing = RequestRouting::Discovered {
                    operation: "DescribeEndpoints".to_owned(), endpoints_path: path("Endpoints")?,
                    address_path: path("Address")?, cache_minutes_path: path("CachePeriodInMinutes")?,
                    trusted_dns_suffix: ".timestream.{region}.amazonaws.com".to_owned(),
                };
            }
            if spec.service == "wafv2" {
                // Tag APIs lack a Scope input, but must retain the owner's regional/CloudFront route.
                plan.branches = super::endpoint_plans::branches(false)?;
            }
            service.operations.insert(name("ListTagsForResource")?, plan);
        }
    }
    Ok(services)
}
