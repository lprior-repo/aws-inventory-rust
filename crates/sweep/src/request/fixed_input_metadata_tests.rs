use super::super::{field_mut, map_mut};
use super::{
    TestResult, change_shape_and_reseal_models, compiled, filters, kind_value, signed_bytes,
    with_field,
};
use catalog_tool::overlay::FixedValue;
use inventory_core::{
    catalog::{
        Catalog, CatalogError, CatalogKey,
        runtime::{RuntimeCatalog, RuntimeCatalogError},
    },
    record::{Key, Record, Value, ValueRef},
};
use serde_json::json;

type BytesResult = Result<Vec<u8>, Box<dyn std::error::Error>>;

fn domain(bytes: Vec<u8>, name: &str, value: Value) -> BytesResult {
    let root = change_shape_and_reseal_models(bytes, name, |shape| {
        *field_mut(shape, CatalogKey::EnumValues.field()?.value())? = value;
        Ok(())
    })?;
    Ok(Record::encode(&root)?.into_bytes())
}
fn historical(bytes: Vec<u8>, name: &str) -> BytesResult {
    let root = change_shape_and_reseal_models(bytes, name, |shape| {
        *field_mut(shape, CatalogKey::Kind.field()?.value())? = kind_value("string")?;
        map_mut(shape)?
            .remove(&Key::Field(CatalogKey::EnumValues.field()?))
            .ok_or("domain missing")?;
        Ok(())
    })?;
    Ok(Record::encode(&root)?.into_bytes())
}
fn accepted(bytes: &[u8], name: &str, expected: ValueRef<'_>, member: bool) -> TestResult {
    RuntimeCatalog::from_bytes(bytes)?
        .lookup("fixed", "ListClusters")
        .ok_or("plan missing")?;
    let catalog = Catalog::borrow(bytes)?;
    let service = catalog.service("fixed")?.ok_or("service missing")?;
    let shapes = service.shapes()?.collect::<Result<Vec<_>, _>>()?;
    let shape = shapes
        .iter()
        .find(|shape| shape.name().is_ok_and(|found| found == name))
        .ok_or("shape missing")?;
    assert_eq!(shape.accepts_enum_value(expected)?, member);
    Ok(())
}
fn rejected(bytes: &[u8]) {
    assert!(matches!(
        RuntimeCatalog::from_bytes(bytes),
        Err(RuntimeCatalogError::Catalog(CatalogError::Field(
            CatalogKey::EnumValues
        )))
    ));
}

#[test]
fn given_unused_enum_metadata_when_type_or_kind_is_invalid_then_public_loader_rejects_exact_field()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    for (name, value) in [
        ("Legacy", Value::Bool(true)),
        ("Legacy", Value::Null),
        ("Legacy", Value::List(vec![Value::Int(7)])),
        ("Mode", Value::Null),
        ("Mode", Value::List(vec![Value::Text("7".into())])),
        (
            "Mode",
            Value::List(vec![Value::Int(i64::from(i32::MAX) + 1)]),
        ),
        (
            "Mode",
            Value::List(vec![Value::Int(i64::from(i32::MIN) - 1)]),
        ),
    ] {
        rejected(&domain(base.clone(), name, value)?);
    }
    let root = change_shape_and_reseal_models(base, "Single", |shape| {
        map_mut(shape)?.insert(
            Key::Field(CatalogKey::EnumValues.field()?),
            Value::List(vec![Value::Text("wire".into())]),
        );
        Ok(())
    })?;
    rejected(&Record::encode(&root)?.into_bytes());
    Ok(())
}

#[test]
fn given_unused_enum_count_when_below_at_or_above_limit_then_public_loader_enforces_exact_bound()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    for count in [65_535, 65_536] {
        let bytes = domain(
            base.clone(),
            "Legacy",
            Value::List(vec![Value::Text("wire".into()); count]),
        )?;
        accepted(&bytes, "Legacy", ValueRef::Text("wire"), true)?;
    }
    rejected(&domain(
        base,
        "Legacy",
        Value::List(vec![Value::Text("wire".into()); 65_537]),
    )?);
    Ok(())
}

