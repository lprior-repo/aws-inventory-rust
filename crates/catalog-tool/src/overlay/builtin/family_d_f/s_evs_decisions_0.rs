// Explicit dispositions from .local/aws-models/models/evs/service/2023-07-27/evs-2023-07-27.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AssociateEipToVlan",
        "Requires environmentId, vlanName, allocationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates an Elastic IP address with a public HCX VLAN.",
    ),
    (
        "CreateEntitlement",
        "Requires environmentId, connectorId, entitlementType, vmIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Windows Server License entitlement for virtual machines in an Amazon EVS environment using the provided vCenter Server connector.",
    ),
    (
        "CreateEnvironment",
        "Requires vpcId, serviceAccessSubnetId, vcfVersion, termsAccepted, initialVlans; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Amazon EVS environment that runs VCF software, such as SDDC Manager, NSX Manager, and vCenter Server.",
    ),
    (
        "CreateEnvironmentConnector",
        "Requires environmentId, type, applianceFqdn, secretIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a connector for an Amazon EVS environment.",
    ),
    (
        "CreateEnvironmentHost",
        "Requires environmentId, host; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an ESX host and adds it to an Amazon EVS environment.",
    ),
    (
        "DeleteEntitlement",
        "Requires environmentId, connectorId, entitlementType, vmIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a Windows Server License entitlement for virtual machines in an Amazon EVS environment.",
    ),
    (
        "DeleteEnvironment",
        "Requires environmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an Amazon EVS environment.",
    ),
    (
        "DeleteEnvironmentConnector",
        "Requires environmentId, connectorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a connector from an Amazon EVS environment.",
    ),
    (
        "DeleteEnvironmentHost",
        "Requires environmentId, hostName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a host from an Amazon EVS environment.",
    ),
    (
        "DisassociateEipFromVlan",
        "Requires environmentId, vlanName, associationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates an Elastic IP address from a public HCX VLAN.",
    ),
    (
        "GetAccountSettings",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns the configured EVS settings for your Amazon Web Services account in the specified Amazon Web Services Region.",
    ),
    (
        "GetDepotUrl",
        "Requires environmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a URL and authentication token for accessing the Amazon EVS Custom Addon depot.",
    ),
    (
        "GetEnvironment",
        "Requires environmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a description of the specified environment.",
    ),
    (
        "GetVersions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns information about VCF versions, ESX versions and EC2 instance types provided by Amazon EVS.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags for an Amazon EVS resource.",
    ),
    (
        "ListVmEntitlements",
        "Requires environmentId, connectorId, entitlementType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the Windows Server License entitlements for virtual machines in an Amazon EVS environment.",
    ),
    (
        "PutAccountSettings",
        "Requires settings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates account-level EVS settings for your Amazon Web Services account in the specified Amazon Web Services Region.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates the specified tags to an Amazon EVS resource with the specified resourceArn .",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes specified tags from an Amazon EVS resource.",
    ),
    (
        "UpdateEnvironmentConnector",
        "Requires environmentId, connectorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a connector for an Amazon EVS environment.",
    ),
];
