use super::{BindingPlan, BindingSource, BindingValue, RuntimeCatalogError, map, metadata};
use crate::{
    catalog::runtime::{ArnResourceKind, ContextSource},
    record::ValueRef,
};
use super::{binding_alternatives, fixed_input, routing, selection};

pub fn read(list: crate::record::ListRef<'_>) -> Result<Vec<BindingPlan>, RuntimeCatalogError> {
    metadata::bounded(list, binding, 4096)
}

pub fn binding(value: ValueRef<'_>) -> Result<BindingPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let path = metadata::id_path(metadata::list(fields, 1)?)?;
    let source_type = metadata::integer(fields, 2)?;
    let source = match source_type {
        0 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::Account
        }
        1 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::Region
        }
        2 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            BindingSource::Fixed(decode_value(metadata::field(fields, 3)?)?)
        }
        3 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            BindingSource::ParentIdentity {
                path: metadata::selected_path(metadata::list(fields, 3)?)?,
            }
        }
        4 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
            BindingSource::Operation {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
            }
        }
        5..=7 => selection::binding(fields)?,
        8 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
            let arn_service = metadata::text(fields, 6)?;
            let resource_kind = match (arn_service.as_str(), metadata::integer(fields, 7)?) {
                ("sns", 0) => ArnResourceKind::SnsSubscription,
                ("sqs", 1) => ArnResourceKind::SqsQueue,
                ("sns", 2) => ArnResourceKind::SnsTopic,
                _ => return Err(RuntimeCatalogError::Capability),
            };
            BindingSource::OperationArn {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
                arn_service: arn_service.into_boxed_str(),
                resource_kind,
            }
        }
        9 => selection::transformed(fields)?,
        10 => selection::url_binding(fields)?,
        11 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::RegionList
        }
        12 => binding_alternatives::decode(fields)?,
        13 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
            let branch = metadata::text(fields, 6)?;
            if !routing::valid_branch_name(&branch) {
                return Err(RuntimeCatalogError::Capability);
            }
            BindingSource::ContextFromBranch {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                branch: branch.into_boxed_str(),
                source: match metadata::integer(fields, 7)? {
                    5 => ContextSource::SelectedItem,
                    6 => ContextSource::Response,
                    7 => ContextSource::Request,
                    _ => return Err(RuntimeCatalogError::Capability),
                },
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
            }
        }
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingPlan { path, source })
}

fn decode_value(value: ValueRef<'_>) -> Result<BindingValue, RuntimeCatalogError> {
    fixed_input::decode(value)
}
