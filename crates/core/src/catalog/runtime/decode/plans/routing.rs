use super::{RuntimeCatalogError, metadata};
use crate::{
    catalog::runtime::{ExecutionBranch, RequestRouting},
    record::{FieldId, MapRef, ValueRef},
};

pub(super) fn read(fields: MapRef<'_>) -> Result<RequestRouting, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(15)?)? else {
        return Ok(RequestRouting::Regional);
    };
    let map = super::map(value)?;
    match metadata::integer(map, 1)? {
        0 => {
            metadata::only_fields(map, &[1])?;
            Ok(RequestRouting::Regional)
        }
        1 => {
            metadata::only_fields(map, &[1, 2])?;
            let region = metadata::text(map, 2)?;
            crate::inventory::RegionId::parse(&region)?;
            Ok(RequestRouting::FixedRegion { region })
        }
        2 => discovered(map),
        3 => {
            metadata::only_fields(map, &[1])?;
            Ok(RequestRouting::S3Outposts)
        }
        4 => {
            metadata::only_fields(map, &[1])?;
            Ok(RequestRouting::S3ExpressControl)
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}
fn discovered(map: MapRef<'_>) -> Result<RequestRouting, RuntimeCatalogError> {
    metadata::only_fields(map, &[1, 2, 3, 4, 5, 6])?;
    let operation = metadata::text(map, 2)?;
    crate::inventory::OperationId::parse(&operation)?;
    let trusted_dns_suffix = metadata::text(map, 6)?;
    if trusted_dns_suffix != ".timestream.{region}.amazonaws.com" {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(RequestRouting::Discovered {
        operation,
        endpoints_path: metadata::id_path(metadata::list(map, 3)?)?,
        address_path: metadata::id_path(metadata::list(map, 4)?)?,
        cache_minutes_path: metadata::id_path(metadata::list(map, 5)?)?,
        trusted_dns_suffix,
    })
}
pub(super) fn branches(fields: MapRef<'_>) -> Result<Box<[ExecutionBranch]>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(16)?)? else {
        return Ok(Box::default());
    };
    let ValueRef::List(list) = value else {
        return Err(RuntimeCatalogError::Capability);
    };
    let branches = metadata::bounded(list, branch, 16)?;
    let names = branches
        .iter()
        .map(|b| &b.discriminator)
        .collect::<std::collections::BTreeSet<_>>();
    if names.len() != branches.len() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(branches.into_boxed_slice())
}
fn branch(value: ValueRef<'_>) -> Result<ExecutionBranch, RuntimeCatalogError> {
    let map = super::map(value)?;
    metadata::only_fields(map, &[1, 2, 3, 4, 5, 6])?;
    let discriminator = metadata::text(map, 1)?;
    if !valid_branch_name(&discriminator) {
        return Err(RuntimeCatalogError::Capability);
    }
    let bindings = super::bindings(metadata::list(map, 2)?)?;
    let request_region = match map.get_field(FieldId::new(4)?)? {
        None => None,
        Some(ValueRef::Text(region)) => {
            crate::inventory::RegionId::parse(region)?;
            Some(region.to_owned())
        }
        _ => return Err(RuntimeCatalogError::Capability),
    };
    let emission = match map.get_field(FieldId::new(5)?)? {
        None => None,
        Some(ValueRef::Int(0)) => Some(crate::catalog::runtime::Emission::EmitMetadata),
        Some(ValueRef::Int(1)) => Some(crate::catalog::runtime::Emission::DiscoveryOnly),
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(ExecutionBranch {
        discriminator,
        bindings: bindings.into_boxed_slice(),
        emission,
        row_policy: map
            .get_field(FieldId::new(6)?)?
            .map(super::row_policy::read_value)
            .transpose()?,
        scope: super::scope(metadata::integer(map, 3)?)?,
        request_region,
    })
}

pub(super) fn validate(parts: &super::PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    use crate::catalog::runtime::BindingSource;
    match &parts.routing {
        RequestRouting::S3Outposts if parts.service_name.as_ref() != "s3-control" => {
            return Err(RuntimeCatalogError::Capability);
        }
        RequestRouting::S3ExpressControl
            if !matches!(parts.service_name.as_ref(), "s3" | "s3-control") =>
        {
            return Err(RuntimeCatalogError::Capability);
        }
        RequestRouting::Discovered { operation, .. }
            if operation != "DescribeEndpoints"
                || !matches!(
                    parts.service_name.as_ref(),
                    "timestream-write" | "timestream-query"
                ) =>
        {
            return Err(RuntimeCatalogError::Capability);
        }
        _ => {}
    }
    for branch in &parts.branches {
        for (index, binding) in branch.bindings.iter().enumerate() {
            let target =
                super::tags::resolve(parts.service, parts.operation.input()?, &binding.path)?;
            if let BindingSource::Fixed(value) = &binding.source {
                super::fixed_input::validate(parts.service, target, value)?;
            }
            let conflicts = parts
                .bindings
                .iter()
                .filter(|base| base.path != binding.path)
                .map(|binding| binding.path.as_ref())
                .chain(
                    parts
                        .pagination
                        .tokens
                        .iter()
                        .map(|token| token.request_path.as_ref()),
                )
                .chain(
                    parts
                        .pagination
                        .page_size
                        .iter()
                        .map(|size| size.path.as_ref()),
                )
                .chain(
                    branch
                        .bindings
                        .iter()
                        .take(index)
                        .map(|binding| binding.path.as_ref()),
                );
            if conflicts
                .into_iter()
                .any(|prior| binding.path.starts_with(prior) || prior.starts_with(&binding.path))
            {
                return Err(RuntimeCatalogError::Capability);
            }
        }
    }
    Ok(())
}

pub(super) fn validate_plans(plans: &[super::CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    for plan in plans {
        let RequestRouting::Discovered {
            operation,
            endpoints_path,
            address_path,
            cache_minutes_path,
            ..
        } = plan.routing()
        else {
            continue;
        };
        let discovery = plans
            .iter()
            .find(|candidate| {
                candidate.service_name() == plan.service_name()
                    && candidate.operation_name() == operation
            })
            .ok_or(RuntimeCatalogError::Capability)?;
        if !matches!(
            discovery.purpose(),
            crate::catalog::runtime::PlanPurpose::EndpointDiscovery
        ) || !matches!(discovery.routing(), RequestRouting::Regional)
            || !discovery.branches().is_empty()
            || endpoints_path.is_empty()
            || address_path.is_empty()
            || cache_minutes_path.is_empty()
        {
            return Err(RuntimeCatalogError::Capability);
        }
        let service = discovery.service();
        let target =
            super::tags::resolve(service, discovery.operation().output()?, endpoints_path)?;
        let list = super::tags::shape(service, target)?;
        if list.kind()? != "list" {
            return Err(RuntimeCatalogError::Capability);
        }
        let item = list.element()?.ok_or(RuntimeCatalogError::Capability)?;
        let address = super::tags::resolve(service, item, address_path)?;
        let cache = super::tags::resolve(service, item, cache_minutes_path)?;
        if !matches!(
            super::tags::target_kind(service, address)?,
            "string" | "enum"
        ) || !matches!(
            super::tags::target_kind(service, cache)?,
            "integer" | "long" | "short"
        ) {
            return Err(RuntimeCatalogError::Capability);
        }
    }
    Ok(())
}

pub(super) fn valid_branch_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Key, Record, RecordError, Value};
    use std::collections::BTreeMap;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn object<const N: usize>(entries: [(u32, Value); N]) -> Result<Value, RecordError> {
        let entries = entries
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<Result<BTreeMap<_, _>, RecordError>>()?;
        Ok(Value::Map(entries))
    }
    fn route(value: Value) -> Result<Record, RecordError> {
        Record::encode(&object([(15, value)])?)
    }
    fn variant(name: &str, bindings: Vec<Value>) -> Result<Value, RecordError> {
        object([
            (1, Value::Text(name.to_owned())),
            (2, Value::List(bindings)),
            (3, Value::Int(0)),
        ])
    }
    #[test]
    fn routing_metadata_rejects_unknown_fields_and_untrusted_discovery_suffix() -> TestResult {
        let record = route(object([
            (1, Value::Int(0)),
            (2, Value::Text("unexpected".to_owned())),
        ])?)?;
        assert!(read(record.root()).is_err());
        let record = route(object([
            (1, Value::Int(1)),
            (2, Value::Text(String::new())),
        ])?)?;
        assert!(read(record.root()).is_err());
        for suffix in [
            ".evil.example",
            "timestream.{region}.amazonaws.com",
            ".timestream.{region}.amazonaws.com.evil.example",
        ] {
            let record = route(object([
                (1, Value::Int(2)),
                (2, Value::Text("DescribeEndpoints".to_owned())),
                (3, Value::List(vec![Value::Int(1)])),
                (4, Value::List(vec![Value::Int(2)])),
                (5, Value::List(vec![Value::Int(3)])),
                (6, Value::Text(suffix.to_owned())),
            ])?)?;
            assert!(read(record.root()).is_err(), "{suffix}");
        }
        let record = route(object([
            (1, Value::Int(2)),
            (2, Value::Text("DescribeEndpoints".to_owned())),
            (3, Value::List(vec![Value::Int(1)])),
            (4, Value::List(vec![Value::Int(2)])),
            (5, Value::List(vec![Value::Int(3)])),
            (
                6,
                Value::Text(".timestream.{region}.amazonaws.com".to_owned()),
            ),
        ])?)?;
        assert!(matches!(
            read(record.root())?,
            RequestRouting::Discovered { .. }
        ));
        Ok(())
    }
    #[test]
    fn branch_metadata_rejects_duplicate_names_unknown_bindings_and_unbounded_variants()
    -> TestResult {
        let record = Record::encode(&object([(
            16,
            Value::List(vec![
                variant("ACCOUNT", vec![])?,
                variant("RESOURCE", vec![])?,
            ]),
        )])?)?;
        assert_eq!(branches(record.root())?.len(), 2);
        for values in [
            vec![variant("ACCOUNT", vec![])?, variant("ACCOUNT", vec![])?],
            vec![variant("../ACCOUNT", vec![])?],
            vec![variant(
                "ACCOUNT",
                vec![object([
                    (1, Value::List(vec![Value::Int(1)])),
                    (2, Value::Int(i64::MAX)),
                ])?],
            )?],
            (0..17)
                .map(|index| variant(&format!("branch{index}"), vec![]))
                .collect::<Result<Vec<_>, _>>()?,
        ] {
            let record = Record::encode(&object([(16, Value::List(values))])?)?;
            assert!(branches(record.root()).is_err());
        }
        Ok(())
    }
}
