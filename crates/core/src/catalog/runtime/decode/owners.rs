use super::{RuntimeCatalogError, map, metadata};
use crate::{catalog::runtime::{ArnOwnerRule, ArnRegionRule, ReferenceOwnerRule}, record::{FieldId, MapRef}};

pub(super) fn read_owner(fields: MapRef<'_>) -> Result<ReferenceOwnerRule, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(8)?)? else {
        return Ok(ReferenceOwnerRule::ExecutionAccount);
    };
    let fields = map(value)?;
    metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
    if metadata::integer(fields, 1)? != 1 { return Err(RuntimeCatalogError::Capability); }
    let rule = ArnOwnerRule {
        partition: metadata::text(fields, 2)?,
        service: metadata::text(fields, 3)?,
        resource_prefix: metadata::text(fields, 4)?,
        region: match metadata::integer(fields, 5)? {
            0 => ArnRegionRule::Source,
            1 => ArnRegionRule::Global,
            _ => return Err(RuntimeCatalogError::Capability),
        },
    };
    if !matches!(rule.partition.as_str(), "aws" | "aws-cn" | "aws-us-gov")
        || !token(&rule.service) || rule.resource_prefix.len() > 128 {
        return Err(RuntimeCatalogError::Capability);
    }
    let prefix = rule.resource_prefix.strip_suffix('/').or_else(|| rule.resource_prefix.strip_suffix(':'))
        .ok_or(RuntimeCatalogError::Capability)?;
    if !token(prefix) { return Err(RuntimeCatalogError::Capability); }
    Ok(ReferenceOwnerRule::Arn(rule))
}
fn token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
        && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