#[test]
fn given_unused_enum_text_when_utf8_bytes_cross_limit_then_public_loader_enforces_exact_bound()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    for value in [format!("{}a", "é".repeat(8_191)), "é".repeat(8_192)] {
        let bytes = domain(
            base.clone(),
            "Legacy",
            Value::List(vec![Value::Text(value.clone())]),
        )?;
        accepted(&bytes, "Legacy", ValueRef::Text(&value), true)?;
    }
    rejected(&domain(
        base,
        "Legacy",
        Value::List(vec![Value::Text(format!("{}a", "é".repeat(8_192)))]),
    )?);
    Ok(())
}

#[test]
fn given_unused_enum_table_when_total_text_crosses_limit_then_public_loader_enforces_exact_bound()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    let value = "x".repeat(16_384);
    let mut below = vec![Value::Text(value.clone()); 63];
    below.push(Value::Text("x".repeat(16_383)));
    accepted(
        &domain(base.clone(), "Legacy", Value::List(below))?,
        "Legacy",
        ValueRef::Text(&value),
        true,
    )?;
    let mut at = vec![Value::Text(value.clone()); 64];
    accepted(
        &domain(base.clone(), "Legacy", Value::List(at.clone()))?,
        "Legacy",
        ValueRef::Text(&value),
        true,
    )?;
    at.push(Value::Text("x".into()));
    rejected(&domain(base, "Legacy", Value::List(at))?);
    Ok(())
}

#[test]
fn given_unused_empty_duplicate_and_wildcard_domains_when_loaded_then_set_membership_is_literal()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    accepted(
        &domain(base.clone(), "Legacy", Value::List(vec![]))?,
        "Legacy",
        ValueRef::Text("anything"),
        false,
    )?;
    let bytes = domain(
        base,
        "Legacy",
        Value::List(vec![
            Value::Text("*".into()),
            Value::Text("*".into()),
            Value::Text("wire".into()),
        ]),
    )?;
    accepted(&bytes, "Legacy", ValueRef::Text("*"), true)?;
    accepted(&bytes, "Legacy", ValueRef::Text("wire"), true)?;
    accepted(&bytes, "Legacy", ValueRef::Text("anything"), false)
}

#[test]
fn given_historical_unknown_string_domains_when_unused_then_readable_but_scalar_list_and_map_key_use_fail_closed()
-> TestResult {
    let base = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    let unused = historical(base.clone(), "Legacy")?;
    accepted(&unused, "Legacy", ValueRef::Text("owned"), false)?;
    let scalar = compiled(
        "aws.protocols#awsJson1_0",
        with_field("Legacy", FixedValue::String("owned".into()))?,
    )?;
    for (bytes, name) in [
        (scalar, "Legacy"),
        (base.clone(), "Engine"),
        (base, "Owner"),
    ] {
        let bytes = historical(bytes, name)?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

#[test]
fn given_ordinary_named_string_when_explicit_unconstrained_marker_is_compiled_then_signing_succeeds_but_marker_deletion_rejects_use()
-> TestResult {
    let bytes = compiled(
        "aws.protocols#awsJson1_0",
        with_field("Label", FixedValue::String("ok-7".into()))?,
    )?;
    let request = signed_bytes(&bytes)?;
    let body: serde_json::Value = serde_json::from_slice(request.body())?;
    assert_eq!(body.pointer("/Filters/0/Label"), Some(&json!("ok-7")));
    let root = change_shape_and_reseal_models(bytes, "Label", |shape| {
        assert_eq!(
            field_mut(shape, CatalogKey::EnumValues.field()?.value())?,
            &Value::Null
        );
        map_mut(shape)?
            .remove(&Key::Field(CatalogKey::EnumValues.field()?))
            .ok_or("marker absent")?;
        Ok(())
    })?;
    let bytes = Record::encode(&root)?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
