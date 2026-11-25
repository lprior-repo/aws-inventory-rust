use crate::runtime_catalog::emission_sources;
use crate::runtime_catalog::model_path::validation_context::ValidationContext;

#[derive(Clone, Copy)]
pub(super) enum Origin {
    SelectedOutput,
    Request,
}

#[derive(Clone, Copy)]
pub(super) enum Terminal {
    Text,
    TextOrList,
}

pub(in crate::runtime_catalog) fn active_kind(
    context: &ValidationContext<'_, '_>,
    kind: &str,
) -> bool {
    emission_sources::sources(context.kinds, context.checked).any(|source| source.emits_kind(kind))
}
