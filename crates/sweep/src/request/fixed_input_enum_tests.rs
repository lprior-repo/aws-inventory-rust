use super::{TestResult, compiled, filters, signed_bytes, with_field};
use catalog_tool::overlay::FixedValue;
use inventory_core::{
    catalog::{
        CatalogKey,
        runtime::{RuntimeCatalog, RuntimeCatalogError},
    },
    record::{FieldId, Key, ListRef, Record, Value, ValueRef, encode_list},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type ValueResult<'a> = Result<&'a mut Value, Box<dyn std::error::Error>>;

fn literal_mut(root: &mut Value, input: u32) -> ValueResult<'_> {
    let plans = super::field_mut(super::field_mut(root, 60_000)?, 2)?;
    let bindings = super::field_mut(super::first_mut(plans)?, 7)?;
    let Value::List(bindings) = bindings else {
        return Err("bindings missing".into());
    };
    let path_key = Key::Field(FieldId::new(1)?);
    let binding = bindings
        .iter_mut()
        .find(|binding| match binding {
            Value::Map(fields) => fields
                .get(&path_key)
                .is_some_and(|path| *path == Value::List(vec![Value::Int(i64::from(input))])),
            _ => false,
        })
        .ok_or("fixed binding absent")?;
    super::field_mut(binding, 3)
}
fn filter_field(root: &mut Value, field: u32) -> ValueResult<'_> {
    let Value::List(filters) = literal_mut(root, 3)? else {
        return Err("Filters list absent".into());
    };
    super::field_mut(filters.first_mut().ok_or("first filter absent")?, field)
}
fn reject_resealed(bytes: &[u8], change: impl FnOnce(&mut Value) -> TestResult) -> TestResult {
    let record = Record::from_bytes(bytes.to_vec())?;
    let mut root = Value::Map(
        record
            .root()
            .iter()?
            .map(|entry| {
                let (key, value) = entry?;
                Ok((key.to_key(), value.to_value()?))
            })
            .collect::<Result<_, inventory_core::record::RecordError>>()?,
    );
    change(&mut root)?;
    super::reseal(&mut root)?;
    let forged = Record::encode(&root)?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&forged),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_valid_enums_when_scalar_and_aggregate_literals_are_resealed_then_forged_values_are_rejected()
