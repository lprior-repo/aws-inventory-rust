use super::{
    MAX_ITEMS, RuntimeCatalogError, bounded, integer, list, map, only_fields, relation_modes,
    selected_path, text, text_value,
};
use crate::{
    catalog::runtime::{RelationScope, RelationSource, RelationshipPlan},
    inventory::Kind,
    record::{FieldId, ListRef, MapRef, ValueRef},
};

pub(in crate::catalog::runtime::decode) fn read(
    list: ListRef<'_>,
) -> Result<Vec<RelationshipPlan>, RuntimeCatalogError> {
    bounded(list, relation, MAX_ITEMS)
}

fn relation(value: ValueRef<'_>) -> Result<RelationshipPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let target_kind = Kind::parse(&text(fields, 3)?)?;
    let target_kinds = target_kinds(fields, &target_kind)?;
    let result = RelationshipPlan {
        source_kind: Kind::parse(&text(fields, 1)?)?,
        source_path: selected_path(list(fields, 2)?)?,
        source_path_mode: relation_modes::source_mode(fields)?,
        target_mode: relation_modes::target_mode(fields)?,
        guard: relation_modes::guard(fields)?,
        components: relation_modes::components(fields)?,
        source: source(fields)?,
        edge_source: edge_source(fields)?,
        target_kind,
        target_kinds: target_kinds.into_boxed_slice(),
        target_path: selected_path(list(fields, 4)?)?,
        relation: crate::inventory::Relation::parse(&text(fields, 5)?)?,
        scope: scope(fields)?,
        owner: super::super::owners::read_owner(fields)?,
    };
    relation_modes::validate(&result)?;
    only_fields(fields, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14])?;
    Ok(result)
}

fn target_kinds(fields: MapRef<'_>, target: &Kind) -> Result<Vec<Kind>, RuntimeCatalogError> {
    let kinds = match fields.get_field(FieldId::new(9)?)? {
        None => vec![target.clone()],
        Some(ValueRef::List(values)) => bounded(
            values,
            |value| Kind::parse(&text_value(value)?).map_err(RuntimeCatalogError::from),
            64,
        )?,
        Some(_) => return Err(RuntimeCatalogError::Capability),
    };
    if kinds.is_empty()
        || !kinds.contains(target)
        || kinds
            .iter()
            .enumerate()
            .any(|(index, kind)| kinds.iter().take(index).any(|prior| prior == kind))
    {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(kinds)
}

fn source(fields: MapRef<'_>) -> Result<RelationSource, RuntimeCatalogError> {
    match fields.get_field(FieldId::new(10)?)? {
        None | Some(ValueRef::Int(0)) => Ok(RelationSource::SelectedOutput),
        Some(ValueRef::Int(1)) => Ok(RelationSource::Request),
        Some(ValueRef::Int(2)) => Ok(RelationSource::Parent),
        Some(_) => Err(RuntimeCatalogError::Capability),
    }
}

fn edge_source(fields: MapRef<'_>) -> Result<crate::inventory::EdgeSource, RuntimeCatalogError> {
    use crate::inventory::EdgeSource;
    match fields.get_field(FieldId::new(7)?)? {
        Some(ValueRef::Int(1)) => Ok(EdgeSource::ReviewedRule),
        Some(ValueRef::Int(0)) => Ok(EdgeSource::Model),
        None | Some(_) => Err(RuntimeCatalogError::Capability),
    }
}

fn scope(fields: MapRef<'_>) -> Result<RelationScope, RuntimeCatalogError> {
    match integer(fields, 6)? {
        0 => Ok(RelationScope::AccountRegion),
        1 => Ok(RelationScope::Account),
        2 => Ok(RelationScope::HomeRegion),
        3 => Ok(RelationScope::ArnRegion),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

#[cfg(test)]
mod tests;
