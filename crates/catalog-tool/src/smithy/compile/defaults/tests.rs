use super::*;
use inventory_core::record::Key;

#[test]
fn target_defaults_normalize_blob_timestamp_and_null() -> Result<(), CompileError> {
    assert_eq!(
        normalize(&Json::String("AQID".into()), Some("blob"))?,
        Value::Bytes(vec![1, 2, 3])
    );
    assert_eq!(
        normalize(&Json::String(String::new()), Some("blob"))?,
        Value::Bytes(Vec::new())
    );
    assert_eq!(normalize(&Json::Null, Some("string"))?, Value::Null);
    let Value::Time(time) = normalize(&Json::from(1.25), Some("timestamp"))? else {
        return Err(CompileError::Trait("timestamp".into()));
    };
    assert_eq!((time.seconds(), time.nanoseconds()), (1, 250_000_000));
    let Value::Time(time) = normalize(&Json::from(0.999_999_999_9), Some("timestamp"))? else {
        return Err(CompileError::Trait("timestamp carry".into()));
    };
    assert_eq!((time.seconds(), time.nanoseconds()), (1, 0));
    Ok(())
}

#[test]
fn member_metadata_retains_null_default_and_client_optional_marker() -> Result<(), CompileError> {
    let traits = BTreeMap::from([
        ("smithy.api#default".into(), Json::Null),
        (
            "smithy.api#clientOptional".into(),
            Json::Object(serde_json::Map::new()),
        ),
    ]);
    assert_eq!(
        fields(&traits, "smithy.api#String", &BTreeMap::new())?,
        vec![
            (CatalogKey::DefaultValue, Value::Null),
            (CatalogKey::ClientOptional, Value::Bool(true)),
        ]
    );
    Ok(())
}

#[test]
fn client_optional_absence_and_invalid_payload_are_distinct() -> Result<(), CompileError> {
    assert!(fields(&BTreeMap::new(), "smithy.api#String", &BTreeMap::new())?.is_empty());
    let invalid = BTreeMap::from([("smithy.api#clientOptional".into(), Json::Bool(false))]);
    assert!(fields(&invalid, "smithy.api#String", &BTreeMap::new()).is_err());
    Ok(())
}
#[test]
fn compound_defaults_preserve_empty_values_and_document_keys() -> Result<(), CompileError> {
    let value = normalize(
        &serde_json::json!({"enabled":false,"zero":0,"items":[],"empty":{},"missing":null}),
        Some("document"),
    )?;
    let Value::Map(values) = value else {
        return Err(CompileError::Trait("map".into()));
    };
    assert_eq!(
        values.get(&Key::Name("enabled".into())),
        Some(&Value::Bool(false))
    );
    assert_eq!(values.get(&Key::Name("zero".into())), Some(&Value::Int(0)));
    assert_eq!(
        values.get(&Key::Name("items".into())),
        Some(&Value::List(Vec::new()))
    );
    assert_eq!(
        values.get(&Key::Name("empty".into())),
        Some(&Value::Map(BTreeMap::new()))
    );
    assert_eq!(values.get(&Key::Name("missing".into())), Some(&Value::Null));
    Ok(())
}

#[test]
fn integer_and_decimal_defaults_retain_target_exactness() -> Result<(), CompileError> {
    assert_eq!(normalize(&Json::from(0), Some("integer"))?, Value::Int(0));
    assert!(normalize(&Json::from(128), Some("byte")).is_err());
    assert_eq!(
        normalize(&Json::from(u64::MAX), None)?,
        Value::Text(u64::MAX.to_string())
    );
    assert_eq!(
        normalize(&Json::from(1.25), Some("bigDecimal"))?,
        Value::Text("1.25".into())
    );
    Ok(())
}

#[test]
fn blob_defaults_reject_invalid_base64_padding() {
    assert!(normalize(&Json::String("=AAA".into()), Some("blob")).is_err());
    assert!(normalize(&Json::String("AQ==".into()), Some("blob")).is_ok());
}

#[test]
fn nonfinite_numeric_defaults_use_text_only_for_float_targets() -> Result<(), CompileError> {
    assert_eq!(
        normalize(&Json::String("NaN".into()), Some("double"))?,
        Value::Text("NaN".into())
    );
    assert_eq!(
        normalize(&Json::String("NaN".into()), Some("string"))?,
        Value::Text("NaN".into())
    );
    assert!(normalize(&Json::String("invalid".into()), Some("float")).is_err());
    Ok(())
}
