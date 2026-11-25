use crate::overlay::{ContextSource, Identity, IdentityChoice, IdentityField, IdentityLeaf};
use crate::runtime_catalog::RuntimeCatalogCompileError;

pub(super) fn validate(
    identity: &Identity,
    operation: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    let fields = |fields: &[IdentityField]| {
        fields.iter().any(|field|
        matches!(field, IdentityField::Context { operation: producer, source: ContextSource::SelectedItem, .. }
            if producer.as_str() == operation))
    };
    let leaf = |leaf: &IdentityLeaf| matches!(leaf, IdentityLeaf::Fields(values) if fields(values));
    let ambiguous = match identity {
        Identity::CorrelatedKey(values) => fields(values),
        Identity::Choice(IdentityChoice::PreferArn { fallback, .. }) => leaf(fallback),
        Identity::Choice(IdentityChoice::Discriminated(branches)) => {
            branches.iter().any(|branch| leaf(&branch.identity))
        }
        _ => false,
    };
    if ambiguous {
        Err(RuntimeCatalogCompileError::Path {
            path: "identity ambiguous correlation".to_owned(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
