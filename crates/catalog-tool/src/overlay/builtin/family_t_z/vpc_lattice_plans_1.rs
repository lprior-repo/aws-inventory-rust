use super::{inventory, path, paths, bind};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope};

pub(super) fn get_domain_verification() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "domainName",
        "status",
    ])?;
    operation.tags = Some(path("tags")?);
    bind(&mut operation, "domainVerificationIdentifier", "ListDomainVerifications", "id")?;
    Ok(operation)
}

pub(super) fn get_resource_configuration() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "resourceGatewayId",
        "resourceConfigurationGroupId",
        "type",
        "allowAssociationToShareableServiceNetwork",
        "protocol",
        "customDomainName",
        "status",
        "amazonManaged",
        "failureReason",
        "domainVerificationId",
        "domainVerificationArn",
        "domainVerificationStatus",
        "groupDomain",
    ])?;
    bind(&mut operation, "resourceConfigurationIdentifier", "ListResourceConfigurations", "id")?;
    Ok(operation)
}

pub(super) fn get_resource_gateway() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "name",
        "id",
        "arn",
        "status",
        "vpcId",
        "serviceManaged",
        "managedBy",
        "ipAddressType",
        "ipv4AddressesPerEni",
        "resourceConfigDnsResolution",
    ])?;
    bind(&mut operation, "resourceGatewayIdentifier", "ListResourceGateways", "id")?;
    Ok(operation)
}

pub(super) fn get_service() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "customDomainName",
        "certificateArn",
        "status",
        "authType",
        "idleTimeoutSeconds",
        "failureCode",
        "failureMessage",
    ])?;
    bind(&mut operation, "serviceIdentifier", "ListServices", "id")?;
    Ok(operation)
}

pub(super) fn get_service_network() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "authType",
        "numberOfAssociatedVPCs",
        "numberOfAssociatedServices",
    ])?;
    bind(&mut operation, "serviceNetworkIdentifier", "ListServiceNetworks", "id")?;
    Ok(operation)
}

pub(super) fn get_service_network_resource_association() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "status",
        "createdBy",
        "resourceConfigurationId",
        "resourceConfigurationArn",
        "resourceConfigurationName",
        "serviceNetworkId",
        "serviceNetworkArn",
        "serviceNetworkName",
        "failureReason",
        "failureCode",
        "privateDnsEnabled",
        "isManagedAssociation",
        "domainVerificationStatus",
    ])?;
    bind(&mut operation, "serviceNetworkResourceAssociationIdentifier", "ListServiceNetworkResourceAssociations", "id")?;
    Ok(operation)
}

pub(super) fn get_service_network_service_association() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "status",
        "arn",
        "createdBy",
        "serviceId",
        "serviceName",
        "serviceArn",
        "serviceNetworkId",
        "serviceNetworkName",
        "serviceNetworkArn",
        "customDomainName",
        "failureMessage",
        "failureCode",
    ])?;
    bind(&mut operation, "serviceNetworkServiceAssociationIdentifier", "ListServiceNetworkServiceAssociations", "id")?;
    Ok(operation)
}

pub(super) fn get_service_network_vpc_association() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "status",
        "arn",
        "createdBy",
        "serviceNetworkId",
        "serviceNetworkName",
        "serviceNetworkArn",
        "vpcId",
        "privateDnsEnabled",
        "failureMessage",
        "failureCode",
    ])?;
    bind(&mut operation, "serviceNetworkVpcAssociationIdentifier", "ListServiceNetworkVpcAssociations", "id")?;
    Ok(operation)
}

pub(super) fn get_target_group() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "$",
        Identity::Arn(path("arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "name",
        "type",
        "status",
        "failureMessage",
        "failureCode",
    ])?;
    bind(&mut operation, "targetGroupIdentifier", "ListTargetGroups", "id")?;
    Ok(operation)
}
