use super::*;

fn rewrite(
    bytes: &[u8],
    change: impl FnOnce(&mut BTreeMap<Key, Value>) -> TestResult,
) -> TestResult<Vec<u8>> {
    let mut value = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let Value::Map(root) = &mut value else {
        return Err("root is not map".into());
    };
    let Some(Value::Map(extension)) = root.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
        return Err("extension absent".into());
    };
    change(extension)?;
    Ok(Record::encode(&value)?.into_bytes())
}

#[test]
fn given_old_extension_schema_when_loaded_then_reader_rejects_without_compatibility_shim()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for version in [1, 3] {
        let bytes = rewrite(&bytes, |extension| {
            extension.insert(Key::Field(FieldId::new(1)?), Value::Int(version));
            Ok(())
        })?;
        assert!(
            matches!(RuntimeCatalog::from_bytes(&bytes), Err(RuntimeCatalogError::ExtensionVersion(actual)) if actual == version)
        );
    }
    Ok(())
}

#[test]
fn given_tampered_numeric_tag_shape_when_loaded_then_model_mismatch_is_rejected() -> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for (field, replacement) in [
        (1, Value::Int(1)),
        (2, Value::List(vec![Value::Int(2)])),
        (3, Value::Int(2)),
        (4, Value::Int(4)),
    ] {
        let bytes = rewrite(&bytes, |extension| {
            let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
                return Err("plans absent".into());
            };
            let Some(Value::Map(plan)) = plans.first_mut() else {
                return Err("plan absent".into());
            };
            let Some(Value::Map(tags)) = plan.get_mut(&Key::Field(FieldId::new(14)?)) else {
                return Err("tags absent".into());
            };
            tags.insert(Key::Field(FieldId::new(field)?), replacement);
            Ok(())
        })?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

#[test]
fn given_nonmap_tag_metadata_when_loaded_then_invalid_type_is_not_treated_as_unobserved()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for replacement in [Value::Null, Value::Int(0), Value::List(Vec::new())] {
        let bytes = rewrite(&bytes, |extension| {
            let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
                return Err("plans absent".into());
            };
            let Some(Value::Map(plan)) = plans.first_mut() else {
                return Err("plan absent".into());
            };
            plan.insert(Key::Field(FieldId::new(14)?), replacement);
            Ok(())
        })?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

#[test]
fn given_map_tag_metadata_pointing_to_list_shape_when_loaded_then_shape_kind_mismatch_is_rejected()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    let bytes = rewrite(&bytes, |extension| {
        let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
            return Err("plans absent".into());
        };
        let Some(Value::Map(plan)) = plans.first_mut() else {
            return Err("plan absent".into());
        };
        let Some(Value::Map(tags)) = plan.get_mut(&Key::Field(FieldId::new(14)?)) else {
            return Err("tags absent".into());
        };
        tags.insert(Key::Field(FieldId::new(1)?), Value::Int(1));
        tags.remove(&Key::Field(FieldId::new(3)?));
        tags.remove(&Key::Field(FieldId::new(4)?));
        Ok(())
    })?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
