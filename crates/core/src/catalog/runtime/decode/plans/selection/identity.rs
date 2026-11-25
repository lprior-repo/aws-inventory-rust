use super::super::{RuntimeCatalogError, map, metadata};
use crate::{
    catalog::runtime::{
        IdentityBranchPlan, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan,
        IdentityPredicatePlan,
    },
    record::{MapRef, ValueRef},
};

pub(in crate::catalog::runtime::decode::plans) fn read(
    fields: MapRef<'_>,
) -> Result<IdentityPlan, RuntimeCatalogError> {
    let tag = metadata::integer(fields, 1)?;
    if tag <= 2 {
        return super::super::identity(fields);
    }
    match tag {
        3 => {
            metadata::only_fields(fields, &[1, 2, 3, 4])?;
            let review = bounded_text(fields, 4, 1024)?;
            Ok(IdentityPlan::Choice(IdentityChoicePlan::PreferArn {
                path: metadata::selected_path(metadata::list(fields, 2)?)?,
                fallback: leaf(map(metadata::field(fields, 3)?)?)?,
                review,
            }))
        }
        4 => {
            metadata::only_fields(fields, &[1, 2])?;
            let branches = metadata::bounded(metadata::list(fields, 2)?, branch, 16)?;
            if branches.is_empty() {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(IdentityPlan::Choice(IdentityChoicePlan::Discriminated(
                branches.into_boxed_slice(),
            )))
        }
        5 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(IdentityPlan::CorrelatedKey(fields_list(fields)?))
        }
        6 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
            let resource_kind = super::arn_kind(metadata::integer(fields, 4)?)?;
            let request_url = fields
                .get_field(crate::record::FieldId::new(5)?)?
                .map(|value| {
                    super::super::super::list_value(value).and_then(metadata::selected_path)
                })
                .transpose()?;
            if matches!(
                resource_kind,
                crate::catalog::runtime::ArnResourceKind::SqsQueue
            ) != request_url.is_some()
            {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(IdentityPlan::MapArn {
                path: metadata::selected_path(metadata::list(fields, 2)?)?,
                key: bounded_text(fields, 3, 4096)?,
                resource_kind,
                request_url,
            })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn leaf(fields: MapRef<'_>) -> Result<IdentityLeafPlan, RuntimeCatalogError> {
    match metadata::integer(fields, 1)? {
        0..=2 => match super::super::identity(fields)? {
            IdentityPlan::Arn(path) => Ok(IdentityLeafPlan::Arn(path)),
            IdentityPlan::Template(parts) => Ok(IdentityLeafPlan::Template(parts)),
            IdentityPlan::Key(paths) => Ok(IdentityLeafPlan::Key(paths)),
            _ => Err(RuntimeCatalogError::Capability),
        },
        5 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(IdentityLeafPlan::Fields(fields_list(fields)?))
        }
        7 => {
            metadata::only_fields(fields, &[1, 2, 3, 4])?;
            let resource_kind = super::arn_kind(metadata::integer(fields, 4)?)?;
            if matches!(
                resource_kind,
                crate::catalog::runtime::ArnResourceKind::SqsQueue
            ) {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(IdentityLeafPlan::OwnedArn {
                path: metadata::selected_path(metadata::list(fields, 2)?)?,
                owner: metadata::selected_path(metadata::list(fields, 3)?)?,
                resource_kind,
            })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn branch(value: ValueRef<'_>) -> Result<IdentityBranchPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    metadata::only_fields(fields, &[1, 2])?;
    Ok(IdentityBranchPlan {
        predicate: predicate(map(metadata::field(fields, 1)?)?)?,
        identity: leaf(map(metadata::field(fields, 2)?)?)?,
    })
}

fn predicate(fields: MapRef<'_>) -> Result<IdentityPredicatePlan, RuntimeCatalogError> {
    let tag = metadata::integer(fields, 1)?;
    metadata::only_fields(fields, if tag == 0 { &[1, 2] } else { &[1, 2, 3] })?;
    let path = metadata::selected_path(metadata::list(fields, 2)?)?;
    match tag {
        0 => Ok(IdentityPredicatePlan::Present(path)),
        1 => Ok(IdentityPredicatePlan::TextEquals {
            path,
            value: bounded_text(fields, 3, 4096)?,
        }),
        2 => Ok(IdentityPredicatePlan::RequestTextEquals {
            path,
            value: bounded_text(fields, 3, 4096)?,
        }),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn fields_list(fields: MapRef<'_>) -> Result<Box<[IdentityFieldPlan]>, RuntimeCatalogError> {
    let fields = metadata::bounded(metadata::list(fields, 2)?, field, 64)?;
    if fields.is_empty() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(fields.into_boxed_slice())
}

fn field(value: ValueRef<'_>) -> Result<IdentityFieldPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let tag = metadata::integer(fields, 1)?;
    match tag {
        0 | 1 => {
            metadata::only_fields(fields, &[1, 2])?;
            let path = metadata::selected_path(metadata::list(fields, 2)?)?;
            Ok(if tag == 0 {
                IdentityFieldPlan::Selected(path)
            } else {
                IdentityFieldPlan::SelectedDigest(path)
            })
        }
        2 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
            let service = metadata::text(fields, 2)?;
            let operation = metadata::text(fields, 3)?;
            crate::inventory::ServiceId::parse(&service)?;
            crate::inventory::OperationId::parse(&operation)?;
            Ok(IdentityFieldPlan::Context {
                service: service.into_boxed_str(),
                operation: operation.into_boxed_str(),
                source: super::source(metadata::integer(fields, 4)?)?,
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
            })
        }
        3 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            Ok(IdentityFieldPlan::MapValue {
                path: metadata::selected_path(metadata::list(fields, 2)?)?,
                key: bounded_text(fields, 3, 4096)?,
            })
        }
        4 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(IdentityFieldPlan::Request(metadata::selected_path(
                metadata::list(fields, 2)?,
            )?))
        }
        8 | 9 => {
            metadata::only_fields(fields, &[1, 2])?;
            let path = metadata::selected_path(metadata::list(fields, 2)?)?;
            Ok(if tag == 8 {
                IdentityFieldPlan::SelectedArn(path)
            } else {
                IdentityFieldPlan::RequestArn(path)
            })
        }
        5..=7 => {
            metadata::only_fields(fields, &[1, 2, 3, 4])?;
            let path = metadata::selected_path(metadata::list(fields, 2)?)?;
            let lower = metadata::integer(fields, 3)?;
            let upper = metadata::integer(fields, 4)?;
            match tag {
                5 | 6 if lower <= upper => Ok(if tag == 5 {
                    IdentityFieldPlan::SelectedInteger {
                        path,
                        minimum: lower,
                        maximum: upper,
                    }
                } else {
                    IdentityFieldPlan::RequestInteger {
                        path,
                        minimum: lower,
                        maximum: upper,
                    }
                }),
                7 if (1..=64).contains(&lower) && (1..=4096).contains(&upper) => {
                    Ok(IdentityFieldPlan::SelectedStringList {
                        path,
                        max_items: u32::try_from(lower).map_err(|_| RuntimeCatalogError::Bounds)?,
                        max_item_bytes: u32::try_from(upper)
                            .map_err(|_| RuntimeCatalogError::Bounds)?,
                    })
                }
                _ => Err(RuntimeCatalogError::Bounds),
            }
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn bounded_text(
    fields: MapRef<'_>,
    field: u32,
    bound: usize,
) -> Result<String, RuntimeCatalogError> {
    let text = metadata::text(fields, field)?;
    if text.is_empty() || text.len() > bound || text.chars().any(char::is_control) {
        return Err(RuntimeCatalogError::Bounds);
    }
    Ok(text)
}
