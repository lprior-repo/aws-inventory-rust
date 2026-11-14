use super::*;

fn unknown_at_depth_32() -> Result<Vec<u8>, RecordError> {
    let unknown =
        std::iter::repeat_n((), 31).fold(Value::Null, |value, ()| Value::List(vec![value]));
    let Value::Map(mut fields) = map([
        (CatalogKey::Format, Value::Int(1)),
        (CatalogKey::ModelsCommit, Value::Text("depth".into())),
        (CatalogKey::Services, Value::List(vec![])),
    ])?
    else {
        return Err(RecordError::NotMap);
    };
    fields.insert(Key::Field(crate::record::FieldId::new(99)?), unknown);
    Ok(Record::encode(&Value::Map(fields))?.into_bytes())
}

fn unknown_at_depth_33() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = unknown_at_depth_32()?;
    let leaf = bytes.pop().ok_or("missing unknown leaf")?;
    assert_eq!(leaf, 0xf6);
    // Only tests mutate writer bytes: wrap the unknown leaf in one additional array.
    bytes.extend([0x81, leaf]);
    Ok(bytes)
}

#[test]
fn depth_32_in_an_unknown_catalog_key_is_accepted_at_open() -> TestResult {
    // Given root map plus 31 arrays under an unknown key, opening accepts exactly 32 frames.
    let bytes = unknown_at_depth_32()?;
    MapRef::from_bytes_bounded(&bytes)?;
    assert_eq!(Catalog::borrow(&bytes)?.counts()?.get("services"), Some(0));
    Ok(())
}

#[test]
fn depth_33_in_an_unknown_catalog_key_is_typed_at_open() -> TestResult {
    // Given one more array, catalog opening rejects it before interpreting known fields.
    let bytes = unknown_at_depth_33()?;
    assert!(matches!(
        Catalog::borrow(&bytes),
        Err(CatalogError::Record(RecordError::DepthLimit))
    ));
    assert!(matches!(
        MapRef::from_bytes_bounded(&bytes),
        Err(RecordError::DepthLimit)
    ));
    Ok(())
}

#[test]
fn generic_record_readers_keep_deep_unknown_value_compatibility() -> TestResult {
    // Given depth 33 in an unknown value, generic readers retain their existing acceptance.
    let bytes = unknown_at_depth_33()?;
    MapRef::from_bytes(&bytes)?;
    Record::from_bytes(bytes)?;
    Ok(())
}

#[test]
fn unknown_tag_headers_do_not_spend_container_depth_frames() -> TestResult {
    // Given 32 containers and an opaque tag prefix, tags must not turn depth 32 into depth 33.
    let mut bytes = unknown_at_depth_32()?;
    let leaf = bytes.pop().ok_or("leaf")?;
    bytes.extend([0xc0, leaf]);
    MapRef::from_bytes_bounded(&bytes)?;
    Catalog::borrow(&bytes)?;
    Ok(())
}
