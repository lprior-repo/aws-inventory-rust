use super::{TestResult, compile_model};
use inventory_core::{
    catalog::{CATALOG_VERSION, Catalog, CatalogError, CatalogKey, Shape, Target},
    record::{Key, ListRef, Record, Value, encode_list},
};
use std::collections::BTreeMap;

fn root(fields: &[(CatalogKey, Value)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let map = fields
        .iter()
        .try_fold(BTreeMap::new(), |mut map, (key, value)| {
            map.insert(Key::Field(key.field()?), value.clone());
            Ok::<_, Box<dyn std::error::Error>>(map)
        })?;
    Ok(Record::encode(&Value::Map(map))?.into_bytes())
}

fn catalog_root(
    version: i64,
    names: Vec<u8>,
    offsets: Vec<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    root(&[
        (CatalogKey::Format, Value::Int(version)),
        (CatalogKey::ModelsCommit, Value::Text("fixture".to_owned())),
        (CatalogKey::FieldNames, Value::Bytes(names)),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (CatalogKey::Services, Value::List(Vec::new())),
    ])
}
fn service_with_shape(
    kind: i64,
    shape_fields: &[(CatalogKey, Value)],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let shapes = vec![shape_value(kind, shape_fields)?];
    let shape_offsets = shape_offsets_value(&shapes)?;
    let service = service_record(shapes, shape_offsets)?;
    root(&[
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text("fixture".to_owned())),
        (CatalogKey::FieldNames, Value::Bytes(Vec::new())),
        (CatalogKey::FieldNameOffsets, Value::Bytes(Vec::new())),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(service)]),
        ),
    ])
}

fn shape_value(
    kind: i64,
    shape_fields: &[(CatalogKey, Value)],
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut fields = shape_fields.to_vec();
    fields.push((CatalogKey::Name, Value::Text("Fixture".to_owned())));
    fields.push((CatalogKey::Kind, Value::Int(kind)));
    let values = fields
        .iter()
        .try_fold(BTreeMap::new(), |mut map, (key, value)| {
            map.insert(Key::Field(key.field()?), value.clone());
            Ok::<_, Box<dyn std::error::Error>>(map)
        })?;
    Ok(Value::Map(values))
}

fn shape_offsets_value(shapes: &[Value]) -> Result<Value, Box<dyn std::error::Error>> {
    let list_bytes = encode_list(shapes)?;
    let offsets = ListRef::from_bytes(&list_bytes)?.encoded_item_offsets()?;
    Ok(Value::Bytes(
        offsets
            .iter()
            .flat_map(|offset| offset.to_be_bytes())
            .collect(),
    ))
}

fn service_record(
    shapes: Vec<Value>,
    shape_offsets: Value,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    root(&[
        (CatalogKey::Name, Value::Text("example".to_owned())),
        (
            CatalogKey::ServiceShape,
            Value::Text("example#Service".to_owned()),
        ),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::Operations, Value::List(Vec::new())),
        (CatalogKey::Protocols, Value::List(Vec::new())),
        (CatalogKey::Resources, Value::List(Vec::new())),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, shape_offsets),
    ])
}

#[test]
fn compiler_bytes_are_repeatable_and_catalog_is_exactly_v4() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Input":{"type":"structure","members":{"Value":{"target":"smithy.api#String"}}}}}"#;
    let (_source, first) = compile_model(model, "Value\n")?;
    let (_source_again, second) = compile_model(model, "Value\n")?;
    assert_eq!(first, second);
    let service = Catalog::borrow(&first)?
        .service("example")?
        .ok_or("compiled service missing")?;
    assert_eq!(service.version()?, "1");
    assert_eq!(CATALOG_VERSION, 4);
    Ok(())
}

#[test]
fn v1_catalog_is_rejected_with_the_observed_version() -> TestResult {
    let bytes = catalog_root(1, Vec::new(), Vec::new())?;
    assert!(matches!(
        Catalog::borrow(&bytes),
        Err(CatalogError::Version(1))
    ));
    Ok(())
}

#[test]
fn field_registry_rejects_out_of_bounds_and_non_utf8_offset_ranges() -> TestResult {
    let out_of_bounds = catalog_root(
        CATALOG_VERSION,
        b"A".to_vec(),
        [0, 0, 0, 0, 0, 0, 0, 2].to_vec(),
    )?;
    assert!(matches!(
        Catalog::borrow(&out_of_bounds),
        Err(CatalogError::Reference)
    ));
    let non_utf8 = catalog_root(
        CATALOG_VERSION,
        [0xff].to_vec(),
        [0, 0, 0, 0, 0, 0, 0, 1].to_vec(),
    )?;
    assert!(matches!(
        Catalog::borrow(&non_utf8),
        Err(CatalogError::Reference)
    ));
    let discontinuous = catalog_root(
        CATALOG_VERSION,
        b"AB".to_vec(),
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2].to_vec(),
    )?;
    assert!(matches!(
        Catalog::borrow(&discontinuous),
        Err(CatalogError::Reference)
    ));
    Ok(())
}
#[test]
fn public_reader_rejects_unknown_wire_enums_and_out_of_bounds_shape_references() -> TestResult {
    let scalar = service_with_shape(2, &[])?;
    assert_eq!(fixture_shape(&scalar)?.kind()?, "string");
    let recursive_list = service_with_shape(16, &[(CatalogKey::Element, Value::Int(0))])?;
    assert_eq!(
        fixture_shape(&recursive_list)?.element()?,
        Some(Target::Shape(0))
    );
    let enum_bytes = service_with_shape(
        2,
        &[(
            CatalogKey::ShapeTimestampFormat,
            Value::Text("wall-clock".to_owned()),
        )],
    )?;
    let observed = Catalog::borrow(&enum_bytes)?.service("example");
    assert!(
        matches!(
            &observed,
            Err(CatalogError::Field(CatalogKey::ShapeTimestampFormat))
        ),
        "observed error: {:?}",
        observed.as_ref().err()
    );
    let reference_bytes = service_with_shape(16, &[(CatalogKey::Element, Value::Int(9))])?;
    assert!(matches!(
        Catalog::borrow(&reference_bytes)?.service("example"),
        Err(CatalogError::Reference)
    ));
    Ok(())
}

fn fixture_shape(bytes: &[u8]) -> Result<Shape<'_>, Box<dyn std::error::Error>> {
    Catalog::borrow(bytes)?
        .service("example")?
        .ok_or("service missing")?
        .shapes()?
        .next()
        .transpose()?
        .ok_or_else(|| "shape missing".into())
}
