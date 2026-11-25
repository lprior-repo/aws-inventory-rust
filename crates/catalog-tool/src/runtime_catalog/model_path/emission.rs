use crate::runtime_catalog::emission_sources;
use crate::runtime_catalog::model_path::validation_context::ValidationContext;

/// Which side of the call a relation path is read from.
#[derive(Clone, Copy)]
pub(super) enum Origin {
    SelectedOutput,
    Request,
}

/// What the resolved terminal must be.
#[derive(Clone, Copy)]
pub(super) enum Terminal {
    Text,
    TextOrList,
}

/// A kind is active when any callable emission, primary or additional, produces it.
pub(in crate::runtime_catalog) fn active_kind(
    context: &ValidationContext<'_, '_>,
    kind: &str,
) -> bool {
    emission_sources::sources(context.kinds, context.checked).any(|source| source.emits_kind(kind))
}


