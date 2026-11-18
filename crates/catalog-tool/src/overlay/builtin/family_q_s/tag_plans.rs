#[path = "tag_parents.rs"]
mod parents;
use super::super::{name, path};
use crate::overlay::{
    Binding, NameError, Paginator, RequestRouting, Scope, ServiceOverlay, TagEndpoint,
    TagEndpointParent, TagEndpointRequest, TagEndpointSelector, TokenPair,
};

type Parent = (&'static str, &'static str);
struct Tags {
    service: &'static str,
    operation: &'static str,
    input: &'static str,
    output: &'static str,
    fields: Option<(&'static str, &'static str)>,
    token: Option<&'static str>,
    parents: &'static [Parent],
}

// Explicit reviewed parent sources. No runtime operation-prefix or ARN-string discovery.
const TAGS: &[Tags] = &[
    Tags {
        service: "sagemaker",
        operation: "ListTags",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: Some("NextToken"),
        parents: parents::SAGEMAKER,
    },
    Tags {
        service: "s3-control",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[
            ("ListStorageLensGroups", "StorageLensGroupArn"),
            ("ListAccessGrantsInstances", "AccessGrantsInstanceArn"),
            ("ListAccessGrantsLocations", "AccessGrantsLocationArn"),
            ("ListAccessGrants", "AccessGrantArn"),
        ],
    },
    Tags {
        service: "route-53-domains",
        operation: "ListTagsForDomain",
        input: "DomainName",
        output: "TagList",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[("ListDomains", "DomainName")],
    },
    Tags {
        service: "route53-recovery-readiness",
        operation: "ListTagsForResources",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[
            ("ListCells", "CellArn"),
            ("ListReadinessChecks", "ReadinessCheckArn"),
            ("ListRecoveryGroups", "RecoveryGroupArn"),
            ("ListResourceSets", "ResourceSetArn"),
        ],
    },
    Tags {
        service: "qconnect",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListAssistants", "assistantArn"),
            ("ListKnowledgeBases", "knowledgeBaseArn"),
            ("ListAssistantAssociations", "assistantAssociationArn"),
        ],
    },
    Tags {
        service: "quicksight",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[
            ("ListAnalyses", "Arn"),
            ("ListDashboards", "Arn"),
            ("ListDataSets", "Arn"),
            ("ListDataSources", "Arn"),
            ("ListFolders", "Arn"),
            ("ListThemes", "Arn"),
            ("ListVPCConnections", "Arn"),
        ],
    },
    Tags {
        service: "redshift-serverless",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: Some(("key", "value")),
        token: None,
        parents: &[
            ("ListNamespaces", "namespaceArn"),
            ("ListWorkgroups", "workgroupArn"),
            ("ListSnapshots", "snapshotArn"),
        ],
    },
    Tags {
        service: "repostspace",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("ListSpaces", "arn")],
    },
    Tags {
        service: "resiliencehub",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListApps", "appArn"),
            ("ListResiliencyPolicies", "policyArn"),
            ("ListAppAssessments", "assessmentArn"),
        ],
    },
    Tags {
        service: "resiliencehubv2",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListServices", "serviceArn"),
            ("ListPolicies", "policyArn"),
        ],
    },
    Tags {
        service: "resource-explorer-2",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[("ListViews", "$"), ("GetIndex", "Arn")],
    },
    Tags {
        service: "resource-groups",
        operation: "GetTags",
        input: "Arn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[("ListGroups", "GroupArn")],
    },
    Tags {
        service: "rolesanywhere",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: Some(("key", "value")),
        token: None,
        parents: &[
            ("ListCrls", "crlArn"),
            ("ListProfiles", "profileArn"),
            ("ListTrustAnchors", "trustAnchorArn"),
        ],
    },
    Tags {
        service: "route53-recovery-control-config",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[
            ("ListClusters", "ClusterArn"),
            ("ListControlPanels", "ControlPanelArn"),
            ("ListRoutingControls", "RoutingControlArn"),
        ],
    },
    Tags {
        service: "route53profiles",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[("ListProfiles", "Arn")],
    },
    Tags {
        service: "route53resolver",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: Some("NextToken"),
        parents: &[
            ("ListResolverEndpoints", "Arn"),
            ("ListResolverRules", "Arn"),
            ("ListResolverQueryLogConfigs", "Arn"),
            ("ListFirewallDomainLists", "Arn"),
            ("ListFirewallRuleGroups", "Arn"),
            ("ListOutpostResolvers", "Arn"),
        ],
    },
    Tags {
        service: "s3tables",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("ListTableBuckets", "arn"), ("ListTables", "tableARN")],
    },
    Tags {
        service: "s3vectors",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListVectorBuckets", "vectorBucketArn"),
            ("ListIndexes", "indexArn"),
        ],
    },
    Tags {
        service: "sagemaker-geospatial",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[
            ("ListEarthObservationJobs", "Arn"),
            ("ListVectorEnrichmentJobs", "Arn"),
        ],
    },
    Tags {
        service: "savingsplans",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("DescribeSavingsPlans", "savingsPlanArn")],
    },
    Tags {
        service: "scheduler",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[("ListScheduleGroups", "Arn")],
    },
    Tags {
        service: "schemas",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[
            ("ListDiscoverers", "DiscovererArn"),
            ("ListRegistries", "RegistryArn"),
            ("ListSchemas", "SchemaArn"),
        ],
    },
    Tags {
        service: "security-ir",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("ListMemberships", "membershipArn")],
    },
    Tags {
        service: "securityhub",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[
            ("DescribeHub", "HubArn"),
            ("DescribeActionTargets", "ActionTargetArn"),
            ("ListFindingAggregators", "FindingAggregatorArn"),
            ("ListConfigurationPolicies", "Arn"),
            ("ListAutomationRules", "RuleArn"),
            ("GetInsights", "InsightArn"),
        ],
    },
    Tags {
        service: "securitylake",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: Some(("key", "value")),
        token: None,
        parents: &[("ListSubscribers", "subscriberArn")],
    },
    Tags {
        service: "service-catalog-appregistry",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("ListApplications", "arn"), ("ListAttributeGroups", "arn")],
    },
    Tags {
        service: "servicediscovery",
        operation: "ListTagsForResource",
        input: "ResourceARN",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[("ListServices", "Arn")],
    },
    Tags {
        service: "sesv2",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[("ListTenants", "TenantArn")],
    },
    Tags {
        service: "shield",
        operation: "ListTagsForResource",
        input: "ResourceARN",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[
            ("ListProtectionGroups", "ProtectionGroupArn"),
            ("ListProtections", "ProtectionArn"),
        ],
    },
    Tags {
        service: "signer",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[("ListSigningProfiles", "arn")],
    },
    Tags {
        service: "snow-device-management",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListDevices", "managedDeviceArn"),
            ("ListTasks", "taskArn"),
        ],
    },
    Tags {
        service: "socialmessaging",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: Some(("key", "value")),
        token: None,
        parents: &[("ListLinkedWhatsAppBusinessAccounts", "arn")],
    },
    Tags {
        service: "ssm-contacts",
        operation: "ListTagsForResource",
        input: "ResourceARN",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: None,
        parents: &[
            ("ListContacts", "ContactArn"),
            ("ListRotations", "RotationArn"),
        ],
    },
    Tags {
        service: "ssm-incidents",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListResponsePlans", "arn"),
            ("ListReplicationSets", "$"),
            ("ListIncidentRecords", "arn"),
        ],
    },
    Tags {
        service: "ssm-sap",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListApplications", "Arn"),
            ("ListComponents", "Arn"),
            ("ListDatabases", "Arn"),
        ],
    },
    Tags {
        service: "storage-gateway",
        operation: "ListTagsForResource",
        input: "ResourceARN",
        output: "Tags",
        fields: Some(("Key", "Value")),
        token: Some("Marker"),
        parents: &[
            ("ListGateways", "GatewayARN"),
            ("ListFileShares", "FileShareARN"),
            ("ListFileSystemAssociations", "FileSystemAssociationARN"),
            ("ListTapePools", "PoolARN"),
            ("ListTapes", "TapeARN"),
            ("ListVolumes", "VolumeARN"),
            ("DescribeTapeArchives", "TapeARN"),
        ],
    },
    Tags {
        service: "supplychain",
        operation: "ListTagsForResource",
        input: "resourceArn",
        output: "tags",
        fields: None,
        token: None,
        parents: &[
            ("ListDataLakeNamespaces", "arn"),
            ("ListDataLakeDatasets", "arn"),
        ],
    },
    Tags {
        service: "synthetics",
        operation: "ListTagsForResource",
        input: "ResourceArn",
        output: "Tags",
        fields: None,
        token: None,
        parents: &[("ListGroups", "Arn")],
    },
];

