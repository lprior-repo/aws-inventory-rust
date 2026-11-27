#[path = "support/runtime_provenance.rs"]
mod runtime_provenance;

use inventory_core::{
    catalog::runtime::{
        CatalogSourceProvenance, ModelSourceProvenance, RelationScope, ReviewDisposition,
        RuntimeCatalog, RuntimeCatalogError, compiled_inputs_sha256,
    },
    record::{FieldId, Key, MapRef, Value, ValueRef},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[path = "support/base_catalog.rs"]
mod base_catalog;
use base_catalog::base;

fn field_map(
    entries: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let entries = entries
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Value::Map(entries))
}

fn runtime_bytes(version: i64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        base()?,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(version)),
        (2, Value::List(Vec::new())),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(Vec::new())),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    let mut root = Value::Map(root.clone());
    runtime_provenance::reseal(&mut root, base()?)?;
    Ok(inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

fn denied_runtime_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let catalog = inventory_core::catalog::Catalog::borrow(base()?)?;
    let service = catalog
        .services()?
        .next()
        .ok_or("base catalog has no service")??;
    let operation = service
        .operations()?
        .next()
        .ok_or("base service has no operation")??;
    let decision = field_map([
        (1, Value::Text(service.name()?.to_owned())),
        (2, Value::Text(operation.name()?.to_owned())),
        (3, Value::Int(1)),
        (4, Value::Text("explicitly reviewed denial".to_owned())),
    ])?;
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        base()?,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(4)),
        (2, Value::List(Vec::new())),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(vec![decision])),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    let mut root = Value::Map(root.clone());
    runtime_provenance::reseal(&mut root, base()?)?;
    Ok(inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

fn relationship_runtime_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        base()?,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let relations = match extension(&mut root)?.get_mut(&key(4)?) {
        Some(Value::List(relations)) => relations,
        _ => return Err("base catalog has no relationships".into()),
    };
    relations.truncate(1);
    let relation = relations
        .first_mut()
        .ok_or("base catalog has no relationship")?;
    fields(relation)?.insert(key(6)?, Value::Int(2));
    runtime_provenance::reseal(&mut root, base()?)?;
    Ok(inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

#[test]
fn unknown_runtime_extension_version_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(6)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::ExtensionVersion(6))
    ));
    Ok(())
}

#[test]
fn current_runtime_extension_version_is_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(5)?;
    assert!(RuntimeCatalog::from_bytes(&bytes)?.plans().is_empty());
    Ok(())
}

#[test]
fn previous_version_three_runtime_schema_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(3)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::ExtensionVersion(3))
    ));
    Ok(())
}

#[test]
fn previous_version_two_runtime_schema_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(2)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::ExtensionVersion(2))
    ));
    Ok(())
}

#[test]
fn tagless_previous_runtime_schema_is_rejected_without_guessing_observation_state()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(1)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::ExtensionVersion(1))
    ));
    Ok(())
}

#[test]
fn valid_empty_runtime_extension_keeps_borrowed_catalog_and_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(4)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(runtime.plans().is_empty());
    assert!(runtime.schemas().is_empty());
    assert!(runtime.decisions().is_empty());
    assert_eq!(runtime.raw_bytes(), bytes.as_slice());
    Ok(())
}

#[test]
fn incomplete_plan_metadata_is_rejected_as_a_closed_capability()
-> Result<(), Box<dyn std::error::Error>> {
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        base()?,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(4)),
        (2, Value::List(vec![field_map([])?])),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(Vec::new())),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    let mut root = Value::Map(root.clone());
    runtime_provenance::reseal(&mut root, base()?)?;
    let bytes = inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn denied_review_decision_is_retained_without_minting_a_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = denied_runtime_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(runtime.plans().is_empty());
    assert_eq!(runtime.decisions().len(), 1);
    assert_eq!(
        runtime.decisions().first().ok_or("decision")?.disposition,
        ReviewDisposition::Denied {
            reason: "explicitly reviewed denial".to_owned(),
        }
    );
    Ok(())
}

#[test]
fn relationship_scope_preserves_home_region_as_distinct_typed_value()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = relationship_runtime_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(runtime.relationships().len(), 1);
    assert_eq!(
        runtime.relationships().first().ok_or("relationship")?.scope,
        RelationScope::HomeRegion
    );
    Ok(())
}

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut Fields> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("runtime fixture expected a map".into()),
    }
}

fn extension(root: &mut Value) -> TestResult<&mut Fields> {
    fields(
        fields(root)?
            .get_mut(&key(60_000)?)
            .ok_or("fixture extension missing")?,
    )
}

fn receipt(root: &mut Value) -> TestResult<&mut Fields> {
    fields(
        extension(root)?
            .get_mut(&key(7)?)
            .ok_or("fixture receipt missing")?,
    )
}

