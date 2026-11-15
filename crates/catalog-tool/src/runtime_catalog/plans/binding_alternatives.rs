use super::{RuntimeCatalogCompileError, encode, selected_path};
use crate::overlay::{BindingAlternative, ContextSource, MemberPath};
use inventory_core::record::Value;

pub(super) fn encode(
    service: &str, path: &MemberPath, alternatives: &[BindingAlternative],
    services: &crate::runtime_catalog::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if alternatives.is_empty() || alternatives.len() > 8
        || alternatives.iter().enumerate().any(|(index, alternative)| alternatives.iter().take(index)
            .any(|prior| prior.operation == alternative.operation)) {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let entries = alternatives.iter().map(|alternative| {
        let source = match alternative.source {
            ContextSource::SelectedItem => 0,
            ContextSource::Response => 1,
            ContextSource::Request => 2,
        };
        encode::map([
            (1, Value::Text(service.to_owned())),
            (2, Value::Text(alternative.operation.to_string())),
            (3, Value::Int(source)),
            (4, selected_path(services, alternative.path.as_str())?),
        ])
    }).collect::<Result<Vec<_>, _>>()?;
    encode::map([
        (1, encode::path(services, path.as_str())?),
        (2, Value::Int(12)),
        (3, Value::List(entries)),
    ])
}