pub(super) fn apply(mut service: ServiceOverlay) -> Result<ServiceOverlay, NameError> {
    for spec in TAGS.iter().filter(|spec| spec.service == service.service) {
        let parents = spec
            .parents
            .iter()
            .map(|(operation, field)| {
                Ok(TagEndpointParent {
                    operation: name(operation)?,
                    request: TagEndpointRequest::Member {
                        input: path(spec.input)?,
                        parent: path(field)?,
                    },
                })
            })
            .collect::<Result<Vec<_>, NameError>>()?;
        let selector = match spec.fields {
            Some((key, value)) => TagEndpointSelector::List {
                path: path(spec.output)?,
                key: path(key)?,
                value: path(value)?,
            },
            None => TagEndpointSelector::Map {
                path: path(spec.output)?,
            },
        };
        let paginator = match spec.token {
            Some(field) => Paginator::Token(TokenPair {
                input: path(field)?,
                output: path(field)?,
                optional: false,
            }),
            None => Paginator::None,
        };
        let mut operation =
            TagEndpoint { parents, selector }.operation(Scope::Region, paginator)?;
        if spec.service == "s3-control" {
            operation.bind.insert(path("AccountId")?, Binding::Account);
        }
        if let Some(region) = match spec.service {
            "route-53-domains" => Some("us-east-1"),
            "route53-recovery-readiness" => Some("us-west-2"),
            _ => None,
        } {
            operation.routing = RequestRouting::FixedRegion {
                region: region.to_owned(),
            };
        }
        service.operations.insert(name(spec.operation)?, operation);
    }
    Ok(service)
}
