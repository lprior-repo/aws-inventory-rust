pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ListVpcEndpointAccess",
        "Requires DomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about each principal that is allowed to access a given Amazon OpenSearch Service domain through the use of an interface VPC endpoint.",
    ),
    (
        "ListVpcEndpointsForDomain",
        "Requires DomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves all Amazon OpenSearch Service-managed VPC endpoints associated with a particular domain.",
    ),
    (
        "PurchaseReservedElasticsearchInstanceOffering",
        "Requires ReservedElasticsearchInstanceOfferingId, ReservationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to purchase reserved Elasticsearch instances.",
    ),
    (
        "RejectInboundCrossClusterSearchConnection",
        "Requires CrossClusterSearchConnectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows the destination domain owner to reject an inbound cross-cluster search connection request.",
    ),
    (
        "RemoveTags",
        "Requires ARN, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified set of tags from the specified Elasticsearch domain.",
    ),
    (
        "RevokeVpcEndpointAccess",
        "Requires DomainName, Account; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Revokes access to an Amazon OpenSearch Service domain that was provided through an interface VPC endpoint.",
    ),
    (
        "StartElasticsearchServiceSoftwareUpdate",
        "Requires DomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Schedules a service software update for an Amazon ES domain.",
    ),
    (
        "UpdateClone",
        "Requires DomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Manages a cloned blue/green upgrade that is in progress on a domain.",
    ),
    (
        "UpdateElasticsearchDomainConfig",
        "Requires DomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the cluster configuration of the specified Elasticsearch domain, setting as setting the instance type and the number of instances.",
    ),
    (
        "UpdatePackage",
        "Requires PackageID, PackageSource; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a package for use with Amazon ES domains.",
    ),
    (
        "UpdateVpcEndpoint",
        "Requires VpcEndpointId, VpcOptions; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an Amazon OpenSearch Service-managed interface VPC endpoint.",
    ),
    (
        "UpgradeElasticsearchDomain",
        "Requires DomainName, TargetVersion; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to either upgrade your domain or perform an Upgrade eligibility check to a compatible Elasticsearch version.",
    ),
];
