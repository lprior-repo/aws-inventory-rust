mod selected;
use super::{RuntimeCatalogCompileError, ServiceIndex, encode};
use crate::kind_map::{KindMap, KindTarget};
use inventory_core::record::Value;
use std::collections::BTreeSet;

pub(super) fn compile(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    checked: Option<&crate::overlay::CheckedOverlays>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let context = Context {
        kinds,
        services,
        checked,
    };
    mapped_kinds(kinds)
        .chain(
            kinds
                .api_only()
                .iter()
                .map(|entry| {
                    (
                        entry.kind.as_str(),
                        entry.table_name.as_str(),
                        entry.display_name_path.as_deref(),
                    )
                }),
        )
        .map(|(kind, table, name_path)| schema(kind, table, name_path, &context))
        .collect()
}

fn mapped_kinds(kinds: &KindMap) -> impl Iterator<Item = (&str, &str, Option<&str>)> {
    kinds
        .entries()
        .iter()
        .filter_map(|entry| match &entry.target {
            KindTarget::Mapped { kind, table_name } => Some((
                kind.as_str(),
                table_name.as_str(),
                entry.display_name_path.as_deref(),
            )),
            KindTarget::Unmapped { .. } => None,
        })
}

struct Context<'a, 'b> {
    kinds: &'a KindMap,
    services: &'a ServiceIndex<'b>,
    checked: Option<&'a crate::overlay::CheckedOverlays>,
}

fn schema(
    kind: &str,
    table: &str,
    name_path: Option<&str>,
    context: &Context<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let active = context.checked.is_none_or(|checked| {
        super::model_path::active_kind(
            &super::model_path::ValidationContext {
                kinds: context.kinds,
                checked,
                services: context.services,
            },
            kind,
        )
    });
    let name_path = name_path.filter(|_| active);
    let mut columns = if active {
        text_columns(context, kind, name_path)?
    } else {
        Vec::new()
    };
    if let Some(column) = selected::column(context, kind)? {
        columns.push(column);
    }
    let name_path_value = name_path
        .map(|path| encode::path(context.services, path))
        .transpose()?;
    let fields = [
        (1, Value::Text(kind.to_owned())),
        (2, Value::Text(table.to_owned())),
        (3, Value::List(columns)),
    ];
    encode::map(
        fields
            .into_iter()
            .chain(name_path_value.map(|path| (4, path))),
    )
}

fn text_columns(
    context: &Context<'_, '_>,
    kind: &str,
    name_path: Option<&str>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let targets = context
        .kinds
        .relations()
        .iter()
        .filter(|relation| relation.target_kinds.iter().any(|target| target == kind))
        .filter(|relation| {
            context.checked.is_none_or(|checked| {
                super::model_path::active_kind(
                    &super::model_path::ValidationContext {
                        kinds: context.kinds,
                        checked,
                        services: context.services,
                    },
                    &relation.source_kind,
                )
            })
        });
    let paths = targets.filter_map(|relation| match &relation.target {
        crate::kind_map::ReviewedRelationTarget::SelectedOutput(
            crate::kind_map::ReviewedValuePath::Fields(path),
        ) => Some(path.as_str()),
        _ => None,
    });
    let compound_paths = context
        .kinds
        .compound_relations()
        .iter()
        .filter(|relation| relation.target_kinds.iter().any(|target| target == kind))
        .filter(|relation| {
            context.checked.is_none_or(|checked| {
                super::model_path::active_kind(
                    &super::model_path::ValidationContext {
                        kinds: context.kinds,
                        checked,
                        services: context.services,
                    },
                    &relation.source_kind,
                )
            })
        })
        .flat_map(|relation| &relation.components)
        .filter_map(|component| match &component.target {
            crate::kind_map::ReviewedRelationTarget::SelectedOutput(
                crate::kind_map::ReviewedValuePath::Fields(path),
            ) => Some(path.as_str()),
            _ => None,
        });
    let text_paths = name_path
        .into_iter()
        .chain(paths)
        .chain(compound_paths)
        .collect::<BTreeSet<_>>();
    text_paths
        .into_iter()
        .map(|path| {
            encode::map([
                (1, encode::path(context.services, path)?),
                (2, Value::Int(3)),
            ])
        })
        .collect()
}
