use super::{TestResult, compile_model};
use inventory_core::{catalog::{Catalog, CatalogError, CatalogKey, CATALOG_VERSION}, record::{Key, Record, Value}};
use std::collections::BTreeMap;

fn root(fields: &[(CatalogKey, Value)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let map = fields.iter().try_fold(BTreeMap::new(), |mut map, (key, value)| {
        map.insert(Key::Field(key.field()?), value.clone());
        Ok::<_, Box<dyn std::error::Error>>(map)
    })?;
    Ok(Record::encode(&Value::Map(map))?.into_bytes())
}

fn catalog_root(version: i64, names: Vec<u8>, offsets: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    root(&[
        (CatalogKey::Format, Value::Int(version)),
        (CatalogKey::ModelsCommit, Value::Text("fixture".to_owned())),
        (CatalogKey::FieldNames, Value::Bytes(names)),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (CatalogKey::Services, Value::List(Vec::new())),
    ])
}
fn service_with_shape(kind: i64, shape_fields: &[(CatalogKey, Value)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut shape = shape_fields.to_vec();
    shape.push((CatalogKey::Name, Value::Text("Fixture".to_owned())));
    shape.push((CatalogKey::Kind, Value::Int(kind)));
    let shape = Record::encode(&Value::Map(shape.iter().try_fold(BTreeMap::new(), |mut map, (key, value)| {
        map.insert(Key::Field(key.field()?), value.clone());
        Ok::<_, Box<dyn std::error::Error>>(map)
    })?))?.into_bytes();
    let service = root(&[
        (CatalogKey::Name, Value::Text("example".to_owned())),
        (CatalogKey::ServiceShape, Value::Text("example#Service".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::Operations, Value::List(Vec::new())),
        (CatalogKey::Shapes, Value::List(vec![Value::Bytes(shape)])),
    ])?;
    root(&[
        (CatalogKey::Format, Value::Int(2)),
        (CatalogKey::ModelsCommit, Value::Text("fixture".to_owned())),
        (CatalogKey::FieldNames, Value::Bytes(Vec::new())),
        (CatalogKey::FieldNameOffsets, Value::Bytes(Vec::new())),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(service)])),
    ])
}

#[test]
fn compiler_bytes_are_repeatable_and_catalog_is_exactly_v2() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Input":{"type":"structure","members":{"Value":{"target":"smithy.api#String"}}}}}"#;
    let (_source, first) = compile_model(model, "Value\n")?;
    let (_source_again, second) = compile_model(model, "Value\n")?;
    assert_eq!(first, second);
    let service = Catalog::borrow(&first)?.service("example")?.ok_or("compiled service missing")?;
    assert_eq!(service.version()?, "1");
    assert_eq!(CATALOG_VERSION, 2);
    Ok(())
}

#[test]
fn v1_catalog_is_rejected_with_the_observed_version() -> TestResult {
    let bytes = catalog_root(1, Vec::new(), Vec::new())?;
    assert!(matches!(Catalog::borrow(&bytes), Err(CatalogError::Version(1))));
    Ok(())
}

#[test]
fn field_registry_rejects_out_of_bounds_and_non_utf8_offset_ranges() -> TestResult {
    let out_of_bounds = catalog_root(2, b"A".to_vec(), [0, 0, 0, 0, 0, 0, 0, 2].to_vec())?;
    assert!(matches!(Catalog::borrow(&out_of_bounds), Err(CatalogError::Reference)));
    let non_utf8 = catalog_root(2, [0xff].to_vec(), [0, 0, 0, 0, 0, 0, 0, 1].to_vec())?;
    assert!(matches!(Catalog::borrow(&non_utf8), Err(CatalogError::Reference)));
    let discontinuous = catalog_root(2, b"AB".to_vec(), [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2].to_vec())?;
    assert!(matches!(Catalog::borrow(&discontinuous), Err(CatalogError::Reference)));
    Ok(())
}
#[test]
fn public_reader_rejects_unknown_wire_enums_and_out_of_bounds_shape_references() -> TestResult {
    let enum_bytes = service_with_shape(2, &[(CatalogKey::ShapeTimestampFormat, Value::Text("wall-clock".to_owned()))])?;
    assert!(matches!(Catalog::borrow(&enum_bytes)?.service("example"), Err(CatalogError::Field(CatalogKey::ShapeTimestampFormat))));
    let reference_bytes = service_with_shape(16, &[(CatalogKey::Element, Value::Int(9))])?;
    assert!(matches!(Catalog::borrow(&reference_bytes)?.service("example"), Err(CatalogError::Reference)));
    Ok(())
}
