use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn describe_agreement() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Agreement",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "AgreementId",
        "ServerId",
        "LocalProfileId",
        "PartnerProfileId",
        "Description",
        "Status",
        "BaseDirectory",
        "AccessRole",
        "PreserveFilename",
        "EnforceMessageSigning",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(
        &mut operation,
        "AgreementId",
        "ListAgreements",
        "AgreementId",
    )?;
    bind(&mut operation, "ServerId", "ListAgreements", "ServerId")?;
    Ok(operation)
}

pub(super) fn describe_certificate() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Certificate",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "CertificateId",
        "Usage",
        "Status",
        "Certificate",
        "CertificateChain",
        "Serial",
        "Type",
        "Description",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(
        &mut operation,
        "CertificateId",
        "ListCertificates",
        "CertificateId",
    )?;
    Ok(operation)
}

pub(super) fn describe_connector() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Connector",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "ConnectorId",
        "Url",
        "AccessRole",
        "LoggingRole",
        "SecurityPolicyName",
        "EgressType",
        "ErrorMessage",
        "Status",
        "IpAddressType",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(
        &mut operation,
        "ConnectorId",
        "ListConnectors",
        "ConnectorId",
    )?;
    Ok(operation)
}

pub(super) fn describe_profile() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Profile",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["Arn", "ProfileId", "ProfileType", "As2Id"])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "ProfileId", "ListProfiles", "ProfileId")?;
    Ok(operation)
}

pub(super) fn describe_server() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Server",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "Certificate",
        "Domain",
        "EndpointType",
        "HostKeyFingerprint",
        "IdentityProviderType",
        "LoggingRole",
        "PostAuthenticationLoginBanner",
        "PreAuthenticationLoginBanner",
        "SecurityPolicyName",
        "ServerId",
        "State",
        "UserCount",
        "IpAddressType",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "ServerId", "ListServers", "ServerId")?;
    Ok(operation)
}

pub(super) fn describe_web_app() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WebApp",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "WebAppId",
        "AccessEndpoint",
        "WebAppEndpoint",
        "WebAppEndpointPolicy",
        "EndpointType",
    ])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "WebAppId", "ListWebApps", "WebAppId")?;
    Ok(operation)
}

pub(super) fn describe_workflow() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Workflow",
        Identity::Arn(path("Arn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&["Arn", "Description", "WorkflowId"])?;
    operation.tags = Some(path("Tags")?);
    bind(&mut operation, "WorkflowId", "ListWorkflows", "WorkflowId")?;
    Ok(operation)
}

pub(super) fn list_agreements() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Agreements",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "AgreementId",
        "Description",
        "Status",
        "ServerId",
        "LocalProfileId",
        "PartnerProfileId",
    ])?;
    operation.page_size = Some(100);
    bind(&mut operation, "ServerId", "ListServers", "ServerId")?;
    Ok(operation)
}

pub(super) fn list_certificates() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Certificates",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "CertificateId",
        "Usage",
        "Status",
        "Type",
        "Description",
    ])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_connectors() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Connectors",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["Arn", "ConnectorId", "Url"])?;
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_host_keys() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "HostKeys",
        Identity::Arn(path("Arn")?),
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "Arn",
        "HostKeyId",
        "Fingerprint",
        "Description",
        "Type",
        "StorageType",
        "SecretId",
    ])?;
    bind(&mut operation, "ServerId", "ListServers", "ServerId")?;
    Ok(operation)
}