fn mutated_receipt(change: impl FnOnce(&mut Fields) -> TestResult) -> TestResult<Vec<u8>> {
    let bytes = runtime_bytes(4)?;
    mutate_receipt_in(&bytes, change)
}

fn mutate_receipt_in(
    bytes: &[u8],
    change: impl FnOnce(&mut Fields) -> TestResult,
) -> TestResult<Vec<u8>> {
    let control = RuntimeCatalog::from_bytes(bytes)?;
    assert_eq!(control.raw_bytes(), bytes);
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        bytes,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    change(receipt(&mut root)?)?;
    Ok(inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

fn rejects_capability(bytes: &[u8]) -> TestResult {
    match RuntimeCatalog::from_bytes(bytes) {
        Err(RuntimeCatalogError::Capability) => Ok(()),
        Err(error) => Err(format!("expected Capability, got {error:?}").into()),
        Ok(_) => Err("hostile source receipt was accepted".into()),
    }
}

#[test]
fn assembled_extension_preserves_true_source_pins_without_claiming_repository_or_cfn_input()
-> TestResult {
    let control = RuntimeCatalog::from_bytes(base()?)?;
    let bytes = runtime_bytes(4)?;
    let base_input = runtime_provenance::base_bytes(base()?)?;
    let mut restored = ValueRef::Map(MapRef::from_container_bytes(
        &base_input,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    assert!(!fields(&mut restored)?.contains_key(&key(60_000)?));
    let original_extension =
        MapRef::from_container_bytes(base()?, inventory_core::catalog::MAX_CATALOG_BYTES)?
            .get_field(FieldId::new(60_000)?)?
            .ok_or("control extension missing")?
            .to_value()?;
    fields(&mut restored)?.insert(key(60_000)?, original_extension);
    let rebuilt = inventory_core::record::encode_container(
        &restored,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    assert_eq!(rebuilt.as_slice(), base()?);
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let root = MapRef::from_container_bytes(&bytes, inventory_core::catalog::MAX_CATALOG_BYTES)?;
    let Some(ValueRef::Map(extension)) = root.get_field(FieldId::new(60_000)?)? else {
        return Err("fixture extension missing".into());
    };
    let source = control.source_provenance();
    assert_eq!(
        runtime.source_provenance(),
        &CatalogSourceProvenance {
            version: 1,
            models: ModelSourceProvenance::CompiledCatalog {
                sha256: Sha256::digest(&base_input).into(),
            },
            kind_map_version: source.kind_map_version,
            kind_map_sha256: source.kind_map_sha256,
            cfn_sha256: None,
            embedded_sources_sha256: source.embedded_sources_sha256,
            reviewed_inputs_sha256: compiled_inputs_sha256(extension)?,
        }
    );
    assert_eq!(runtime.raw_bytes(), bytes.as_slice());
    Ok(())
}

#[test]
fn loader_rejects_capability_when_runtime_source_receipt_is_missing() -> TestResult {
    let bytes = runtime_bytes(4)?;
    let control = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(control.source_provenance().version, 1);
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        &bytes,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    extension(&mut root)?
        .remove(&key(7)?)
        .ok_or("fixture receipt missing")?;
    rejects_capability(&inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
}

#[test]
fn loader_rejects_capability_when_any_required_receipt_field_is_missing() -> TestResult {
    (1..=7).try_for_each(|id| {
        let bytes = mutated_receipt(|receipt| {
            receipt
                .remove(&key(id)?)
                .ok_or("required receipt field missing")?;
            Ok(())
        })?;
        rejects_capability(&bytes)
    })
}

#[test]
fn loader_rejects_capability_when_receipt_or_kind_map_version_is_unsupported() -> TestResult {
    [(1, 0), (1, 2), (3, 0), (3, 2)]
        .into_iter()
        .try_for_each(|(id, version)| {
            rejects_capability(&mutated_receipt(|receipt| {
                receipt.insert(key(id)?, Value::Int(version));
                Ok(())
            })?)
        })
}

#[test]
fn loader_rejects_capability_when_receipt_has_unknown_numeric_or_named_keys() -> TestResult {
    [key(65_535)?, Key::Name("models".to_owned())]
        .into_iter()
        .try_for_each(|extra| {
            rejects_capability(&mutated_receipt(|receipt| {
                receipt.insert(extra, Value::Null);
                Ok(())
            })?)
        })
}

#[test]
fn loader_rejects_capability_when_receipt_sha256_fields_have_wrong_lengths_or_native_types()
-> TestResult {
    let invalid = [
        Value::Bytes(vec![]),
        Value::Bytes(vec![0; 31]),
        Value::Bytes(vec![0; 33]),
        Value::Text("00".repeat(32)),
        Value::Int(32),
        Value::List(vec![]),
    ];
    [2, 4, 5, 6, 7].into_iter().try_for_each(|id| {
        invalid.iter().try_for_each(|value| {
            rejects_capability(&mutated_receipt(|receipt| {
                if id == 2 {
                    let models = fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?;
                    models.insert(key(2)?, value.clone());
                } else {
                    receipt.insert(key(id)?, value.clone());
                }
                Ok(())
            })?)
        })
    })
}

#[test]
fn loader_rejects_capability_when_nonoptional_source_hash_is_null() -> TestResult {
    [2, 4, 6, 7].into_iter().try_for_each(|id| {
        rejects_capability(&mutated_receipt(|receipt| {
            if id == 2 {
                fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
                    .insert(key(2)?, Value::Null);
            } else {
                receipt.insert(key(id)?, Value::Null);
            }
            Ok(())
        })?)
    })
}

#[test]
fn loader_rejects_capability_when_model_origin_is_unknown_or_has_extra_fields() -> TestResult {
    [-1, 2, i64::MAX].into_iter().try_for_each(|tag| {
        rejects_capability(&mutated_receipt(|receipt| {
            fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
                .insert(key(1)?, Value::Int(tag));
            Ok(())
        })?)
    })?;
    [key(3)?, key(65_535)?, Key::Name("commit".to_owned())]
        .into_iter()
        .try_for_each(|extra| {
            rejects_capability(&mutated_receipt(|receipt| {
                fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
                    .insert(extra, Value::Null);
                Ok(())
            })?)
        })
}

#[test]
fn loader_rejects_capability_when_reviewed_checksum_is_changed_by_one_byte() -> TestResult {
    let bytes = mutated_receipt(|receipt| {
        let Some(Value::Bytes(checksum)) = receipt.get_mut(&key(7)?) else {
            return Err("fixture checksum is not bytes".into());
        };
        let first = checksum.first_mut().ok_or("fixture checksum is empty")?;
        *first ^= 1;
        Ok(())
    })?;
    rejects_capability(&bytes)
}

#[test]
fn loader_rejects_capability_when_valid_reviewed_metadata_changes_without_resealing() -> TestResult
{
    let bytes = denied_runtime_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(runtime.decisions().len(), 1);
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        &bytes,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    let Some(Value::List(decisions)) = extension(&mut root)?.get_mut(&key(6)?) else {
        return Err("fixture decisions missing".into());
    };
    fields(decisions.first_mut().ok_or("fixture decision missing")?)?.insert(
        key(4)?,
        Value::Text("different but valid reviewed denial".to_owned()),
    );
    let stale = inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    rejects_capability(&stale)?;
    runtime_provenance::reseal(&mut root, base()?)?;
    let sealed = inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    let runtime = RuntimeCatalog::from_bytes(&sealed)?;
    assert_eq!(
        runtime.decisions().first().ok_or("decision")?.disposition,
        ReviewDisposition::Denied {
            reason: "different but valid reviewed denial".to_owned(),
        }
    );
    Ok(())
}

#[test]
fn loader_rejects_capability_when_source_receipt_is_not_a_map() -> TestResult {
    [Value::Null, Value::Bytes(vec![0; 32]), Value::List(vec![])]
        .into_iter()
        .try_for_each(|malformed| {
            let bytes = runtime_bytes(4)?;
            let control = RuntimeCatalog::from_bytes(&bytes)?;
            assert_eq!(control.source_provenance().version, 1);
            let mut root = ValueRef::Map(MapRef::from_container_bytes(
                &bytes,
                inventory_core::catalog::MAX_CATALOG_BYTES,
            )?)
            .to_value()?;
            extension(&mut root)?.insert(key(7)?, malformed);
            rejects_capability(&inventory_core::record::encode_container(
                &root,
                inventory_core::catalog::MAX_CATALOG_BYTES,
            )?)
        })
}

#[test]
fn loader_rejects_capability_when_model_descriptor_is_missing_fields_or_is_mistyped() -> TestResult
{
    [1, 2].into_iter().try_for_each(|id| {
        rejects_capability(&mutated_receipt(|receipt| {
            fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
                .remove(&key(id)?)
                .ok_or("required model field missing")?;
            Ok(())
        })?)
    })?;
    [
        Value::Null,
        Value::Text("compiled".to_owned()),
        Value::List(vec![]),
    ]
    .into_iter()
    .try_for_each(|value| {
        rejects_capability(&mutated_receipt(|receipt| {
            receipt.insert(key(2)?, value);
            Ok(())
        })?)
    })
}

fn reviewed_checksum(bytes: &[u8]) -> TestResult<[u8; 32]> {
    let Some(ValueRef::Map(extension)) =
        MapRef::from_container_bytes(bytes, inventory_core::catalog::MAX_CATALOG_BYTES)?
            .get_field(FieldId::new(60_000)?)?
    else {
        return Err("fixture extension missing".into());
    };
    Ok(compiled_inputs_sha256(extension)?)
}

#[test]
fn public_reviewed_checksum_excludes_receipt_even_when_that_receipt_is_absent_or_corrupt()
-> TestResult {
    let bytes = runtime_bytes(4)?;
    let expected = reviewed_checksum(&bytes)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(runtime.source_provenance().reviewed_inputs_sha256, expected);
    let mut root = ValueRef::Map(MapRef::from_container_bytes(
        &bytes,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?)
    .to_value()?;
    extension(&mut root)?
        .remove(&key(7)?)
        .ok_or("fixture receipt missing")?;
    let missing = inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    extension(&mut root)?.insert(key(7)?, Value::Text("corrupt receipt".to_owned()));
    let corrupt = inventory_core::record::encode_container(
        &root,
        inventory_core::catalog::MAX_CATALOG_BYTES,
    )?;
    assert_eq!(reviewed_checksum(&missing)?, expected);
    assert_eq!(reviewed_checksum(&corrupt)?, expected);
    rejects_capability(&missing)?;
    rejects_capability(&corrupt)
}

#[test]
fn public_reviewed_checksum_binds_actual_version_and_each_capability_list() -> TestResult {
    let bytes = runtime_bytes(4)?;
    let expected = reviewed_checksum(&bytes)?;
    assert_ne!(reviewed_checksum(&runtime_bytes(2)?)?, expected);
    (2..=6).try_for_each(|id| {
        let mut root = ValueRef::Map(MapRef::from_container_bytes(
            &bytes,
            inventory_core::catalog::MAX_CATALOG_BYTES,
        )?)
        .to_value()?;
        extension(&mut root)?.insert(key(id)?, Value::List(vec![Value::Null]));
        let changed = inventory_core::record::encode_container(
            &root,
            inventory_core::catalog::MAX_CATALOG_BYTES,
        )?;
        assert_ne!(
            reviewed_checksum(&changed)?,
            expected,
            "unbound extension field {id}"
        );
        Ok(())
    })
}

fn repository_control() -> TestResult<RuntimeCatalog<'static>> {
    let runtime = RuntimeCatalog::from_bytes(base()?)?;
    let ModelSourceProvenance::Repository { commit, .. } = &runtime.source_provenance().models
    else {
        return Err("regenerated control must contain genuine repository provenance".into());
    };
    assert_eq!(commit, runtime.catalog().models_commit()?);
    let Some(_) = runtime.source_provenance().cfn_sha256 else {
        return Err("repository control must retain its actual CFN source hash".into());
    };
    Ok(runtime)
}

#[test]
fn loader_rejects_capability_when_repository_commit_is_well_formed_but_different_from_catalog()
-> TestResult {
    let control = repository_control()?;
    let mut commit = control.catalog().models_commit()?.as_bytes().to_vec();
    let first = commit
        .first_mut()
        .ok_or("repository control commit is empty")?;
    *first = if *first == b'a' { b'b' } else { b'a' };
    let changed = String::from_utf8(commit)?;
    let bytes = mutate_receipt_in(base()?, |receipt| {
        fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
            .insert(key(3)?, Value::Text(changed));
        Ok(())
    })?;
    rejects_capability(&bytes)
}

#[test]
fn loader_rejects_capability_when_repository_commit_is_missing_or_malformed() -> TestResult {
    let control = repository_control()?;
    assert_eq!(control.source_provenance().version, 1);
    let bytes = mutate_receipt_in(base()?, |receipt| {
        fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?
            .remove(&key(3)?)
            .ok_or("repository commit missing")?;
        Ok(())
    })?;
    rejects_capability(&bytes)?;
    [
        Value::Text(String::new()),
        Value::Text("a".repeat(39)),
        Value::Text("a".repeat(41)),
        Value::Text("z".repeat(40)),
        Value::Int(40),
    ]
    .into_iter()
    .try_for_each(|commit| {
        rejects_capability(&mutate_receipt_in(base()?, |receipt| {
            fields(receipt.get_mut(&key(2)?).ok_or("models missing")?)?.insert(key(3)?, commit);
            Ok(())
        })?)
    })
}

#[test]
fn loader_rejects_capability_when_repository_origin_loses_its_cfn_evidence() -> TestResult {
    let control = repository_control()?;
    assert_eq!(control.source_provenance().kind_map_version, 1);
    let bytes = mutate_receipt_in(base()?, |receipt| {
        receipt.insert(key(5)?, Value::Null);
        Ok(())
    })?;
    rejects_capability(&bytes)
}
