use super::{tags, BindingSource, BindingValue, IdentityPart, IdentityPlan, PlanParts,
    PlanPurpose, RuntimeCatalogError, RuntimeProtocol, TokenPlacement, TokenType};
use crate::{catalog::{HttpBinding, Member, Service, Target}, record::FieldId};

pub(super) fn validate(parts: &PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    let item = tags::selected_item(parts.service, parts.operation, &parts.items)?;
    if parts.items != parts.pagination.items {
        return Err(RuntimeCatalogError::Capability);
    }
    super::selection::validate_projection(parts)?;
    super::routing::validate(parts)?;
    super::row_policy::validate(parts)?;
    if !matches!(parts.purpose, PlanPurpose::TagEndpoint(_) | PlanPurpose::EndpointDiscovery)
        && parts.emission == super::super::super::Emission::EmitMetadata {
        identity(parts, item)?;
    }
    parts.bindings.iter().chain(parts.branches.iter().flat_map(|branch| branch.bindings.iter())).try_for_each(|binding| {
        let target = tags::resolve(parts.service, parts.operation.input()?, &binding.path)?;
        match &binding.source {
            BindingSource::Account | BindingSource::Region => text(parts.service, target),
            BindingSource::RegionList => {
                let list = tags::shape(parts.service, target)?;
                if list.kind()? != "list" { return Err(RuntimeCatalogError::Capability); }
                text(parts.service, list.element()?.ok_or(RuntimeCatalogError::Capability)?)
            }
            BindingSource::Fixed(value) => fixed(parts.service, target, value),
            BindingSource::Transformed { .. } => text(parts.service, target),
            BindingSource::ParentIdentity { .. } | BindingSource::Operation { .. }
                | BindingSource::OperationArn { .. }
                | BindingSource::OperationUrl { .. }
                | BindingSource::Context { .. } | BindingSource::Alternatives(_) | BindingSource::ContextFromBranch { .. } => Ok(()),
        }
    })?;
    destinations(parts)?;
    pagination(parts)?;
    purpose(parts)
}

fn selected(service: Service<'_>, item: Target, path: &[FieldId]) -> Result<Target, RuntimeCatalogError> {
    let target = tags::resolve(service, item, path)?;
    if path.is_empty() { scalar(service, target)?; }
    Ok(target)
}

fn identity(parts: &PlanParts<'_>, item: Target) -> Result<(), RuntimeCatalogError> {
    let service = parts.service;
    match &parts.identity {
        IdentityPlan::Arn(path) => text(service, selected(service, item, path)?),
        IdentityPlan::Key(paths) => paths.iter().try_for_each(|path| scalar(service, selected(service, item, path)?)),
        IdentityPlan::Template(parts) => parts.iter().try_for_each(|part| match part {
            IdentityPart::Field(path) => scalar(service, selected(service, item, path)?),
            _ => Ok(()),
        }),
        IdentityPlan::Choice(_) | IdentityPlan::CorrelatedKey(_) | IdentityPlan::MapArn { .. } =>
            super::selection::validate_identity(parts),
    }
}

fn text(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    if matches!(tags::target_kind(service, target)?, "string" | "enum") { Ok(()) }
    else { Err(RuntimeCatalogError::Capability) }
}

fn scalar(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    if matches!(tags::target_kind(service, target)?, "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean") { Ok(()) }
    else { Err(RuntimeCatalogError::Capability) }
}

fn fixed(service: Service<'_>, target: Target, value: &BindingValue) -> Result<(), RuntimeCatalogError> {
    super::fixed_input::validate(service, target, value)
}

fn destinations(parts: &PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    let paths = parts.bindings.iter().map(|binding| binding.path.as_ref())
        .chain(parts.pagination.tokens.iter().map(|token| token.request_path.as_ref()))
        .chain(parts.pagination.page_size.iter().map(|size| size.path.as_ref()))
        .collect::<Vec<_>>();
    if paths.iter().enumerate().any(|(index, path)| paths.iter().take(index)
        .any(|prior| path.starts_with(prior) || prior.starts_with(path))) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

fn terminal<'a>(service: Service<'a>, target: Target, path: &[FieldId]) -> Result<Member<'a>, RuntimeCatalogError> {
    let (last, prefix) = path.split_last().ok_or(RuntimeCatalogError::Capability)?;
    tags::member(tags::shape(service, tags::resolve(service, target, prefix)?)?, *last)
}

