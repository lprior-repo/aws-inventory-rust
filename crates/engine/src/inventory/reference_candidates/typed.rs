use super::{FieldId, Record, ReferenceTarget, ResourceId, StoreError, Value, ValueRef};
use inventory_core::record::Key;
use inventory_core::{
    catalog::runtime::RelationTargetMode,
    inventory::{AccountId, Kind, ReferenceComponent, Region},
};
use std::collections::BTreeMap;

// Zero cannot be a legacy FieldId, so the discriminator cannot shadow a V3 path.
const PREFIX: &[u8] = b"\0\0\0\0RT4";

pub(in crate::inventory) fn mode_code(mode: RelationTargetMode) -> u8 {
    match mode {
        RelationTargetMode::SelectedFields => 0,
        RelationTargetMode::SelectedRootText => 1,
        RelationTargetMode::BoundContextFields => 2,
        RelationTargetMode::BoundContextRootText => 3,
        RelationTargetMode::CanonicalId => 4,
    }
}

fn mode(value: i64) -> Result<RelationTargetMode, StoreError> {
    match value {
        0 => Ok(RelationTargetMode::SelectedFields),
        1 => Ok(RelationTargetMode::SelectedRootText),
        2 => Ok(RelationTargetMode::BoundContextFields),
        3 => Ok(RelationTargetMode::BoundContextRootText),
        4 => Ok(RelationTargetMode::CanonicalId),
        _ => Err(StoreError::Invalid("unknown reference target mode")),
    }
}

pub(in crate::inventory) fn stored_path(target: &ReferenceTarget) -> Result<Vec<u8>, StoreError> {
    if target.components().is_empty()
        && matches!(
            target.mode(),
            RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText
        )
    {
        return Ok(super::commit::reference_path(target.path()));
    }
    let terms = std::iter::once((target.mode(), target.path(), target.value()))
        .chain(
            target
                .components()
                .iter()
                .map(|term| (term.mode, term.path.as_ref(), &term.value)),
        )
        .map(|(mode, path, value)| {
            Value::List(vec![
                Value::Int(i64::from(mode_code(mode))),
                Value::List(
                    path.iter()
                        .map(|field| Value::Int(i64::from(field.value())))
                        .collect(),
                ),
                Value::Text(value.as_str().to_owned()),
            ])
        })
        .collect();
    let record = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        Value::List(terms),
    )])))?;
    Ok(PREFIX
        .iter()
        .copied()
        .chain(record.as_bytes().iter().copied())
        .collect())
}

pub(in crate::inventory) fn decode_target(
    account: AccountId,
    region: Region,
    kind: Kind,
    bytes: &[u8],
    value: ResourceId,
) -> Result<ReferenceTarget, StoreError> {
    let Some(encoded) = bytes.strip_prefix(PREFIX) else {
        let path = legacy_path(bytes)?;
        let mode = if path.is_empty() {
            RelationTargetMode::SelectedRootText
        } else {
            RelationTargetMode::SelectedFields
        };
        return Ok(ReferenceTarget::typed(
            account, region, kind, path, value, mode,
        )?);
    };
    let record = Record::from_bytes(encoded.to_vec())?;
    let Some(ValueRef::List(terms)) = record.root().get_field(FieldId::new(1)?)? else {
        return Err(StoreError::Invalid("typed reference terms are absent"));
    };
    if !(1..=8).contains(&terms.count()?) {
        return Err(StoreError::Invalid(
            "typed reference term count exceeds bounds",
        ));
    }
    let mut terms = terms.iter()?.map(|term| decode_term(term?));
    let primary = terms.next().ok_or(StoreError::Invalid(
        "typed reference primary term is absent",
    ))??;
    if primary.value != value {
        return Err(StoreError::Invalid(
            "typed reference primary value disagrees",
        ));
    }
    let components = terms.collect::<Result<Vec<_>, _>>()?;
    Ok(ReferenceTarget::typed(
        account,
        region,
        kind,
        primary.path.into_vec(),
        value,
        primary.mode,
    )?
    .with_components(components)?)
}

