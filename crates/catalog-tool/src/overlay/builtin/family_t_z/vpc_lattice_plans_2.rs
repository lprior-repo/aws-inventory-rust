use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_domain_verifications() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["id", "arn", "domainName", "status"])?;
    operation.tags = Some(path("tags")?);
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_listeners() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["arn", "id", "name", "protocol", "port"])?;
    operation.page_size = Some(100);
    bind(&mut operation, "serviceIdentifier", "ListServices", "id")?;
    Ok(operation)
}

pub(super) fn list_resource_configurations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "resourceGatewayId",
        "resourceConfigurationGroupId",
        "type",
        "status",
        "amazonManaged",
        "customDomainName",
        "domainVerificationId",
        "groupDomain",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_resource_endpoint_associations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "resourceConfigurationId",
        "resourceConfigurationArn",
        "resourceConfigurationName",
        "vpcEndpointId",
        "vpcEndpointOwner",
        "createdBy",
    ])?;
    operation.page_size = Some(100);
    bind(
        &mut operation,
        "resourceConfigurationIdentifier",
        "ListResourceConfigurations",
        "id",
    )?;
    Ok(operation)
}

pub(super) fn list_resource_gateways() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "name",
        "id",
        "arn",
        "status",
        "vpcIdentifier",
        "ipAddressType",
        "ipv4AddressesPerEni",
        "resourceConfigDnsResolution",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_service_network_resource_associations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
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
        "isManagedAssociation",
        "failureCode",
        "privateDnsEnabled",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_service_networks() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "name",
        "arn",
        "numberOfAssociatedVPCs",
        "numberOfAssociatedServices",
        "numberOfAssociatedResourceConfigurations",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_service_network_service_associations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
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
    ])?;
    operation.page_size = Some(100);
    bind(
        &mut operation,
        "serviceNetworkIdentifier",
        "ListServiceNetworks",
        "id",
    )?;
    Ok(operation)
}

pub(super) fn list_service_network_vpc_associations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "id",
        "arn",
        "status",
        "createdBy",
        "serviceNetworkId",
        "serviceNetworkName",
        "serviceNetworkArn",
        "privateDnsEnabled",
        "vpcId",
    ])?;
    operation.page_size = Some(100);
    bind(
        &mut operation,
        "serviceNetworkIdentifier",
        "ListServiceNetworks",
        "id",
    )?;
    Ok(operation)
}

pub(super) fn list_service_network_vpc_endpoint_associations() -> Result<OperationOverlay, NameError>
{
    let mut operation = inventory(
        "items",
        Identity::Key(paths(&["serviceNetworkArn", "id"])?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "vpcEndpointId",
        "vpcId",
        "vpcEndpointOwnerId",
        "id",
        "state",
        "serviceNetworkArn",
    ])?;
    operation.page_size = Some(100);
    bind(
        &mut operation,
        "serviceNetworkIdentifier",
        "ListServiceNetworks",
        "id",
    )?;
    Ok(operation)
}

pub(super) fn list_services() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "items",
        Identity::Arn(path("arn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["id", "name", "arn", "customDomainName", "status"])?;
    operation.page_size = Some(100);
    Ok(operation)
}
