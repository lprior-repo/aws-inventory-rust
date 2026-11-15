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
    let context = Context { kinds, services, checked };
    mapped_kinds(kinds)
        .chain(
            kinds
                .api_only()
                .iter()
                .map(|entry| (entry.kind.as_str(), entry.table_name.as_str(), None)),
        )
        .map(|(kind, table, name_path)| schema(kind, table, name_path, &context))
        .collect()
}

fn mapped_kinds(kinds: &KindMap) -> impl Iterator<Item = (&str, &str, Option<&str>)> {
    kinds.entries().iter().filter_map(|entry| match &entry.target {
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
    let mut columns = text_columns(context.kinds, kind, name_path, context.services)?;
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
    kinds: &KindMap,
    kind: &str,
    name_path: Option<&str>,
    services: &ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let targets = kinds
        .relations()
        .iter()
        .filter(|relation| relation.target_kind == kind);
    let paths = targets.map(|relation| relation.target_path.as_str());
    let text_paths = name_path.into_iter().chain(paths).collect::<BTreeSet<_>>();
    text_paths
        .into_iter()
        .map(|path| encode::map([(1, encode::path(services, path)?), (2, Value::Int(3))]))
        .collect()
}
