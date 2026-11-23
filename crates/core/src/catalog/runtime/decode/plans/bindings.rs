use super::{
    BindingPlan, BindingSource, RuntimeCatalogError, binding_alternatives, binding_value, map,
    metadata, routing, selection,
};
use crate::{
    catalog::runtime::{ArnResourceKind, ContextSource},
    record::{ListRef, MapRef, ValueRef},
};

pub(super) fn bindings(list: ListRef<'_>) -> Result<Vec<BindingPlan>, RuntimeCatalogError> {
    metadata::bounded(list, binding, 4096)
}

pub(super) fn binding(value: ValueRef<'_>) -> Result<BindingPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let path = metadata::id_path(metadata::list(fields, 1)?)?;
    let source_type = metadata::integer(fields, 2)?;
    let source = match source_type {
        0..=3 => basic(fields, source_type)?,
        4 => operation(fields)?,
        5..=7 => selection::binding(fields)?,
        8 => operation_arn(fields)?,
        9 => selection::transformed(fields)?,
        10 => selection::url_binding(fields)?,
        11 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::RegionList
        }
        12 => binding_alternatives::decode(fields)?,
        13 => context_from_branch(fields)?,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingPlan { path, source })
}

fn basic(fields: MapRef<'_>, source_type: i64) -> Result<BindingSource, RuntimeCatalogError> {
    match source_type {
        0..=1 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(match source_type {
                0 => BindingSource::Account,
                _ => BindingSource::Region,
            })
        }
        2 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            Ok(BindingSource::Fixed(binding_value(metadata::field(fields, 3)?)?))
        }
        3 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            Ok(BindingSource::ParentIdentity {
                path: metadata::selected_path(metadata::list(fields, 3)?)?,
            })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn operation(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
    Ok(BindingSource::Operation {
        service: metadata::text(fields, 3)?.into_boxed_str(),
        operation: metadata::text(fields, 4)?.into_boxed_str(),
        path: metadata::selected_path(metadata::list(fields, 5)?)?,
    })
}

fn operation_arn(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
    let arn_service = metadata::text(fields, 6)?;
    let resource_kind = match (arn_service.as_str(), metadata::integer(fields, 7)?) {
        ("sns", 0) => ArnResourceKind::SnsSubscription,
        ("sqs", 1) => ArnResourceKind::SqsQueue,
        ("sns", 2) => ArnResourceKind::SnsTopic,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingSource::OperationArn {
        service: metadata::text(fields, 3)?.into_boxed_str(),
        operation: metadata::text(fields, 4)?.into_boxed_str(),
        path: metadata::selected_path(metadata::list(fields, 5)?)?,
        arn_service: arn_service.into_boxed_str(),
        resource_kind,
    })
}

fn context_from_branch(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
    let branch = metadata::text(fields, 6)?;
    if !routing::valid_branch_name(&branch) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(BindingSource::ContextFromBranch {
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
    })
}
