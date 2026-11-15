use super::{BindingSource, RuntimeCatalogError, map, metadata};
use crate::{catalog::runtime::{BindingAlternativePlan, ContextSource}, record::{MapRef, ValueRef}};

pub(super) fn decode(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3])?;
    let alternatives = metadata::bounded(metadata::list(fields, 3)?, alternative, 8)?;
    if alternatives.is_empty() || alternatives.iter().enumerate().any(|(index, alternative)|
        alternatives.iter().take(index).any(|prior| prior.operation == alternative.operation)) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(BindingSource::Alternatives(alternatives.into_boxed_slice()))
}

fn alternative(value: ValueRef<'_>) -> Result<BindingAlternativePlan, RuntimeCatalogError> {
    let fields = map(value)?;
    metadata::only_fields(fields, &[1, 2, 3, 4])?;
    let source = match metadata::integer(fields, 3)? {
        0 => ContextSource::SelectedItem,
        1 => ContextSource::Response,
        2 => ContextSource::Request,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingAlternativePlan {
        service: metadata::text(fields, 1)?.into_boxed_str(),
        operation: metadata::text(fields, 2)?.into_boxed_str(),
        source,
        path: metadata::selected_path(metadata::list(fields, 4)?)?,
    })
}

#[cfg(test)]
mod tests;
