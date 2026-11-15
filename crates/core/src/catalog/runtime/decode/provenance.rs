use super::metadata::{field, integer, map_field, only_fields, text};
use super::{RuntimeCatalogError, lookup};
use super::super::{CatalogSourceProvenance, ModelSourceProvenance};
use crate::record::{MapRef, ValueRef};
use sha2::{Digest, Sha256};

pub(super) fn validate_inputs(
    extension: MapRef<'_>,
    provenance: &CatalogSourceProvenance,
) -> Result<(), RuntimeCatalogError> {
    if compiled_inputs_sha256(extension)? != provenance.reviewed_inputs_sha256 {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

pub(crate) fn compiled_inputs_sha256(extension: MapRef<'_>) -> Result<[u8; 32], RuntimeCatalogError> {
    let mut hash = Sha256::new();
    hash.update(b"runtime-reviewed-inputs-v1\0");
    hash.update(integer(extension, 1)?.to_le_bytes());
    for field in 2_u32..=6 {
        let ValueRef::List(list) = lookup(extension, field)? else {
            return Err(RuntimeCatalogError::Capability);
        };
        let bytes = list.encoded_bytes()?;
        hash.update(field.to_le_bytes());
        hash.update(u64::try_from(bytes.len()).map_err(|_| RuntimeCatalogError::Bounds)?.to_le_bytes());
        hash.update(bytes);
    }
    Ok(hash.finalize().into())
}

pub(super) fn validate_models(
    bytes: &[u8],
    catalog: crate::catalog::Catalog<'_>,
    provenance: &CatalogSourceProvenance,
) -> Result<(), RuntimeCatalogError> {
    let matches = match &provenance.models {
        ModelSourceProvenance::Repository { commit, .. } => catalog.models_commit()? == commit,
        ModelSourceProvenance::CompiledCatalog { sha256 } => compiled_catalog_sha256(bytes)? == *sha256,
    };
    if !matches {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

struct HashWriter<'a>(&'a mut Sha256);

impl minicbor::encode::Write for HashWriter<'_> {
    type Error = std::convert::Infallible;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.update(bytes);
        Ok(())
    }
}

fn compiled_catalog_sha256(bytes: &[u8]) -> Result<[u8; 32], RuntimeCatalogError> {
    let mut decoder = minicbor::Decoder::new(bytes);
    let count = decoder.map().map_err(crate::record::RecordError::from)?
        .ok_or(RuntimeCatalogError::Capability)?;
    let mut hash = Sha256::new();
    minicbor::Encoder::new(HashWriter(&mut hash))
        .map(count.checked_sub(1).ok_or(RuntimeCatalogError::Capability)?)
        .map_err(|_| RuntimeCatalogError::Capability)?;
    let mut found = false;
    for _ in 0..count {
        let start = decoder.position();
        let extension = decoder.clone().u32().ok() == Some(60_000);
        decoder.skip().map_err(crate::record::RecordError::from)?;
        decoder.skip().map_err(crate::record::RecordError::from)?;
        if extension {
            if found {
                return Err(RuntimeCatalogError::Capability);
            }
            found = true;
        } else {
            hash.update(bytes.get(start..decoder.position()).ok_or(RuntimeCatalogError::Bounds)?);
        }
    }
    if !found || decoder.position() != bytes.len() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(hash.finalize().into())
}

pub(super) fn read(fields: MapRef<'_>) -> Result<CatalogSourceProvenance, RuntimeCatalogError> {
    only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
    if integer(fields, 1)? != 1 || integer(fields, 3)? != 1 {
        return Err(RuntimeCatalogError::Capability);
    }
    let models = models(map_field(fields, 2)?)?;
    let cfn_sha256 = match field(fields, 5)? {
        ValueRef::Null => None,
        value => Some(sha256(value)?),
    };
    if matches!(models, ModelSourceProvenance::Repository { .. }) && cfn_sha256.is_none() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(CatalogSourceProvenance {
        version: 1,
        models,
        kind_map_version: 1,
        kind_map_sha256: sha256(lookup(fields, 4)?)?,
        cfn_sha256,
        embedded_sources_sha256: sha256(lookup(fields, 6)?)?,
        reviewed_inputs_sha256: sha256(lookup(fields, 7)?)?,
    })
}

fn models(fields: MapRef<'_>) -> Result<ModelSourceProvenance, RuntimeCatalogError> {
    let sha256 = sha256(lookup(fields, 2)?)?;
    match integer(fields, 1)? {
        0 => {
            only_fields(fields, &[1, 2, 3])?;
            let commit = text(fields, 3)?;
            if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(ModelSourceProvenance::Repository { commit, sha256 })
        }
        1 => {
            only_fields(fields, &[1, 2])?;
            Ok(ModelSourceProvenance::CompiledCatalog { sha256 })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn sha256(value: ValueRef<'_>) -> Result<[u8; 32], RuntimeCatalogError> {
    match value {
        ValueRef::Bytes(bytes) => bytes.try_into().map_err(|_| RuntimeCatalogError::Capability),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{FieldId, Key, Record, Value};
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> Result<Value, RuntimeCatalogError> {
        entries.into_iter().map(|(key, value)| Ok((Key::Field(FieldId::new(key)?), value)))
            .collect::<Result<_, RuntimeCatalogError>>().map(Value::Map)
    }

    fn source(repository: bool) -> Result<Value, RuntimeCatalogError> {
        let mut fields = vec![(1, Value::Int(i64::from(!repository))), (2, Value::Bytes(vec![7; 32]))];
        if repository {
            fields.push((3, Value::Text("0123456789abcdef0123456789abcdef01234567".into())));
        }
        map(fields)
    }

    fn receipt(models: Value, cfn: Value) -> Result<Value, RuntimeCatalogError> {
        map([(1, Value::Int(1)), (2, models), (3, Value::Int(1)),
            (4, Value::Bytes(vec![8; 32])), (5, cfn), (6, Value::Bytes(vec![9; 32])),
            (7, Value::Bytes(vec![10; 32]))])
    }

    fn decode(value: Value) -> Result<CatalogSourceProvenance, RuntimeCatalogError> {
        read(Record::encode(&value)?.root())
    }

    #[test]
    fn repository_receipt_preserves_exact_pins() -> TestResult {
        let result = decode(receipt(source(true)?, Value::Bytes(vec![6; 32]))?)?;
        assert_eq!(result, CatalogSourceProvenance {
            version: 1,
            models: ModelSourceProvenance::Repository {
                commit: "0123456789abcdef0123456789abcdef01234567".into(), sha256: [7; 32],
            },
            kind_map_version: 1, kind_map_sha256: [8; 32], cfn_sha256: Some([6; 32]),
            embedded_sources_sha256: [9; 32],
            reviewed_inputs_sha256: [10; 32],
        });
        Ok(())
    }

    #[test]
    fn assembled_receipt_does_not_claim_repository_or_cfn_sources() -> TestResult {
        let result = decode(receipt(source(false)?, Value::Null)?)?;
        assert_eq!(result.models, ModelSourceProvenance::CompiledCatalog { sha256: [7; 32] });
        assert_eq!(result.cfn_sha256, None);
        Ok(())
    }

    #[test]
    fn repository_receipt_requires_its_actual_cfn_pin() -> TestResult {
        assert!(matches!(decode(receipt(source(true)?, Value::Null)?), Err(RuntimeCatalogError::Capability)));
        Ok(())
    }

    #[test]
    fn malformed_receipt_fields_are_rejected_instead_of_ignored() -> TestResult {
        for (field, invalid) in [(1, Value::Int(2)), (3, Value::Int(0)),
            (4, Value::Bytes(vec![8; 31])), (5, Value::Bytes(vec![6; 33])),
            (6, Value::Text("not-a-digest".into())), (7, Value::Bytes(vec![10; 31])), (8, Value::Null)] {
            let Value::Map(mut fields) = receipt(source(true)?, Value::Bytes(vec![6; 32]))? else {
                return Err("receipt fixture is not a map".into());
            };
            fields.insert(Key::Field(FieldId::new(field)?), invalid);
            assert!(matches!(decode(Value::Map(fields)), Err(RuntimeCatalogError::Capability)), "field {field}");
        }
        Ok(())
    }

    #[test]
    fn missing_receipt_fields_and_invalid_model_claims_are_rejected() -> TestResult {
        for missing in 1..=7 {
            let Value::Map(mut fields) = receipt(source(true)?, Value::Bytes(vec![6; 32]))? else {
                return Err("receipt fixture is not a map".into());
            };
            fields.remove(&Key::Field(FieldId::new(missing)?));
            assert!(matches!(decode(Value::Map(fields)), Err(RuntimeCatalogError::Capability)), "field {missing}");
        }
        for models in [
            map([(1, Value::Int(2)), (2, Value::Bytes(vec![7; 32]))])?,
            map([(1, Value::Int(0)), (2, Value::Bytes(vec![7; 32])), (3, Value::Text("not-a-commit".into()))])?,
            map([(1, Value::Int(1)), (2, Value::Bytes(vec![7; 32])), (3, Value::Text("invented-commit".into()))])?,
        ] {
            assert!(matches!(decode(receipt(models, Value::Bytes(vec![6; 32]))?), Err(RuntimeCatalogError::Capability)));
        }
        Ok(())
    }

    #[test]
    fn compiled_input_checksum_binds_every_capability_list_and_excludes_the_receipt() -> TestResult {
        let fields = || map([(1, Value::Int(2)), (2, Value::List(vec![])), (3, Value::List(vec![])),
            (4, Value::List(vec![])), (5, Value::List(vec![])), (6, Value::List(vec![]))]);
        let empty = Record::encode(&fields()?)?;
        let mut expected = Sha256::new();
        expected.update(b"runtime-reviewed-inputs-v1\0");
        expected.update(2_i64.to_le_bytes());
        for number in [2_u32, 3, 4, 5, 6] {
            expected.update(number.to_le_bytes());
            expected.update(1_u64.to_le_bytes());
            expected.update([0x80]);
        }
        let expected: [u8; 32] = expected.finalize().into();
        assert_eq!(compiled_inputs_sha256(empty.root())?, expected);
        let mut pin = decode(receipt(source(false)?, Value::Null)?)?;
        pin.reviewed_inputs_sha256 = expected;
        validate_inputs(empty.root(), &pin)?;
        for number in 2..=6 {
            let Value::Map(mut changed) = fields()? else { return Err("fixture is not a map".into()); };
            changed.insert(Key::Field(FieldId::new(number)?), Value::List(vec![Value::Int(42)]));
            let changed = Record::encode(&Value::Map(changed))?;
            assert!(matches!(validate_inputs(changed.root(), &pin), Err(RuntimeCatalogError::Capability)), "field {number}");
        }
        let Value::Map(mut with_receipt) = fields()? else { return Err("fixture is not a map".into()); };
        with_receipt.insert(Key::Field(FieldId::new(7)?), Value::Text("receipt excluded from its own hash".into()));
        assert_eq!(compiled_inputs_sha256(Record::encode(&Value::Map(with_receipt))?.root())?, expected);
        Ok(())
    }
}
