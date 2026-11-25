mod encode;

pub(in crate::inventory) use encode::encode;

use super::super::{SchemaRegistry, StoreError};
use super::merge::{Path, dependency_paths, encode_paths};
use inventory_core::inventory::{Relation, ResourceId, ResourceKey, UnresolvedReference};
use inventory_core::record::{FieldId, Key, Record, Value, ValueRef};
use std::{collections::BTreeMap, sync::Arc};

pub(in crate::inventory) fn decode(
    _registry: &SchemaRegistry,
    record: &Record,
    from: &ResourceKey,
    facet: usize,
    owners: &BTreeMap<Path, usize>,
    source_facet: &Arc<[u8]>,
    source_authority: Option<&[u8; 32]>,
) -> Result<
    Vec<(
        UnresolvedReference,
        Option<ResourceId>,
        super::super::relation_permissions::ReferenceProvenance,
    )>,
    StoreError,
> {
    let references = list(record, 2)?
        .iter()?
        .map(|entry| {
            let values = values(entry?)?;
            let authority = match values.get(7) {
                Some(ValueRef::Bytes(bytes)) => <[u8; 32]>::try_from(*bytes).map_err(|_| {
                    StoreError::Invalid("checked facet source authority is not 32 bytes")
                })?,
                _ => {
                    return Err(StoreError::Invalid(
                        "checked facet source authority is absent or invalid",
                    ));
                }
            };
            let source_authority = *source_authority.ok_or(StoreError::Invalid(
                "checked SQL facet source authority is absent",
            ))?;
            if !owns_dependencies(values.get(4), facet, owners)? {
                return Ok(None);
            }
            let target = key(&values)?;
            let target_path = match values.get(3) {
                Some(ValueRef::Bytes(bytes)) => bytes.to_vec(),
                Some(ValueRef::List(path)) => {
                    let fields = path
                        .iter()?
                        .map(|value| match value? {
                            ValueRef::Int(value) => {
                                Ok(FieldId::new(u32::try_from(value).map_err(|_| {
                                    StoreError::Invalid("facet target path exceeds bounds")
                                })?)?)
                            }
                            _ => Err(StoreError::Invalid("facet target path is invalid")),
                        })
                        .collect::<Result<Vec<_>, StoreError>>()?;
                    super::super::commit::reference_path(&fields)
                }
                _ => return Err(StoreError::Invalid("facet target path is absent")),
            };
            let candidates = match values.get(5) {
                Some(ValueRef::List(list)) if list.count()? <= 64 => Some(
                    list.iter()?
                        .map(|value| match value? {
                            ValueRef::Text(kind) => {
                                Ok(inventory_core::inventory::Kind::parse(kind)?)
                            }
                            _ => Err(StoreError::Invalid("facet reference candidate is invalid")),
                        })
                        .collect::<Result<Vec<_>, StoreError>>()?,
                ),
                None | Some(ValueRef::Null) => None,
                _ => {
                    return Err(StoreError::Invalid(
                        "facet reference candidates are absent or excessive",
                    ));
                }
            };
            let target = super::super::reference_candidates::decode_target(
                target.account,
                target.region,
                target.kind,
                &target_path,
                target.id,
            )?;
            let target = if let Some(candidates) = candidates {
                target.with_candidate_kinds(candidates)?
            } else {
                target
            };
            let pin = match values.get(6) {
                Some(ValueRef::Null) => None,
                Some(ValueRef::Text(id)) => Some(ResourceId::parse(id)?),
                _ => {
                    return Err(StoreError::Invalid(
                        "checked facet target pin is absent or invalid",
                    ));
                }
            };
            let provenance = super::super::relation_permissions::ReferenceProvenance {
                authority,
                source_facet: Arc::clone(source_facet),
                source_authority,
            };
            Ok(Some((
                UnresolvedReference {
                    from: from.clone(),
                    relation: relation(&values)?,
                    source: source(&values)?,
                    target,
                },
                pin,
                provenance,
            )))
        })
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(references)
}

fn encode_dependencies(dependencies: Option<Vec<Vec<Vec<FieldId>>>>) -> Result<Value, StoreError> {
    let Some(groups) = dependencies else {
        return Ok(Value::Null);
    };
    Ok(Value::List(
        groups
            .into_iter()
            .map(|dependencies| {
                let paths = dependencies
                    .into_iter()
                    .map(|path| path.into_iter().map(Key::Field).collect::<Vec<_>>())
                    .collect::<Vec<_>>();
                Ok(Value::Bytes(
                    encode_paths(paths.iter().map(Vec::as_slice))?.into_bytes(),
                ))
            })
            .collect::<Result<Vec<_>, StoreError>>()?,
    ))
}

