use inventory_core::{
    catalog::runtime::TagPlan,
    inventory::{MAX_RESOURCE_TAGS, ResourceKey, Tag, TagCoverage, TagObservation},
    record::{FieldId, KeyRef, MapRef, Record, ValueRef},
};

use super::SweepError;

pub(in crate::sweep_runtime) fn tag_observation(
    record: &Record,
    resource: &ResourceKey,
    plan: &TagPlan,
) -> Result<TagObservation, SweepError> {
    let path = match plan {
        TagPlan::List { path, .. } | TagPlan::Map { path } => path,
        _ => return Err(SweepError::Operation("unsupported modeled tag plan")),
    };
    let Some(value) = tag_value(record.root(), path)? else {
        return TagObservation::new(resource.clone(), TagCoverage::Unobserved, Vec::new())
            .map_err(SweepError::from);
    };
    let tags = match (plan, value) {
        (TagPlan::List { key, value, .. }, ValueRef::List(list)) => list
            .iter()?
            .take(MAX_RESOURCE_TAGS.saturating_add(1))
            .map(|item| list_pair(item?, *key, *value))
            .collect::<Result<Vec<_>, SweepError>>()?,
        (TagPlan::Map { .. }, ValueRef::Map(map)) => map
            .iter()?
            .take(MAX_RESOURCE_TAGS.saturating_add(1))
            .map(|item| map_pair(item?))
            .collect::<Result<Vec<_>, SweepError>>()?,
        _ => {
            return Err(SweepError::Operation(
                "modeled tags do not match the compiled shape",
            ));
        }
    };
    TagObservation::new(resource.clone(), TagCoverage::Complete, tags).map_err(SweepError::from)
}

// Missing is unobserved, but a present non-map ancestor is malformed, not empty.
fn tag_value<'a>(root: MapRef<'a>, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError> {
    let Some((leaf, parents)) = path.split_last() else {
        return Ok(Some(ValueRef::Map(root)));
    };
    let parent = parents.iter().try_fold(Some(root), |current, field| {
        let Some(map) = current else {
            return Ok(None);
        };
        match map.get_field(*field)? {
            Some(ValueRef::Map(child)) => Ok(Some(child)),
            None => Ok(None),
            _ => Err(SweepError::Operation(
                "modeled tag path contains a non-map ancestor",
            )),
        }
    })?;
    parent.map_or(Ok(None), |map| {
        map.get_field(*leaf).map_err(SweepError::from)
    })
}

fn list_pair(item: ValueRef<'_>, key: FieldId, value: FieldId) -> Result<Tag, SweepError> {
    let ValueRef::Map(map) = item else {
        return Err(SweepError::Operation(
            "modeled tag list contains a non-map pair",
        ));
    };
    match (map.get_field(key)?, map.get_field(value)?) {
        (Some(ValueRef::Text(key)), Some(ValueRef::Text(value))) => {
            Tag::new(key, value).map_err(SweepError::from)
        }
        _ => Err(SweepError::Operation(
            "modeled tag pair requires string key and value",
        )),
    }
}

fn map_pair((key, value): (KeyRef<'_>, ValueRef<'_>)) -> Result<Tag, SweepError> {
    match (key, value) {
        (KeyRef::Name(key), ValueRef::Text(value)) => {
            Tag::new(key, value).map_err(SweepError::from)
        }
        _ => Err(SweepError::Operation(
            "modeled tag map requires string keys and values",
        )),
    }
}

#[cfg(test)]
mod tests;
