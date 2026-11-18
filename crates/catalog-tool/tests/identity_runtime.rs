#[path = "support/identity.rs"]
mod fixture;

use fixture::{TestResult, arn, compiled, compiled_fixed, key};
use inventory_core::catalog::runtime::{
    BindingSource, BindingValue, IdentityPart, IdentityPlan, RuntimeCatalog,
};
use inventory_core::{inventory::ScalarType, record::FieldId};

#[test]
fn given_enum_identity_when_compiled_then_text_identity_paths_roundtrip() -> TestResult {
    let path = vec![FieldId::new(6)?].into_boxed_slice();
    [
        (
            key("Name")?,
            IdentityPlan::Key(vec![path.clone()].into_boxed_slice()),
        ),
        (arn("Name")?, IdentityPlan::Arn(path.clone())),
        (
            catalog_tool::overlay::Identity::Template("name/{Name}".to_owned()),
            IdentityPlan::Template(
                vec![
                    IdentityPart::Literal("name/".to_owned()),
                    IdentityPart::Field(path),
                ]
                .into_boxed_slice(),
            ),
        ),
    ]
    .into_iter()
    .try_for_each(|(identity, expected)| {
        let bytes = compiled(identity, "active-wire", false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("things", "ListThings")
            .ok_or("plan absent")?;
        assert_eq!(plan.identity(), &expected);
        assert!(
            matches!(&plan.bindings().first().ok_or("fixed binding absent")?.source,
            BindingSource::Fixed(BindingValue::String(value)) if value == "active-wire")
        );
        Ok(())
    })
}

#[test]
fn given_enum_self_identity_when_compiled_then_selected_text_schema_roundtrips() -> TestResult {
    [
        (
            key("$")?,
            IdentityPlan::Key(vec![Box::<[FieldId]>::default()].into_boxed_slice()),
        ),
        (arn("$")?, IdentityPlan::Arn(Box::new([]))),
        (
            catalog_tool::overlay::Identity::Template("prefix/{$}".to_owned()),
            IdentityPlan::Template(
                vec![
                    IdentityPart::Literal("prefix/".to_owned()),
                    IdentityPart::Field(Box::new([])),
                ]
                .into_boxed_slice(),
            ),
        ),
    ]
    .into_iter()
    .try_for_each(|(identity, expected)| {
        let bytes = compiled(identity, "second-wire", true)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("things", "ListThings")
            .ok_or("plan absent")?;
        assert_eq!(plan.identity(), &expected);
        let schema = catalog
            .schemas()
            .iter()
            .find(|schema| schema.kind().as_str() == "AWS::ApiOnly::Identity")
            .ok_or("schema absent")?;
        let column = schema.columns().first().ok_or("scalar column absent")?;
        assert!(column.path().is_empty());
        assert_eq!(column.scalar(), ScalarType::Text);
        Ok(())
    })
}

#[test]
fn given_template_when_compiled_then_context_tags_and_field_order_survive_loader() -> TestResult {
    let bytes = compiled(
        catalog_tool::overlay::Identity::Template(
            "arn:{partition}:s3:{region}:{account}:bucket/{Name}".to_owned(),
        ),
        "active-wire",
        false,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("things", "ListThings")
        .ok_or("plan absent")?;
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Template(
            vec![
                IdentityPart::Literal("arn:".to_owned()),
                IdentityPart::Partition,
                IdentityPart::Literal(":s3:".to_owned()),
                IdentityPart::Region,
                IdentityPart::Literal(":".to_owned()),
                IdentityPart::Account,
                IdentityPart::Literal(":bucket/".to_owned()),
                IdentityPart::Field(vec![FieldId::new(6)?].into_boxed_slice()),
            ]
            .into_boxed_slice()
        )
    );
    Ok(())
}

#[test]
fn given_unknown_enum_literal_when_checked_and_augmented_then_compilation_fails() -> TestResult {
    ["ACTIVE", "unknown", "active-wire "]
        .into_iter()
        .try_for_each(|literal| -> TestResult {
            assert!(compiled(key("Name")?, literal, false).is_err());
            Ok(())
        })
}

#[test]
fn given_enum_list_filter_when_compiled_then_exact_wire_literals_are_retained() -> TestResult {
    let bytes = compiled_fixed(
        key("Name")?,
        "active-wire",
        &["second-wire", "active-wire"],
        false,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("things", "ListThings")
        .ok_or("plan absent")?;
    assert!(plan.bindings().iter().any(|binding| matches!(&binding.source,
        BindingSource::Fixed(BindingValue::Strings(values)) if values.as_ref() == ["second-wire", "active-wire"])));
    assert!(
        compiled_fixed(
            key("Name")?,
            "active-wire",
            &["second-wire", "SECOND"],
            false
        )
        .is_err()
    );
    assert!(compiled_fixed(key("Name")?, "active-wire", &["unknown"], false).is_err());
    Ok(())
}

#[test]
fn given_invalid_template_paths_when_checked_then_compile_denies_them() {
    ["{unknown}", "{Name}{", "{Items}", "{$}", "{account.bad}"]
        .into_iter()
        .for_each(|identity| {
            assert!(
                compiled(
                    catalog_tool::overlay::Identity::Template(identity.to_owned()),
                    "active-wire",
                    false
                )
                .is_err()
            )
        });
}

#[test]
fn given_singleton_structure_selection_when_compiled_then_identity_is_relative_to_item()
-> TestResult {
    let bytes = fixture::compiled_singleton(key("Name")?)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("things", "ListThings")
        .ok_or("plan absent")?;
    assert_eq!(
        plan.items(),
        &inventory_core::catalog::runtime::ItemPlan::Path(
            vec![FieldId::new(3)?].into_boxed_slice(),
        )
    );
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Key(vec![vec![FieldId::new(6)?].into_boxed_slice()].into_boxed_slice(),)
    );
    Ok(())
}

#[test]
fn given_bounded_fixed_lists_when_compiled_then_empty_and_more_than_sixteen_values_roundtrip_exactly()
-> TestResult {
    for count in [0, 16, 17, 64] {
        let values = vec!["active-wire"; count];
        let bytes = compiled_fixed(key("Name")?, "active-wire", &values, false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("things", "ListThings")
            .ok_or("plan absent")?;
        let actual = plan
            .bindings()
            .iter()
            .find_map(|binding| match &binding.source {
                BindingSource::Fixed(BindingValue::Strings(values)) => Some(values.as_ref()),
                _ => None,
            })
            .ok_or("fixed list absent")?;
        assert_eq!(actual, vec!["active-wire".to_owned(); count].as_slice());
    }
    let error = compiled_fixed(key("Name")?, "active-wire", &["active-wire"; 65], false)
        .err()
        .ok_or("oversized fixed list was admitted")?;
    let Some(error) = error.downcast_ref::<catalog_tool::overlay::NameError>() else {
        return Err(format!("unexpected oversized-list error: {error}").into());
    };
    assert_eq!(error.value, "fixed: literal bounds exceeded");
    Ok(())
}

fn map_mut(
    value: &mut inventory_core::record::Value,
) -> TestResult<
    &mut std::collections::BTreeMap<inventory_core::record::Key, inventory_core::record::Value>,
> {
    match value {
        inventory_core::record::Value::Map(map) => Ok(map),
        _ => Err("fixture value is not a map".into()),
    }
}

fn tampered_capability(
    bytes: &[u8],
    change: impl FnOnce(
        &mut std::collections::BTreeMap<inventory_core::record::Key, inventory_core::record::Value>,
    ) -> TestResult,
) -> TestResult<inventory_core::record::Record> {
    use inventory_core::record::{Key, MapRef, Record, Value, ValueRef};
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = map_mut(
        map_mut(&mut root)?
            .get_mut(&Key::Field(FieldId::new(60_000)?))
            .ok_or("extension absent")?,
    )?;
    let Value::List(plans) = extension
        .get_mut(&Key::Field(FieldId::new(2)?))
        .ok_or("plans absent")?
    else {
        return Err("plans not a list".into());
    };
    change(map_mut(plans.first_mut().ok_or("first plan absent")?)?)?;
    let encoded = Record::encode(&Value::Map(extension.clone()))?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    let receipt = map_mut(
        extension
            .get_mut(&Key::Field(FieldId::new(7)?))
            .ok_or("receipt absent")?,
    )?;
    receipt.insert(Key::Field(FieldId::new(7)?), Value::Bytes(digest.to_vec()));
    Ok(Record::encode(&root)?)
}

#[test]
fn given_rehashed_numeric_capability_tampering_when_loaded_then_model_boundary_rejects_it()
-> TestResult {
    use inventory_core::{
        catalog::runtime::RuntimeCatalogError,
        record::{Key, Value},
    };
    let bytes = compiled(key("Name")?, "active-wire", false)?;
    for corruption in 0..3 {
        let record = tampered_capability(&bytes, |plan| {
            match corruption {
                0 => {
                    let identity = map_mut(
                        plan.get_mut(&Key::Field(FieldId::new(6)?))
                            .ok_or("identity absent")?,
                    )?;
                    identity.insert(
                        Key::Field(FieldId::new(2)?),
                        Value::List(vec![Value::List(vec![Value::Int(999)])]),
                    );
                }
                1 => {
                    let Value::List(bindings) = plan
                        .get_mut(&Key::Field(FieldId::new(7)?))
                        .ok_or("bindings absent")?
                    else {
                        return Err("bindings not a list".into());
                    };
                    let binding = map_mut(bindings.first_mut().ok_or("binding absent")?)?;
                    binding.insert(
                        Key::Field(FieldId::new(3)?),
                        Value::Text("not-a-wire-enum".to_owned()),
                    );
                }
                _ => {
                    let page = map_mut(
                        plan.get_mut(&Key::Field(FieldId::new(8)?))
                            .ok_or("pagination absent")?,
                    )?;
                    page.insert(
                        Key::Field(FieldId::new(1)?),
                        Value::Map(std::collections::BTreeMap::from([(
                            Key::Field(FieldId::new(1)?),
                            Value::Int(0),
                        )])),
                    );
                }
            }
            Ok(())
        })?;
        assert!(
            matches!(
                RuntimeCatalog::from_bytes(record.as_bytes()),
                Err(RuntimeCatalogError::Capability)
            ),
            "corruption {corruption} was not rejected at the model boundary"
        );
    }
    Ok(())
}

#[test]
fn given_current_response_or_request_identity_context_when_loaded_then_exact_projection_survives()
-> TestResult {
    use catalog_tool::overlay::{
        ContextSource, Identity, IdentityField, MemberPath, OperationName,
    };
    use inventory_core::catalog::runtime::{ContextSource as RuntimeSource, IdentityFieldPlan};
    for (source, expected_source) in [
        (ContextSource::Response, RuntimeSource::Response),
        (ContextSource::Request, RuntimeSource::Request),
    ] {
        let identity = Identity::CorrelatedKey(vec![
            IdentityField::Context {
                operation: OperationName::new("ListThings")?,
                source,
                path: MemberPath::new("Mode")?,
            },
            IdentityField::Selected(MemberPath::new("Name")?),
        ]);
        let bytes = compiled(identity, "active-wire", false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("things", "ListThings")
            .ok_or("plan absent")?;
        let IdentityPlan::CorrelatedKey(fields) = plan.identity() else {
            return Err("correlated identity absent".into());
        };
        let Some(IdentityFieldPlan::Context {
            service,
            operation,
            source,
            path,
        }) = fields.first()
        else {
            return Err("current context field absent".into());
        };
        assert_eq!(
            (service.as_ref(), operation.as_ref(), *source),
            ("things", "ListThings", expected_source)
        );
        let projected = match expected_source {
            RuntimeSource::Response => &plan.context_projection().response,
            RuntimeSource::Request => &plan.context_projection().request,
            RuntimeSource::SelectedItem => return Err("unexpected fixture context".into()),
        };
        assert_eq!(projected.as_ref(), [path.clone()]);
    }
    Ok(())
}

#[test]
fn given_current_selected_item_as_context_when_compiled_then_it_is_rejected() -> TestResult {
    use catalog_tool::overlay::{
        ContextSource, Identity, IdentityField, MemberPath, OperationName,
    };
    let identity = Identity::CorrelatedKey(vec![IdentityField::Context {
        operation: OperationName::new("ListThings")?,
        source: ContextSource::SelectedItem,
        path: MemberPath::new("Name")?,
    }]);
    assert!(compiled(identity, "active-wire", false).is_err());
    Ok(())
}

#[test]
fn given_current_request_or_response_binding_when_compiled_then_future_context_is_rejected()
-> TestResult {
    use catalog_tool::overlay::ContextSource;
    for source in [ContextSource::Request, ContextSource::Response] {
        assert!(fixture::compiled_self_binding(source).is_err());
    }
    Ok(())
}

#[test]
fn given_scalar_presence_identity_when_compiled_then_integral_and_boolean_predicates_roundtrip()
-> TestResult {
    use catalog_tool::overlay::{
        Identity, IdentityBranch, IdentityChoice, IdentityLeaf, IdentityPredicate, MemberPath,
    };
    use inventory_core::catalog::runtime::{IdentityChoicePlan, IdentityPredicatePlan};
    for (path, id) in [("Count", 1), ("Flag", 2)] {
        let identity = Identity::Choice(IdentityChoice::Discriminated(vec![IdentityBranch::new(
            IdentityPredicate::Present(MemberPath::new(path)?),
            IdentityLeaf::Key(vec![MemberPath::new("Name")?]),
        )?]));
        let bytes = compiled(identity, "active-wire", false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("things", "ListThings")
            .ok_or("plan absent")?;
        let IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) = plan.identity()
        else {
            return Err("identity not discriminated".into());
        };
        assert_eq!(branches.len(), 1);
        assert_eq!(
            branches.first().ok_or("branch absent")?.predicate,
            IdentityPredicatePlan::Present(vec![FieldId::new(id)?].into_boxed_slice())
        );
    }
    Ok(())
}