fn owns_dependencies(
    value: Option<&ValueRef<'_>>,
    facet: usize,
    owners: &BTreeMap<Path, usize>,
) -> Result<bool, StoreError> {
    match value {
        Some(ValueRef::Null) => Ok(true),
        Some(ValueRef::Bytes(bytes)) => {
            // V3 encoded alternative single-path authorities as one flat OR.
            let paths = dependency_paths(&Record::from_bytes(bytes.to_vec())?)?;
            Ok(paths.iter().any(|path| owns_path(path, facet, owners)))
        }
        Some(ValueRef::List(groups)) if (1..=64).contains(&groups.count()?) => {
            groups.iter()?.try_fold(false, |owned, group| {
                let ValueRef::Bytes(bytes) = group? else {
                    return Err(StoreError::Invalid("checked dependency group is invalid"));
                };
                let paths = dependency_paths(&Record::from_bytes(bytes.to_vec())?)?;
                if paths.is_empty() {
                    return Err(StoreError::Invalid("checked dependency group is empty"));
                }
                Ok(owned || paths.iter().all(|path| owns_path(path, facet, owners)))
            })
        }
        _ => Err(StoreError::Invalid(
            "checked facet relationship dependencies are absent",
        )),
    }
}

fn owns_path(dependency: &[Key], facet: usize, owners: &BTreeMap<Path, usize>) -> bool {
    let mut matching = owners
        .iter()
        .filter(|(path, _)| path.starts_with(dependency) || dependency.starts_with(path));
    matching.next().is_some_and(|(_, owner)| *owner == facet)
        && matching.all(|(_, owner)| *owner == facet)
}

fn list(record: &Record, field: u32) -> Result<inventory_core::record::ListRef<'_>, StoreError> {
    match record.root().get_field(FieldId::new(field)?)? {
        Some(ValueRef::List(list)) => Ok(list),
        _ => Err(StoreError::Invalid("facet relations list is absent")),
    }
}
fn values(value: ValueRef<'_>) -> Result<Vec<ValueRef<'_>>, StoreError> {
    match value {
        ValueRef::List(list) => Ok(list.iter()?.collect::<Result<Vec<_>, _>>()?),
        _ => Err(StoreError::Invalid("facet relation is not a list")),
    }
}
fn key(values: &[ValueRef<'_>]) -> Result<ResourceKey, StoreError> {
    match values.first() {
        Some(ValueRef::Bytes(bytes)) => super::super::work::decode_resource(bytes),
        _ => Err(StoreError::Invalid("facet relation target is absent")),
    }
}
fn relation(values: &[ValueRef<'_>]) -> Result<Relation, StoreError> {
    match values.get(1) {
        Some(ValueRef::Text(text)) => Ok(Relation::parse(text)?),
        _ => Err(StoreError::Invalid("facet relation label is absent")),
    }
}
fn source(values: &[ValueRef<'_>]) -> Result<inventory_core::inventory::EdgeSource, StoreError> {
    match values.get(2) {
        Some(ValueRef::Text("model")) => Ok(inventory_core::inventory::EdgeSource::Model),
        Some(ValueRef::Text("reviewed-rule")) => {
            Ok(inventory_core::inventory::EdgeSource::ReviewedRule)
        }
        Some(ValueRef::Text("inferred-exact")) => {
            Ok(inventory_core::inventory::EdgeSource::InferredExact)
        }
        _ => Err(StoreError::Invalid("facet relation source is absent")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::inventory::{AccountId, Kind, Region};

    #[test]
    fn given_a_shadowed_dependency_when_source_authority_is_missing_or_malformed_then_decode_rejects()
    -> Result<(), Box<dyn std::error::Error>> {
        let registry = SchemaRegistry {
            schemas: Arc::from([]),
            digest: [0; 32],
            operations: Arc::new(Default::default()),
            permissions: super::super::super::relation_permissions::Permissions::Unavailable,
        };
        let from = ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::parse("us-east-1")?,
            kind: Kind::parse("AWS::Example::Thing")?,
            id: ResourceId::parse("source")?,
        };
        let source = Arc::<[u8]>::from([]);
        for authority in [
            None,
            Some(Value::Null),
            Some(Value::Bytes(vec![0; 31])),
            Some(Value::Text("untrusted".to_owned())),
        ] {
            let mut entry = vec![Value::Null; 7];
            if let Some(authority) = authority {
                entry.push(authority);
            }
            let record = Record::encode(&Value::Map(BTreeMap::from([(
                Key::Field(FieldId::new(2)?),
                Value::List(vec![Value::List(entry)]),
            )])))?;
            assert!(matches!(
                decode(
                    &registry,
                    &record,
                    &from,
                    0,
                    &BTreeMap::new(),
                    &source,
                    Some(&[0; 32])
                ),
                Err(StoreError::Invalid(_))
            ));
        }
        Ok(())
    }
}