fn decode_term(value: ValueRef<'_>) -> Result<ReferenceComponent, StoreError> {
    let ValueRef::List(list) = value else {
        return Err(StoreError::Invalid("typed reference term is not a list"));
    };
    if list.count()? != 3 {
        return Err(StoreError::Invalid("typed reference term is malformed"));
    }
    let values = list.iter()?.collect::<Result<Vec<_>, _>>()?;
    let [
        ValueRef::Int(code),
        ValueRef::List(path),
        ValueRef::Text(value),
    ] = values.as_slice()
    else {
        return Err(StoreError::Invalid("typed reference term is malformed"));
    };
    if path.count()? > 8 {
        return Err(StoreError::Invalid("typed reference path exceeds bounds"));
    }
    let path = path
        .iter()?
        .map(|field| match field? {
            ValueRef::Int(field) => {
                Ok(FieldId::new(u32::try_from(field).map_err(|_| {
                    StoreError::Invalid("typed reference field exceeds bounds")
                })?)?)
            }
            _ => Err(StoreError::Invalid(
                "typed reference field is not an integer",
            )),
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(ReferenceComponent {
        mode: mode(*code)?,
        path: path.into_boxed_slice(),
        value: ResourceId::parse(value)?,
    })
}

fn legacy_path(bytes: &[u8]) -> Result<Vec<FieldId>, StoreError> {
    if bytes.len() > 32 {
        return Err(StoreError::Invalid("reference path exceeds bounds"));
    }
    let (chunks, remainder) = bytes.as_chunks::<4>();
    let fields = chunks
        .iter()
        .map(|chunk| Ok(FieldId::new(u32::from_be_bytes(*chunk))?))
        .collect::<Result<Vec<_>, StoreError>>()?;
    if !remainder.is_empty() {
        return Err(StoreError::Invalid("reference path is truncated"));
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn target(mode: RelationTargetMode, path: Vec<FieldId>) -> Result<ReferenceTarget, StoreError> {
        Ok(ReferenceTarget::typed(
            AccountId::parse("123456789012")?,
            Region::Global,
            Kind::parse("AWS::Fixture::Object")?,
            path,
            ResourceId::parse("value")?,
            mode,
        )?)
    }

    #[test]
    fn given_every_target_mode_when_stored_and_decoded_then_mode_paths_and_all_conjuncts_survive()
    -> TestResult {
        use RelationTargetMode::*;
        // Given every typed domain, including legacy selected fields and scalar roots.
        for mode in [
            SelectedFields,
            SelectedRootText,
            BoundContextFields,
            BoundContextRootText,
            CanonicalId,
        ] {
            let path = if matches!(mode, SelectedFields | BoundContextFields) {
                vec![FieldId::new(1)?]
            } else {
                vec![]
            };
            for compound in [false, true] {
                let target = target(mode, path.clone())?.with_components(if compound {
                    vec![
                        ReferenceComponent {
                            mode: BoundContextFields,
                            path: vec![FieldId::new(2)?].into_boxed_slice(),
                            value: ResourceId::parse("namespace")?,
                        },
                        ReferenceComponent {
                            mode: CanonicalId,
                            path: Box::new([]),
                            value: ResourceId::parse("canonical")?,
                        },
                    ]
                } else {
                    vec![]
                })?;
                // When the same path payload used by both unresolved rows and facet codec round-trips.
                let decoded = decode_target(
                    target.account().clone(),
                    target.region().clone(),
                    target.kind().clone(),
                    &stored_path(&target)?,
                    ResourceId::parse("value")?,
                )?;
                // Then no discriminator, path, value, or component can silently disappear.
                assert_eq!(decoded, target);
            }
        }
        Ok(())
    }

    #[test]
    fn given_legacy_path_matching_old_marker_when_decoded_then_it_remains_a_field_path()
    -> TestResult {
        // Given a legal V3 field ID whose bytes resemble a typed marker.
        let target = target(
            RelationTargetMode::SelectedFields,
            vec![FieldId::new(u32::from_be_bytes(*b"\0RT4"))?],
        )?;
        // When legacy bytes are read by the new codec.
        let encoded = stored_path(&target)?;
        assert_eq!(encoded.as_slice(), b"\0RT4");
        let decoded = decode_target(
            target.account().clone(),
            target.region().clone(),
            target.kind().clone(),
            &encoded,
            ResourceId::parse("value")?,
        )?;
        // Then it must not be treated as a typed record.
        assert_eq!(decoded, target);
        Ok(())
    }

    #[test]
    fn given_typed_reference_when_redundant_primary_value_is_forged_then_decoding_rejects_it()
    -> TestResult {
        // Given a canonical reference whose stored value is part of its typed witness.
        let target = target(RelationTargetMode::CanonicalId, vec![])?;
        // When an unresolved row disagrees with the encoded primary term.
        let result = decode_target(
            target.account().clone(),
            target.region().clone(),
            target.kind().clone(),
            &stored_path(&target)?,
            ResourceId::parse("forged")?,
        );
        // Then the value cannot be replaced independently of its typed evidence.
        assert!(matches!(
            result,
            Err(StoreError::Invalid(
                "typed reference primary value disagrees"
            ))
        ));
        Ok(())
    }
}
