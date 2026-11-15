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
    metadata::only_fields(map_value, &[1, 2, 3, 4, 5])?;
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
    })
}

pub(super) fn review(
    map_value: crate::record::MapRef<'_>,
) -> Result<ReviewProvenance, RuntimeCatalogError> {
    let review = ReviewProvenance {
        source: metadata::text(map_value, 1)?,
        reviewer: metadata::text(map_value, 2)?,
        reviewed_on: metadata::text(map_value, 3)?,
    };
    metadata::only_fields(map_value, &[1, 2, 3])?;
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