-> TestResult {
    for (name, field, source, expected, forged) in [
        (
            "Engine",
            2,
            FixedValue::String("neptune".into()),
            json!("neptune"),
            Value::Text("unknown".into()),
        ),
        ("Mode", 8, FixedValue::Integer(1), json!(1), Value::Int(6)),
        (
            "Legacy",
            7,
            FixedValue::String("owned".into()),
            json!("owned"),
            Value::Text("other".into()),
        ),
        (
            "Items",
            5,
            FixedValue::list([FixedValue::Integer(1), FixedValue::Integer(7)]),
            json!([1, 7]),
            Value::List(vec![Value::Int(6)]),
        ),
        (
            "Attributes",
            1,
            FixedValue::map([("owner", FixedValue::Integer(1))])?,
            json!({"owner":1}),
            Value::Map(BTreeMap::from([(Key::Name("owner".into()), Value::Int(6))])),
        ),
    ] {
        let bytes = compiled("aws.protocols#awsJson1_0", with_field(name, source)?)?;
        let request = signed_bytes(&bytes)?;
        assert!(request.headers().contains_key("authorization"));
        let body: serde_json::Value = serde_json::from_slice(request.body())?;
        assert_eq!(
            body.pointer("/Filters/0")
                .and_then(|filter| filter.get(name)),
            Some(&expected)
        );
        reject_resealed(&bytes, |root| {
            *filter_field(root, field)? = forged;
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn given_string_enum_list_when_valid_input_is_signed_then_resealed_unknown_element_is_rejected()
-> TestResult {
    let bytes = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    let request = signed_bytes(&bytes)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(request.body())?.pointer("/Filters/0/Values"),
        Some(&json!(["neptune"]))
    );
    reject_resealed(&bytes, |root| {
        *filter_field(root, 16)? = Value::List(vec![Value::Text("not-a-wire-enum".into())]);
        Ok(())
    })
}

#[test]
fn given_enum_map_keys_when_valid_input_is_signed_then_resealed_unknown_keys_are_rejected()
-> TestResult {
    let bytes = compiled("aws.protocols#awsJson1_0", filters("neptune")?)?;
    let request = signed_bytes(&bytes)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(request.body())?.get("Attributes"),
        Some(&json!({"owner":"inventory"}))
    );
    reject_resealed(&bytes, |root| {
        *literal_mut(root, 1)? = Value::Map(BTreeMap::from([(
            Key::Name("not-an-owner".into()),
            Value::Text("inventory".into()),
        )]));
        Ok(())
    })?;
    let bytes = compiled(
        "aws.protocols#awsJson1_0",
        with_field(
            "Attributes",
            FixedValue::map([("owner", FixedValue::Integer(1))])?,
        )?,
    )?;
    reject_resealed(&bytes, |root| {
        *filter_field(root, 1)? = Value::Map(BTreeMap::from([(
            Key::Name("not-an-owner".into()),
            Value::Int(1),
        )]));
        Ok(())
    })
}

fn change_shape_and_reseal_models(
    bytes: Vec<u8>,
    name: &str,
    change: impl FnOnce(&mut Value) -> TestResult,
) -> Result<Value, Box<dyn std::error::Error>> {
    let record = Record::from_bytes(bytes)?;
    let mut root = ValueRef::Map(record.root()).to_value()?;
    let encoded = super::first_mut(super::field_mut(
        &mut root,
        CatalogKey::Services.field()?.value(),
    )?)?;
    let Value::Bytes(bytes) = encoded else {
        return Err("service bytes missing".into());
    };
    let service_record = Record::from_bytes(bytes.clone())?;
    let mut service = ValueRef::Map(service_record.root()).to_value()?;
    let Value::List(shapes) = super::field_mut(&mut service, CatalogKey::Shapes.field()?.value())?
    else {
        return Err("shapes missing".into());
    };
    change(named_shape(shapes, name)?)?;
    let shapes_bytes = encode_list(shapes)?;
    let offsets = ListRef::from_bytes(&shapes_bytes)?
        .encoded_item_offsets()?
        .iter()
        .flat_map(|offset| offset.to_be_bytes())
        .collect();
    *super::field_mut(&mut service, CatalogKey::ShapeOffsets.field()?.value())? =
        Value::Bytes(offsets);
    *encoded = Value::Bytes(Record::encode(&service)?.into_bytes());
    reseal_models(&mut root)?;
    Ok(root)
}
fn named_shape<'a>(shapes: &'a mut [Value], name: &str) -> ValueResult<'a> {
    let name_key = Key::Field(CatalogKey::Name.field()?);
    shapes
        .iter_mut()
        .find(|shape| match shape {
            Value::Map(fields) => {
                matches!(fields.get(&name_key), Some(Value::Text(found)) if found == name)
            }
            _ => false,
        })
        .ok_or_else(|| "named shape missing".into())
}
fn reseal_models(root: &mut Value) -> TestResult {
    let mut base = root.clone();
    super::map_mut(&mut base)?
        .remove(&Key::Field(FieldId::new(60_000)?))
        .ok_or("extension missing")?;
    let digest = Sha256::digest(Record::encode(&base)?.into_bytes());
    let provenance = super::field_mut(super::field_mut(root, 60_000)?, 7)?;
    *super::field_mut(super::field_mut(provenance, 2)?, 2)? = Value::Bytes(digest.to_vec());
    super::reseal(root)
}
fn kind_value(kind: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(Value::Int(i64::try_from(
        inventory_core::catalog::kind_code(kind).ok_or("shape kind missing")?,
    )?))
}
fn remove_legacy_domain_and_reseal_models(
    bytes: Vec<u8>,
) -> Result<Value, Box<dyn std::error::Error>> {
    change_shape_and_reseal_models(bytes, "Legacy", |legacy| {
        let fields = super::map_mut(legacy)?;
        assert_eq!(
            fields.get(&Key::Field(CatalogKey::Kind.field()?)),
            Some(&kind_value("enum")?)
        );
        fields
            .remove(&Key::Field(CatalogKey::EnumValues.field()?))
            .ok_or("enum domain missing")?;
        Ok(())
    })
}

#[test]
fn given_legacy_enum_domain_deletion_when_both_receipts_are_resealed_then_only_unused_domains_are_accepted()
-> TestResult {
    let unused = remove_legacy_domain_and_reseal_models(compiled(
        "aws.protocols#awsJson1_0",
        filters("neptune")?,
    )?)?;
    let bytes = Record::encode(&unused)?.into_bytes();
    RuntimeCatalog::from_bytes(&bytes)?;

    let used = compiled(
        "aws.protocols#awsJson1_0",
        with_field("Legacy", FixedValue::String("owned".into()))?,
    )?;
    let mut root = remove_legacy_domain_and_reseal_models(used)?;
    let bytes = Record::encode(&root)?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    *filter_field(&mut root, 7)? = Value::Text("not-a-legacy-wire-enum".into());
    super::reseal(&mut root)?;
    let bytes = Record::encode(&root)?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[path = "fixed_input_metadata_tests.rs"]
mod metadata;

#[path = "fixed_input_source_enum_tests.rs"]
mod source_membership;
