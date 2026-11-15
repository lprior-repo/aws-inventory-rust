use super::{RuntimeCatalogCompileError, ServiceIndex, encode};
use crate::kind_map::{KindMap, KindTarget};
use inventory_core::record::Value;
use std::collections::BTreeSet;

pub(super) fn compile(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
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
        .chain(
            kinds
                .api_only()
                .iter()
                .map(|entry| (entry.kind.as_str(), entry.table_name.as_str(), None)),
        )
        .map(|(kind, table, name_path)| schema(kinds, kind, table, name_path, services))
        .collect()
}

fn schema(
    kinds: &KindMap,
    kind: &str,
    table: &str,
    name_path: Option<&str>,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let columns = text_columns(kinds, kind, name_path, services)?;
    let name_path_value = name_path
        .map(|path| encode::path(services, path))
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
