use super::*;
#[test]
fn given_nested_scalar_constraints_when_compiled_then_ranges_lengths_and_unicode_are_checked()
-> TestResult {
    let float = |value| overlay::FixedValue::Float(inventory_core::record::Float::new(value));
    let cases = [
        ("Ratio", float(0.5), true),
        ("Ratio", float(2.0), false),
        ("Payload", overlay::FixedValue::Bytes(vec![1, 2]), true),
        ("Payload", overlay::FixedValue::Bytes(vec![1, 2, 3]), false),
        ("Unicode", overlay::FixedValue::String("éé".into()), true),
        ("Unicode", overlay::FixedValue::String("ééé".into()), false),
    ];
    cases.into_iter().try_for_each(|(name, value, permitted)| {
        let result = compiled("aws.protocols#awsJson1_0", with_field(name, value)?);
        assert_eq!(result.is_ok(), permitted, "{name}: {result:?}");
        if let Ok(bytes) = result {
            RuntimeCatalog::from_bytes(&bytes)?;
        }
        Ok(())
    })
}
#[test]
fn given_forged_literal_tree_when_loaded_then_required_members_and_types_are_rechecked()
-> TestResult {
    let bytes = compiled("aws.protocols#awsQuery", filters("neptune")?)?;
    let record = Record::from_bytes(bytes)?;
    let original = Value::Map(
        record
            .root()
            .iter()?
            .map(|entry| {
                let (key, value) = entry?;
                Ok((key.to_key(), value.to_value()?))
            })
            .collect::<Result<_, inventory_core::record::RecordError>>()?,
    );
    let path_key = Key::Field(FieldId::new(1)?);
    [
        Value::List(vec![Value::Map(BTreeMap::new())]),
        Value::List(vec![Value::Bool(true)]),
    ]
    .into_iter()
    .try_for_each(|forged| {
        let mut root = original.clone();
        let plans = field_mut(field_mut(&mut root, 60_000)?, 2)?;
        let bindings = field_mut(first_mut(plans)?, 7)?;
        let Value::List(bindings) = bindings else {
            return Err("bindings absent".into());
        };
        let target = bindings
            .iter_mut()
            .find(|binding| match binding {
                Value::Map(fields) => fields
                    .get(&path_key)
                    .is_some_and(|path| *path == Value::List(vec![Value::Int(3)])),
                _ => false,
            })
            .ok_or("Filters binding absent")?;
        *field_mut(target, 3)? = forged;
        reseal(&mut root)?;
        let bytes = Record::encode(&root)?.into_bytes();
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
        Ok(())
    })
}
