// Pinned Smithy source: .local/aws-models/models/opensearch/service/2021-01-01/opensearch-2021-01-01.json
use super::policy::{Group, Id, Input, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptInboundConnection", "AddDataSource", "AddDirectQueryDataSource", 
            "AddReplicationPeer", "AddTags", "AssociatePackage", "AssociatePackages", 
            "AttachDataSource", "AuthorizeVpcEndpointAccess", "CancelDomainConfigChange", 
            "CancelServiceSoftwareUpdate", "CreateApplication", "CreateDomain", "CreateIndex", 
            "CreateOutboundConnection", "CreatePackage", "CreateVpcEndpoint", 
            "DeleteApplication", "DeleteDataSource", "DeleteDirectQueryDataSource", 
            "DeleteDomain", "DeleteInboundConnection", "DeleteIndex", 
            "DeleteOutboundConnection", "DeletePackage", "DeleteVpcEndpoint", 
            "DeregisterCapability", "DetachDataSource", "DissociatePackage", 
            "DissociatePackages", "FailoverReplication", "PurchaseReservedInstanceOffering", 
            "PutDefaultApplicationSetting", "RegisterCapability", "RejectInboundConnection", 
            "RemoveReplicationPeer", "RemoveTags", "RevokeVpcEndpointAccess", 
            "RollbackServiceSoftwareUpdate", "StartDomainMaintenance", "StartMigration", 
            "StartServiceSoftwareUpdate", "SwitchoverReplication", "UpdateApplication", 
            "UpdateClone", "UpdateDataSource", "UpdateDirectQueryDataSource", 
            "UpdateDomainConfig", "UpdateIndex", "UpdatePackage", "UpdatePackageScope", 
            "UpdateScheduledAction", "UpdateVpcEndpoint",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "CompareDomainConfiguration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (SourceDomainArn, TargetDomainArn, Categories); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeDataSourceAttachment",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (id, dataSourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeDomain", "DescribeDomainConfig", "DescribeDomains",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeDomainAutoTunes", "DescribeDomainChangeProgress", "DescribeDomainHealth", 
            "DescribeDomainNodes", "DescribeDomainReplication", "DescribeDryRunProgress", 
            "GetUpgradeHistory", "GetUpgradeStatus", "ListDataSources", 
            "ListDomainMaintenances", "ListPackagesForDomain", "ListScheduledActions", 
            "ListVpcEndpointAccess", "ListVpcEndpointsForDomain",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DomainName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeInboundConnections", "DescribeOutboundConnections", "DescribePackages", 
            "DescribeReservedInstanceOfferings", "DescribeReservedInstances", 
            "GetCompatibleVersions", "GetDefaultApplicationSetting", "ListVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "DescribeInsightDetails",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Entity, InsightId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeInstanceTypeLimits",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (InstanceType, EngineVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeVpcEndpoints",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (VpcEndpointIds); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetApplication", "ListDataSourceAttachments",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetCapability",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (applicationId, capabilityName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDataSource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DomainName, Name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDirectQueryDataSource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DataSourceName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDomainMaintenanceStatus",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DomainName, MaintenanceId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetIndex",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DomainName, IndexName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetMigration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (migrationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetPackageVersionHistory", "ListDomainsForPackage",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PackageID); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "InsightFeedback",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Entity, InsightId, Thumbs); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListInsights",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Entity); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListInstanceTypeDetails",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EngineVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListMigrations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (applicationId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTags",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ARN); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "UpgradeDomain",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DomainName, TargetVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns the names of all Amazon OpenSearch Service domains owned by the current user in the active Region.
    Resource {
        operation: "ListDomainNames",
        items: "DomainNames", id: Id::Key(&["DomainName"]),
        scope: Scope::Region, tokens: None,
        query: &["DomainName", "EngineType"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("EngineType", "OpenSearch")],
    },
    // Lists all OpenSearch applications under your account.
    Resource {
        operation: "ListApplications",
        items: "ApplicationSummaries", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves all Amazon OpenSearch Service-managed VPC endpoints in the current Amazon Web Services account and Region.
    Resource {
        operation: "ListVpcEndpoints",
        items: "VpcEndpointSummaryList", id: Id::Key(&["VpcEndpointId"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["VpcEndpointId", "DomainArn", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists an inventory of all the direct query data sources that you have configured within Amazon OpenSearch Service.
    Resource {
        operation: "ListDirectQueryDataSources",
        items: "DirectQueryDataSources", id: Id::Arn("DataSourceArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["DataSourceArn", "DataSourceName"],
        tags: Some("TagList"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("opensearch", GROUPS, RESOURCES)
}
