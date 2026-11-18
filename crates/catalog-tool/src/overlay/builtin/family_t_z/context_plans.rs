use super::{bind, inventory, name, path, token};
use crate::overlay::{
    Binding, ContextSource, Identity, IdentityField, NameError, OperationOverlay, Paginator, Scope,
    ServiceOverlay,
};

pub(super) fn apply(mut services: Vec<ServiceOverlay>) -> Result<Vec<ServiceOverlay>, NameError> {
    for service in &mut services {
        match service.service.as_str() {
            "transfer" => transfer(service)?,
            "workspaces-web" => certificates(service)?,
            _ => {}
        }
    }
    Ok(services)
}

fn contextual(operation: &str, parent: &str, field: &str) -> Result<Identity, NameError> {
    Ok(Identity::CorrelatedKey(vec![
        IdentityField::Context {
            operation: name(operation)?,
            source: ContextSource::Response,
            path: path(parent)?,
        },
        IdentityField::Selected(path(field)?),
    ]))
}

fn parent_context(
    plan: &mut OperationOverlay,
    input: &str,
    operation: &str,
    field: &str,
) -> Result<(), NameError> {
    plan.bind.insert(
        path(input)?,
        Binding::Context {
            operation: name(operation)?,
            source: ContextSource::Response,
            path: path(field)?,
        },
    );
    Ok(())
}

fn transfer(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut accesses = inventory(
        "Accesses",
        contextual("ListAccesses", "ServerId", "ExternalId")?,
        token("NextToken", "NextToken")?,
        Scope::Region,
    )?;
    bind(&mut accesses, "ServerId", "ListServers", "ServerId")?;
    service.operations.insert(name("ListAccesses")?, accesses);
    let mut access = inventory(
        "Access",
        contextual("DescribeAccess", "ServerId", "ExternalId")?,
        Paginator::None,
        Scope::Region,
    )?;
    bind(&mut access, "ExternalId", "ListAccesses", "ExternalId")?;
    parent_context(&mut access, "ServerId", "ListAccesses", "ServerId")?;
    service.operations.insert(name("DescribeAccess")?, access);
    for (operation, selected, parent, identifier) in [
        ("DescribeUser", "User", "ListUsers", "UserName"),
        ("DescribeHostKey", "HostKey", "ListHostKeys", "HostKeyId"),
    ] {
        let mut plan = inventory(
            selected,
            Identity::Arn(path("Arn")?),
            Paginator::None,
            Scope::Region,
        )?;
        bind(&mut plan, identifier, parent, identifier)?;
        parent_context(&mut plan, "ServerId", parent, "ServerId")?;
        plan.tags = Some(path("Tags")?);
        service.operations.insert(name(operation)?, plan);
    }
    Ok(())
}

fn certificates(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut list = inventory(
        "certificateList",
        contextual("ListTrustStoreCertificates", "trustStoreArn", "thumbprint")?,
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    bind(
        &mut list,
        "trustStoreArn",
        "ListTrustStores",
        "trustStoreArn",
    )?;
    service
        .operations
        .insert(name("ListTrustStoreCertificates")?, list);
    let mut detail = inventory(
        "certificate",
        contextual("GetTrustStoreCertificate", "trustStoreArn", "thumbprint")?,
        Paginator::None,
        Scope::Region,
    )?;
    bind(
        &mut detail,
        "thumbprint",
        "ListTrustStoreCertificates",
        "thumbprint",
    )?;
    parent_context(
        &mut detail,
        "trustStoreArn",
        "ListTrustStoreCertificates",
        "trustStoreArn",
    )?;
    service
        .operations
        .insert(name("GetTrustStoreCertificate")?, detail);
    Ok(())
}
