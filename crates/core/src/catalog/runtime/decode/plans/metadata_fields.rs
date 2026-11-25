use super::*;

pub(super) fn endpoint(
    map_value: crate::record::MapRef<'_>,
) -> Result<EndpointTemplate, RuntimeCatalogError> {
    let url = metadata::text(map_value, 1)?;
    let signing_region = metadata::text(map_value, 2)?;
    let signing_name = metadata::text(map_value, 3)?;
    let class = match metadata::integer(map_value, 4)? {
        0 => EndpointClass::Global,
        1 => EndpointClass::Regional,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    let path_mode = match metadata::integer(map_value, 5)? {
        0 => EndpointPathMode::Standard,
        1 => EndpointPathMode::S3,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    let exceptions: Box<[EndpointException]> = match map_value
        .get_field(crate::record::FieldId::new(6)?)?
    {
        None | Some(ValueRef::Null) => Box::default(),
        Some(value) => {
            metadata::bounded(list_value(value)?, endpoint_exception, metadata::MAX_ITEMS)?.into()
        }
    };
    let mut regions = std::collections::BTreeSet::new();
    if !exceptions
        .iter()
        .all(|item| regions.insert(item.region.as_str()))
    {
        return Err(RuntimeCatalogError::Capability);
    }
    metadata::only_fields(map_value, &[1, 2, 3, 4, 5, 6])?;
    let regional_marker = url.contains("{region}") || signing_region.contains("{region}");
    if (!url.starts_with("https://") && !url.starts_with("http://"))
        || (class == EndpointClass::Regional) != regional_marker
    {
        return Err(RuntimeCatalogError::Endpoint);
    }
    Ok(EndpointTemplate {
        url,
        signing_region,
        signing_name,
        path_mode,
        class,
        exceptions,
    })
}

fn endpoint_exception(value: ValueRef<'_>) -> Result<EndpointException, RuntimeCatalogError> {
    let fields = map(value)?;
    let region = match metadata::text(fields, 1)?.as_str() {
        "aws-global" => "-".to_owned(),
        other => other.to_owned(),
    };
    let url = metadata::text(fields, 2)?;
    let signing_name = metadata::text(fields, 3)?;
    let signing_region = metadata::text(fields, 4)?;
    metadata::only_fields(fields, &[1, 2, 3, 4])?;
    let region_valid = region == "-" || crate::inventory::RegionId::parse(&region).is_ok();
    if !region_valid
        || !(url.starts_with("https://") || url.starts_with("http://"))
        || url.contains(['{', '}'])
    {
        return Err(RuntimeCatalogError::Endpoint);
    }
    Ok(EndpointException {
        region,
        url,
        signing_name,
        signing_region,
    })
}

pub(super) fn review(
    map_value: crate::record::MapRef<'_>,
) -> Result<ReviewProvenance, RuntimeCatalogError> {
    let review = ReviewProvenance {
        source: metadata::text(map_value, 1)?,
        reviewer: metadata::text(map_value, 2)?,
        reviewed_on: metadata::text(map_value, 3)?,
        tool_version: metadata::text(map_value, 4)?,
        evidence: ReviewEvidence {
            machine_reviewed: metadata::flag(map_value, 5)?,
            manually_reviewed: metadata::flag(map_value, 6)?,
            fixture_tested: metadata::flag(map_value, 7)?,
            emulator_tested: metadata::flag(map_value, 8)?,
        },
    };
    metadata::only_fields(map_value, &[1, 2, 3, 4, 5, 6, 7, 8])?;
    Ok(review)
}

pub(super) fn purpose(
    map_value: crate::record::MapRef<'_>,
) -> Result<PlanPurpose, RuntimeCatalogError> {
    match metadata::integer(map_value, 1)? {
        0 => {
            metadata::only_fields(map_value, &[1])?;
            Ok(PlanPurpose::Enumeration)
        }
        1 => {
            metadata::only_fields(map_value, &[1, 2])?;
            Ok(PlanPurpose::AccountIdentity {
                account_path: metadata::id_path(metadata::list(map_value, 2)?)?,
            })
        }
        2 => {
            metadata::only_fields(map_value, &[1, 2, 3, 4])?;
            Ok(PlanPurpose::RegionDiscovery {
                regions_path: metadata::id_path(metadata::list(map_value, 2)?)?,
                region_name_path: metadata::id_path(metadata::list(map_value, 3)?)?,
                opt_in_status_path: optional_path(map_value, 4)?,
            })
        }
        4 => {
            metadata::only_fields(map_value, &[1])?;
            Ok(PlanPurpose::EndpointDiscovery)
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn optional_path(
    map_value: crate::record::MapRef<'_>,
    key: u32,
) -> Result<Option<Box<[crate::record::FieldId]>>, RuntimeCatalogError> {
    match map_value.get_field(crate::record::FieldId::new(key)?)? {
        Some(ValueRef::Null) | None => Ok(None),
        Some(value) => Ok(Some(metadata::id_path(list_value(value)?)?)),
    }
}

pub(super) fn protocol(value: i64) -> Result<RuntimeProtocol, RuntimeCatalogError> {
    match value {
        0 => Ok(RuntimeProtocol::AwsJson10),
        1 => Ok(RuntimeProtocol::AwsJson11),
        2 => Ok(RuntimeProtocol::RpcV2Cbor),
        3 => Ok(RuntimeProtocol::AwsQuery),
        4 => Ok(RuntimeProtocol::Ec2Query),
        5 => Ok(RuntimeProtocol::RestJson1),
        6 => Ok(RuntimeProtocol::RestXml),
        _ => Err(RuntimeCatalogError::Capability),
    }
}
