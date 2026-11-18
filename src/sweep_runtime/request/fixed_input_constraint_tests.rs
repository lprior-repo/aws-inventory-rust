use super::{TestResult, compiled, filters, signed_literal, with_field};
use catalog_tool::overlay::FixedValue;
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeCatalogError},
    record::{FieldId, Float, Key, Record, Value},
};
use serde_json::json;

#[test]
fn given_duplicate_names_when_constructing_structures_and_maps_then_both_are_rejected() -> TestResult
{
    let structure = FixedValue::structure([
        ("Name", FixedValue::String("engine".into())),
        ("Name", FixedValue::String("instance".into())),
    ])
    .err()
    .ok_or("duplicate structure key accepted")?;
    assert_eq!(structure.value, "duplicate fixed-input key: Name");
    let map = FixedValue::map([
        ("owner", FixedValue::String("inventory".into())),
        ("owner", FixedValue::String("other".into())),
    ])
    .err()
    .ok_or("duplicate map key accepted")?;
    assert_eq!(map.value, "duplicate fixed-input key: owner");
    Ok(())
}

#[test]
fn given_model_enums_and_patterns_when_literals_are_signed_then_only_conforming_values_reach_wire()
-> TestResult {
    for (field, value, wire_value, accepted) in [
        ("Mode", FixedValue::Integer(1), json!(1), true),
        ("Mode", FixedValue::Integer(7), json!(7), true),
        ("Mode", FixedValue::Integer(0), json!(0), false),
        ("Mode", FixedValue::Integer(6), json!(6), false),
        (
            "Legacy",
            FixedValue::String("owned".into()),
            json!("owned"),
            true,
        ),
        (
            "Legacy",
            FixedValue::String("shared".into()),
            json!("shared"),
            true,
        ),
        (
            "Legacy",
            FixedValue::String("owner".into()),
            json!("owner"),
            false,
        ),
        (
            "Label",
            FixedValue::String("ok-7".into()),
            json!("ok-7"),
            true,
        ),
        (
            "Label",
            FixedValue::String("ok-8".into()),
            json!("ok-8"),
            false,
        ),
        (
            "Label",
            FixedValue::String("bad-7".into()),
            json!("bad-7"),
            false,
        ),
        (
            "Unsupported",
            FixedValue::String("a".into()),
            json!("a"),
            false,
        ),
    ] {
        let result = signed_literal("aws.protocols#awsJson1_0", with_field(field, value)?);
        if accepted {
            let request = result?;
            assert!(request.headers().contains_key("authorization"));
            let mut expected = json!({"Attributes":{"owner":"inventory"},"Filters":[{"Name":"engine","Values":["neptune"]}]});
            expected
                .pointer_mut("/Filters/0")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or("expected structure missing")?
                .insert(field.into(), wire_value);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(request.body())?,
                expected
            );
        } else {
            super::assert_invalid_value(
                result.err().ok_or("invalid scalar reached wire")?.as_ref(),
            )?;
        }
    }
    Ok(())
}

#[test]
fn given_float_width_boundaries_when_signed_then_f32_endpoints_and_f64_max_keep_their_wire_values()
-> TestResult {
    for (number, bits) in [
        (f64::from(f32::MAX), [0x7f, 0x7f, 0xff, 0xff]),
        (-f64::from(f32::MAX), [0xff, 0x7f, 0xff, 0xff]),
    ] {
        let request = signed_literal(
            "smithy.protocols#rpcv2Cbor",
            with_field("Single", FixedValue::Float(Float::new(number)))?,
        )?;
        let mut expected =
            b"\xbfjAttributes\xbfeowneriinventory\xffgFilters\x9f\xbfdNamefenginefSingle\xfa"
                .to_vec();
        expected.extend_from_slice(&bits);
        expected.extend_from_slice(b"fValues\x9fgneptune\xff\xff\xff\xff");
        assert_eq!(request.body().as_ref(), expected);
        assert!(request.headers().contains_key("authorization"));
    }
    for number in [
        f64::from(f32::MAX).next_up(),
        -f64::from(f32::MAX).next_up(),
        f64::MAX,
    ] {
        let error = signed_literal(
            "smithy.protocols#rpcv2Cbor",
            with_field("Single", FixedValue::Float(Float::new(number)))?,
        )
        .err()
        .ok_or("out-of-width float reached wire")?;
        super::assert_invalid_value(error.as_ref())?;
    }
    let request = signed_literal(
        "aws.protocols#awsJson1_0",
        with_field("Wide", FixedValue::Float(Float::new(f64::MAX)))?,
    )?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(request.body())?
            .pointer("/Filters/0/Wide")
            .and_then(serde_json::Value::as_f64),
        Some(f64::MAX)
    );
    assert!(request.headers().contains_key("authorization"));
    Ok(())
}

#[test]
fn given_forged_float_outside_f32_width_when_catalog_loaded_then_capability_rejects_it()
-> TestResult {
    let bytes = compiled("smithy.protocols#rpcv2Cbor", filters("neptune")?)?;
    let record = Record::from_bytes(bytes)?;
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
    let plans = super::field_mut(super::field_mut(&mut root, 60_000)?, 2)?;
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
                .is_some_and(|path| *path == Value::List(vec![Value::Int(3)])),
            _ => false,
        })
        .ok_or("Filters binding absent")?;
    let fixed = super::field_mut(binding, 3)?;
    let Value::List(filters) = fixed else {
        return Err("filter list missing".into());
    };
    let Value::Map(filter) = filters.first_mut().ok_or("filter missing")? else {
        return Err("filter structure missing".into());
    };
    filter.insert(
        Key::Field(FieldId::new(12)?),
        Value::Float(Float::new(f64::MAX)),
    );
    super::reseal(&mut root)?;
    let forged = Record::encode(&root)?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&forged),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