fn pagination(parts: &PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    parts.pagination.tokens.iter().try_for_each(|token| {
        let request = terminal(parts.service, parts.operation.input()?, &token.request_path)?;
        let response = terminal(parts.service, parts.operation.output()?, &token.response_path)?;
        token_kind(parts.service, request.target, token.value_type)?;
        token_kind(parts.service, response.target, token.value_type)?;
        placement(request, &token.request_path, &token.request_placement, parts.protocol, false)?;
        placement(response, &token.response_path, &token.response_placement, parts.protocol, true)
    })?;
    if let Some(size) = &parts.pagination.page_size {
        let member = terminal(parts.service, parts.operation.input()?, &size.path)?;
        let value_type = match size.value_type {
            crate::catalog::runtime::PageSizeType::String => TokenType::String,
            crate::catalog::runtime::PageSizeType::Integer => TokenType::Integer,
        };
        token_kind(parts.service, member.target, value_type)?;
        if size.max == 0 || parts.pagination.max_page_size.is_some_and(|max| size.max > max) {
            return Err(RuntimeCatalogError::Capability);
        }
        placement(member, &size.path, &size.placement, parts.protocol, false)?;
        fixed(parts.service, member.target, &match size.value_type {
            crate::catalog::runtime::PageSizeType::String => BindingValue::String(size.max.to_string()),
            crate::catalog::runtime::PageSizeType::Integer => BindingValue::Integer(i64::from(size.max)),
        })?;
    }
    if let Some(path) = &parts.pagination.truncated_path {
        token_kind(parts.service, tags::resolve(parts.service, parts.operation.output()?, path)?, TokenType::Boolean)?;
    }
    Ok(())
}

fn token_kind(service: Service<'_>, target: Target, value_type: TokenType) -> Result<(), RuntimeCatalogError> {
    let kind = tags::target_kind(service, target)?;
    let valid = match value_type {
        TokenType::String => matches!(kind, "string" | "enum"),
        TokenType::Integer => matches!(kind, "byte" | "short" | "integer" | "long"),
        TokenType::Boolean => kind == "boolean",
    };
    if valid { Ok(()) } else { Err(RuntimeCatalogError::Capability) }
}

fn placement(member: Member<'_>, path: &[FieldId], placement: &TokenPlacement, protocol: RuntimeProtocol, response: bool) -> Result<(), RuntimeCatalogError> {
    let wire = member.wire()?;
    let rest = matches!(protocol, RuntimeProtocol::RestJson1 | RuntimeProtocol::RestXml);
    let query = matches!(protocol, RuntimeProtocol::AwsQuery | RuntimeProtocol::Ec2Query);
    let valid = match (wire.binding()?, placement) {
        (HttpBinding::Document, TokenPlacement::Body(body)) => body.as_ref() == path && (response || !query || path.len() > 1),
        (HttpBinding::Payload, TokenPlacement::Body(body)) => rest && path.len() == 1 && body.as_ref() == path,
        (HttpBinding::Header(name), TokenPlacement::Header(actual)) => rest && path.len() == 1 && name == actual,
        (HttpBinding::Query(name), TokenPlacement::Query(actual)) => !response && rest && path.len() == 1 && name == actual,
        (HttpBinding::Label, TokenPlacement::Path(actual)) => !response && rest && path.len() == 1 && member.name()? == actual,
        (HttpBinding::Document, TokenPlacement::Query(actual)) if !response && query && path.len() == 1 => {
            let modeled = if protocol == RuntimeProtocol::Ec2Query { wire.ec2_query_name()? } else { wire.xml_name()? };
            modeled.map_or_else(|| member.name().map(|name| name == actual), |name| Ok(name == actual))?
        }
        _ => false,
    };
    if valid { Ok(()) } else { Err(RuntimeCatalogError::Capability) }
}

fn purpose(parts: &PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    match &parts.purpose {
        PlanPurpose::Enumeration | PlanPurpose::EndpointDiscovery => Ok(()),
        PlanPurpose::TagEndpoint(_) => super::tag_endpoint::validate_tag(parts),
        PlanPurpose::AccountIdentity { account_path } => text(parts.service, tags::resolve(parts.service, parts.operation.output()?, account_path)?),
        PlanPurpose::RegionDiscovery { regions_path, region_name_path, opt_in_status_path } => {
            let list = tags::shape(parts.service, tags::resolve(parts.service, parts.operation.output()?, regions_path)?)?;
            if list.kind()? != "list" { return Err(RuntimeCatalogError::Capability); }
            let item = list.element()?.ok_or(RuntimeCatalogError::Capability)?;
            text(parts.service, tags::resolve(parts.service, item, region_name_path)?)?;
            opt_in_status_path.as_ref().map_or(Ok(()), |path| text(parts.service, tags::resolve(parts.service, item, path)?))
        }
    }
}

pub(super) fn source_compatible(source: &super::CallablePlan<'_>, path: &[FieldId], child: &super::CallablePlan<'_>, destination: &[FieldId]) -> Result<(), RuntimeCatalogError> {
    let item = tags::selected_item(source.service(), source.operation(), source.items())?;
    let source_target = selected(source.service(), item, path)?;
    let target = tags::resolve(child.service(), child.operation().input()?, destination)?;
    let source_kind = tags::target_kind(source.service(), source_target)?;
    let target_kind = tags::target_kind(child.service(), target)?;
    if source_kind == target_kind || matches!((source_kind, target_kind), ("string", "enum") | ("enum", "string")) { Ok(()) }
    else { Err(RuntimeCatalogError::Capability) }
}

pub(super) fn exclusions(service: Service<'_>, operation: crate::catalog::Operation<'_>, items: &super::ItemPlan, paths: crate::record::ListRef<'_>) -> Result<(), RuntimeCatalogError> {
    let item = tags::selected_item(service, operation, items)?;
    super::metadata::bounded(paths, |value| {
        let path = super::metadata::id_path(super::list_value(value)?)?;
        tags::resolve(service, item, &path).map(|_| ())
    }, 4096).map(|_| ())
}
