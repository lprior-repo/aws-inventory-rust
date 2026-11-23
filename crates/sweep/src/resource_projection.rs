mod trust;
pub(super) use trust::trust_references;

use super::{SweepError, record};
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ReferenceOwnerRule, RelationScope, RelationshipPlan, ResourceProjectionPlan,
    },
    inventory::{AccountId, Kind, Observation, Region, RegionId},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use sha2::{Digest, Sha256};

pub(super) fn classify(plan: &CallablePlan<'_>, row: &Record) -> Result<Kind, SweepError> {
    plan.classification().map_or_else(
        || Ok(plan.kind().clone()),
        |classification| {
            classification
                .classify(row)
                .cloned()
                .map_err(SweepError::from)
        },
    )
}

pub(super) fn project(plan: &CallablePlan<'_>, row: &Record) -> Result<Record, SweepError> {
    let Some(projection) = plan.projection() else {
        return Ok(row.clone());
    };
    project_with(projection, row)
}

pub(super) fn project_with(
    projection: &ResourceProjectionPlan,
    row: &Record,
) -> Result<Record, SweepError> {
    let path = projection.protected_path();
    if path.is_empty() || path.len() > 8 {
        return Err(SweepError::Operation("invalid reviewed projection path"));
    }
    Ok(Record::encode(&project_map(row.root(), path, projection)?)?)
}

fn project_map(
    map: MapRef<'_>,
    path: &[FieldId],
    projection: &ResourceProjectionPlan,
) -> Result<Value, SweepError> {
    let (field, tail) = path
        .split_first()
        .ok_or(SweepError::Operation("empty projection path"))?;
    map.iter()?
        .map(|entry| {
            let (key, value) = entry?;
            let replacement = if key != inventory_core::record::KeyRef::Field(*field) {
                value.to_value()?
            } else if tail.is_empty() {
                project_leaf(value, projection)?
            } else {
                project_child(value, tail, projection)?
            };
            Ok((key.to_key(), replacement))
        })
        .collect::<Result<_, SweepError>>()
        .map(Value::Map)
}

fn project_child(value: ValueRef<'_>, path: &[FieldId], projection: &ResourceProjectionPlan) -> Result<Value, SweepError> {
    match value {
        ValueRef::Map(child) => project_map(child, path, projection),
        ValueRef::Null => Ok(Value::Null),
        _ => Err(SweepError::Operation("projection parent is not a structure")),
    }
}

fn project_leaf(
    value: ValueRef<'_>,
    projection: &ResourceProjectionPlan,
) -> Result<Value, SweepError> {
    match projection {
        ResourceProjectionPlan::IamTrustSummary { .. } => trust_summary(value),
        ResourceProjectionPlan::SensitiveDigest { .. } => digest_value(value),
        ResourceProjectionPlan::SensitiveMapDigest { key, .. } => sensitive_map(value, key),
    }
}

fn trust_summary(value: ValueRef<'_>) -> Result<Value, SweepError> {
            let document = match value {
                ValueRef::Text(document) => Some(document),
                ValueRef::Null => None,
                _ => return Err(SweepError::Operation("trust document is not text")),
            };
            let statements = inventory_core::iam_trust::summarize(document)
                .map_err(|_| SweepError::Operation("invalid bounded IAM trust document"))?;
            Ok(Value::Map(
                [
                    (
                        Key::Name("trust_statements".into()),
                        summary_value(&serde_json::Value::Array(statements), 0)?,
                    ),
                    (
                        Key::Name("trust_policy_values_omitted".into()),
                        Value::Bool(true),
                    ),
                ]
                .into_iter()
                .collect(),
            ))
}

fn sensitive_map(value: ValueRef<'_>, key: &str) -> Result<Value, SweepError> {
    match value {
            ValueRef::Null => Ok(Value::Null),
            ValueRef::Map(map) => map
                .iter()?
                .map(|entry| {
                    let (name, value) = entry?;
                    let value = if name == inventory_core::record::KeyRef::Name(key) {
                        digest_value(value)?
                    } else {
                        value.to_value()?
                    };
                    Ok((name.to_key(), value))
                })
                .collect::<Result<_, SweepError>>()
                .map(Value::Map),
            _ => Err(SweepError::Operation(
                "sensitive projection target is not a map",
            )),
    }
}

fn digest_value(value: ValueRef<'_>) -> Result<Value, SweepError> {
    use std::fmt::Write;
    let value = match value {
        ValueRef::Null => return Ok(Value::Null),
        ValueRef::Text(value) if value.len() <= 16_384 => value,
        _ => {
            return Err(SweepError::Operation(
                "sensitive metadata text exceeds bounds or has invalid shape",
            ));
        }
    };
    let hash = Sha256::digest(value.as_bytes());
    let mut text = String::with_capacity(71);
    text.push_str("sha256:");
    hash.iter()
        .try_for_each(|byte| write!(&mut text, "{byte:02x}"))
        .map_err(|_| SweepError::Operation("digest formatting failed"))?;
    Ok(Value::Text(text))
}

fn summary_value(value: &serde_json::Value, depth: usize) -> Result<Value, SweepError> {
    if depth > 8 {
        return Err(SweepError::Operation("trust summary exceeds depth bound"));
    }
    match value {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(value) => Ok(Value::Bool(*value)),
        serde_json::Value::String(value) => Ok(Value::Text(value.clone())),
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| summary_value(value, depth.saturating_add(1)))
            .collect::<Result<_, _>>()
            .map(Value::List),
        serde_json::Value::Object(values) => summary_object(values, depth),
        _ => Err(SweepError::Operation("trust summary contains unsupported scalar")),
    }
}

fn summary_object(values: &serde_json::Map<String, serde_json::Value>, depth: usize) -> Result<Value, SweepError> {
    values
            .iter()
            .map(|(key, value)| {
                Ok((
                    Key::Name(key.clone()),
                    summary_value(value, depth.saturating_add(1))?,
                ))
            })
            .collect::<Result<_, SweepError>>()
            .map(Value::Map)
}

pub(super) fn reference_owner(
    relation: &RelationshipPlan,
    observation: &Observation,
    text: &str,
    home: &RegionId,
) -> Result<(AccountId, Region), SweepError> {
    match &relation.owner {
        ReferenceOwnerRule::Arn(rule) => rule
            .owner(text, &observation.key().region)
            .map_err(SweepError::from),
        ReferenceOwnerRule::ExecutionAccount => {
            let region = match relation.scope {
                RelationScope::AccountRegion => observation.key().region.clone(),
                RelationScope::Account => Region::Global,
                RelationScope::HomeRegion => Region::Regional(home.clone()),
                RelationScope::ArnRegion => arn_region(text)?,
            };
            Ok((observation.key().account.clone(), region))
        }
    }
}

fn arn_region(text: &str) -> Result<Region, SweepError> {
    let region = text
        .strip_prefix("arn:")
        .and_then(|rest| rest.split(':').nth(2))
        .filter(|region| !region.is_empty())
        .ok_or(SweepError::Operation(
            "ARN-region relationship value is not a regional ARN",
        ))?;
    RegionId::parse(region)
        .map(Region::Regional)
        .map_err(|_| SweepError::Operation("ARN-region relationship value names an invalid region"))
}

#[cfg(test)]
mod tests;
