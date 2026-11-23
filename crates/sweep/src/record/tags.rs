use inventory_core::{
    catalog::runtime::TagPlan,
    inventory::{MAX_RESOURCE_TAGS, ResourceKey, Tag, TagCoverage, TagObservation},
    record::{FieldId, KeyRef, MapRef, Record, ValueRef},
};

use super::SweepError;

pub(crate) fn tag_observation(
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
    let mut tags = Tags::default();
    tags.collect(plan, value)?;
    tags.observation(resource)
}

#[derive(Default)]
struct Tags {
    valid: Vec<Tag>,
    skipped: u32,
    exceeded: bool,
}

impl Tags {
    fn collect(&mut self, plan: &TagPlan, value: ValueRef<'_>) -> Result<(), SweepError> {
        match (plan, value) {
            (TagPlan::List { key, value, .. }, ValueRef::List(list)) => self.list(list, *key, *value),
            (TagPlan::Map { .. }, ValueRef::Map(map)) => self.map(map),
            _ => Err(SweepError::Operation("modeled tags do not match the compiled shape")),
        }
    }

    fn full(&mut self) -> bool {
        self.exceeded = self.valid.len() >= MAX_RESOURCE_TAGS.saturating_add(1);
        self.exceeded
    }

    fn list(&mut self, list: inventory_core::record::ListRef<'_>, key: FieldId, value: FieldId) -> Result<(), SweepError> {
        for item in list.iter()? {
            if self.full() { break; }
            self.accept(list_pair(item?, key, value), true);
        }
        Ok(())
    }

    fn map(&mut self, map: MapRef<'_>) -> Result<(), SweepError> {
        for item in map.iter()? {
            if self.full() { break; }
            self.accept(map_pair(item?), false);
        }
        Ok(())
    }

    fn accept(&mut self, pair: Result<Tag, SweepError>, list: bool) {
        match pair {
            Ok(tag) => self.valid.push(tag),
            Err(error) => {
                if list {
                    tracing::warn!(%error, "skipping malformed tag pair in list");
                } else {
                    tracing::warn!(%error, "skipping malformed tag pair in map");
                }
                self.skipped = self.skipped.saturating_add(1);
            }
        }
    }

    fn deduplicate(&mut self) {
        self.valid.sort_by(|left, right| left.key().cmp(right.key()));
        let before = self.valid.len();
        self.valid.dedup_by(|later, earlier| later.key() == earlier.key());
        if self.valid.len() < before {
            tracing::warn!("skipping duplicate tag keys");
            self.skipped = self.skipped.saturating_add(1);
        }
    }

    fn observation(mut self, resource: &ResourceKey) -> Result<TagObservation, SweepError> {
        self.deduplicate();
        let truncated = self.valid.len() > MAX_RESOURCE_TAGS;
        if truncated { self.valid.truncate(MAX_RESOURCE_TAGS); }
        let coverage = if self.skipped > 0 || truncated || self.exceeded {
            TagCoverage::Partial
        } else { TagCoverage::Complete };
        TagObservation::new(resource.clone(), coverage, self.valid).map_err(SweepError::from)
    }
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
