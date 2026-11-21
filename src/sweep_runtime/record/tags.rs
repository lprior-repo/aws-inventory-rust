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
    let mut valid_tags = Vec::new();
    let mut skipped: u32 = 0;
    let mut exceeded = false;
    match (plan, value) {
        (TagPlan::List { key, value, .. }, ValueRef::List(list)) => {
            for item in list.iter()? {
                if valid_tags.len() >= MAX_RESOURCE_TAGS.saturating_add(1) {
                    exceeded = true;
                    break;
                }
                let item = item?;
                match list_pair(item, *key, *value) {
                    Ok(tag) => valid_tags.push(tag),
                    Err(error) => {
                        tracing::warn!(%error, "skipping malformed tag pair in list");
                        skipped = skipped.saturating_add(1);
                    }
                }
            }
        }
        (TagPlan::Map { .. }, ValueRef::Map(map)) => {
            for item in map.iter()? {
                if valid_tags.len() >= MAX_RESOURCE_TAGS.saturating_add(1) {
                    exceeded = true;
                    break;
                }
                let item = item?;
                match map_pair(item) {
                    Ok(tag) => valid_tags.push(tag),
                    Err(error) => {
                        tracing::warn!(%error, "skipping malformed tag pair in map");
                        skipped = skipped.saturating_add(1);
                    }
                }
            }
        }
        _ => {
            return Err(SweepError::Operation(
                "modeled tags do not match the compiled shape",
            ));
        }
    }
    // A repeated key cannot be stored: keep the first occurrence, report Partial.
    valid_tags.sort_by(|left, right| left.key().cmp(right.key()));
    let before = valid_tags.len();
    valid_tags.dedup_by(|later, earlier| later.key() == earlier.key());
    if valid_tags.len() < before {
        tracing::warn!("skipping duplicate tag keys");
        skipped = skipped.saturating_add(1);
    }
    let truncated = valid_tags.len() > MAX_RESOURCE_TAGS;
    if truncated {
        valid_tags.truncate(MAX_RESOURCE_TAGS);
    }
    let coverage = if skipped > 0 || truncated || exceeded {
        TagCoverage::Partial
    } else {
        TagCoverage::Complete
    };
    // Truncation must be done before TagObservation::new so the constructor
    // never sees >100 tags.
    TagObservation::new(resource.clone(), coverage, valid_tags).map_err(SweepError::from)
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
