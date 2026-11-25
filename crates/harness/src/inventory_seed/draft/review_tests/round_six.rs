use super::*;

fn draft(extra: serde_json::Value) -> TestResult<String> {
    Ok(generate(
        &RuntimeCatalog::from_bytes(&fixture_runtime(extra)?)?,
        "example",
    )?)
}

fn input(text: &str, operation: &str) -> TestResult<Value> {
    text.parse::<Table>()?
        .get("step")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|step| step.get("operation").and_then(Value::as_str) == Some(operation))
        .and_then(|step| step.get("input"))
        .cloned()
        .ok_or_else(|| "enabled input".into())
}

fn integer_draft(
    shape_traits: serde_json::Value,
    member_traits: serde_json::Value,
) -> TestResult<String> {
    let mut traits = member_traits.as_object().ok_or("member traits")?.clone();
    traits.insert("smithy.api#required".into(), json!({}));
    draft(json!({
        "example#PutThing":{"type":"operation","input":{"target":"example#RangeInput"},"output":{"target":"example#Output"}},
        "example#RangeInput":{"type":"structure","members":{"Count":{"target":"example#Count","traits":traits}}},
        "example#Count":{"type":"long","traits":shape_traits}
    }))
}

#[test]
fn given_ambiguous_plural_endings_when_matching_then_all_six_valid_singulars_survive() -> TestResult
{
    for (plural, singular) in [
        ("Aliases", "Alias"),
        ("Statuses", "Status"),
        ("Caches", "Cache"),
        ("Buses", "Bus"),
        ("Cookies", "Cookie"),
        ("Series", "Series"),
    ] {
        assert!(singular_matches(plural, singular), "{plural}/{singular}");
        let name = format!("Create{singular}");
        let bytes = extended_fixture(
            &[(&name, "smithy.api#Unit", "example#PluralOutput")],
            json!({
                "example#PluralOutput":{"type":"structure","members":{(singular):{"target":"smithy.api#String"}}}
            }),
        )?;
        let service = Catalog::borrow(&bytes)?
            .service("example")?
            .ok_or("service")?;
        let identifiers = [Identifier {
            name: plural.into(),
            kind: format!("AWS::Example::{singular}"),
            scope: PlanScope::Region,
            root_list: true,
        }];
        let Some(DraftOperation::Ready(selected)) =
            candidate(service, operation(service, &name)?, &identifiers)?
        else {
            return Err(format!("missing {name}").into());
        };
        assert_eq!(selected.leaf.name, singular);
    }
    assert!(!singular_matches("Addresses", "Addresse"));
    assert!(!singular_matches("Status", "Statu"));
    Ok(())
}

#[test]
fn given_root_list_name_without_matching_create_noun_when_drafted_then_step_is_commented()
-> TestResult {
    let text = draft(json!({
        "example#CreateMismatch":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#AddressOutput"}},
        "example#AddressOutput":{"type":"structure","members":{"Address":{"target":"smithy.api#String"}}}
    }))?;
    assert!(
        super::disabled_block(
            &text,
            "root-list identifier singular form does not match a create input or output"
        )
        .is_some_and(|block| block.contains("# operation = \"CreateMismatch\"")),
        "CreateMismatch should be disabled with its own reason"
    );
    let loaded = super::super::super::load(&text)?;
    assert!(
        !loaded
            .steps()
            .iter()
            .any(|step| step.operation == "CreateMismatch"),
        "disabled step should not be loaded"
    );
    Ok(())
}

#[test]
fn given_non_simple_pattern_when_drafted_then_step_is_commented_with_reason() -> TestResult {
    for pattern in ["^[a-z]{4}-[0-9]{2}$", "^arn:aws:.*$"] {
        let text = draft(json!({
            "example#PutThing":{"type":"operation","input":{"target":"example#PatternInput"},"output":{"target":"example#Output"}},
            "example#PatternInput":{"type":"structure","members":{"Name":{"target":"example#Name","traits":{"smithy.api#required":{}}}}},
            "example#Name":{"type":"string","traits":{"smithy.api#pattern":pattern}}
        }))?;
        assert!(
            super::disabled_block(&text, "draft pattern is not a simple character class")
                .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
            "PutThing should be disabled with its own pattern reason"
        );
        let loaded = super::super::super::load(&text)?;
        assert!(
            !loaded
                .steps()
                .iter()
                .any(|step| step.operation == "PutThing"),
            "disabled step should not be loaded"
        );
    }
    Ok(())
}

#[test]
fn given_integer_maximum_below_one_when_drafted_then_value_respects_both_bounds() -> TestResult {
    for (shape, member, expected) in [
        (json!({"smithy.api#range":{"max":0}}), json!({}), 0),
        (json!({"smithy.api#range":{"max":-5}}), json!({}), -5),
        (json!({}), json!({"smithy.api#range":{"max":0}}), 0),
        (
            json!({"smithy.api#range":{"min":-7,"max":-3}}),
            json!({"smithy.api#range":{"min":-5,"max":-4}}),
            -5,
        ),
    ] {
        assert_eq!(
            input(&integer_draft(shape, member)?, "PutThing")?.get("Count"),
            Some(&Value::Integer(expected))
        );
    }
    Ok(())
}

#[test]
fn given_unrepresentable_integer_bound_when_drafted_then_only_that_step_is_commented() -> TestResult
{
    for range in [
        json!({"min":9223372036854775808_u64}),
        json!({"min":1.5}),
        json!({"max":9223372036854775808_u64}),
    ] {
        let text = integer_draft(json!({"smithy.api#range":range}), json!({}))?;
        let loaded = super::super::super::load(&text)?;
        assert!(
            super::disabled_block(&text, "draft integer range is not representable as i64")
                .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
            "PutThing should be disabled with its own range reason"
        );
        assert!(
            !loaded
                .steps()
                .iter()
                .any(|step| step.operation == "PutThing"),
            "disabled step should not be loaded"
        );
        assert!(
            loaded
                .steps()
                .iter()
                .any(|step| step.operation == "CreateThing"),
            "CreateThing should be enabled and loaded"
        );
    }
    Ok(())
}

#[test]
fn given_disjoint_integer_bounds_when_drafted_then_step_is_commented() -> TestResult {
    let text = integer_draft(
        json!({"smithy.api#range":{"min":5}}),
        json!({"smithy.api#range":{"max":0}}),
    )?;
    let loaded = super::super::super::load(&text)?;
    assert!(
        super::disabled_block(&text, "draft integer range cannot be satisfied")
            .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
        "PutThing should be disabled with its own range reason"
    );
    assert!(
        !loaded
            .steps()
            .iter()
            .any(|step| step.operation == "PutThing"),
        "disabled step should not be loaded"
    );
    Ok(())
}

fn producer_draft(two_steps: bool) -> TestResult<String> {
    let mut extra = json!({
        "example#CreateThingA":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ProducerOutput"}},
        "example#ProducerOutput":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"First":{"target":"example#NestedOutput"},"Second":{"target":"example#NestedOutput"}}},
        "example#NestedOutput":{"type":"structure","members":{"SubscriptionArn":{"target":"smithy.api#String"}}},
        "example#PutThing":{"type":"operation","input":{"target":"example#ConsumerInput"},"output":{"target":"example#Output"}},
        "example#ConsumerInput":{"type":"structure","members":{"SubscriptionArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
    });
    if two_steps {
        extra.as_object_mut().ok_or("extra")?.insert("example#CreateThingB".into(), json!({"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ProducerOutput"}}));
    }
    draft(extra)
}

#[test]
fn given_one_step_with_duplicate_identifier_leaves_when_drafted_then_first_path_is_referenced()
-> TestResult {
    let text = producer_draft(false)?;
    assert_eq!(
        input(&text, "PutThing")?.get("SubscriptionArn"),
        Some(&Value::Table(Table::from_iter([
            ("from".into(), Value::String("createthinga".into())),
            ("path".into(), Value::String("First.SubscriptionArn".into())),
        ])))
    );
    Ok(())
}

#[test]
fn given_two_steps_with_duplicate_identifier_leaves_when_drafted_then_producer_count_is_two()
-> TestResult {
    let text = producer_draft(true)?;
    assert!(
        super::disabled_block(
            &text,
            "identifier SubscriptionArn needs exactly one earlier producer; found 2"
        )
        .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
        "PutThing should be disabled with its own producer-count reason"
    );
    assert_eq!(
        input(&text, "PutThing")
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "enabled input"
    );
    Ok(())
}

fn nested_identifier(depth: usize) -> TestResult<serde_json::Value> {
    let mut extra = json!({
        "example#CreateThingA":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ProducerOutput"}},
        "example#ProducerOutput":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"SubscriptionArns":{"target":"smithy.api#String"}}},
        "example#PutThing":{"type":"operation","input":{"target":"example#Level0"},"output":{"target":"example#Output"}},
        "example#ArnList":{"type":"list","member":{"target":"smithy.api#String"}}
    });
    for index in 0..depth {
        let next = index.saturating_add(1);
        let (name, target) = if next == depth {
            ("SubscriptionArns", "example#ArnList".into())
        } else {
            ("Nested", format!("example#Level{next}"))
        };
        extra.as_object_mut().ok_or("extra")?.insert(format!("example#Level{index}"), json!({"type":"structure","members":{(name):{"target":target,"traits":{"smithy.api#required":{}}}}}));
    }
    Ok(extra)
}

#[test]
fn given_identifier_list_at_depth_four_when_drafted_then_reference_is_synthesized() -> TestResult {
    let text = draft(nested_identifier(4)?)?;
    let actual = input(&text, "PutThing")?;
    let nested = actual
        .get("Nested")
        .and_then(|value| value.get("Nested"))
        .and_then(|value| value.get("Nested"))
        .ok_or("nested input")?;
    assert_eq!(
        nested.get("SubscriptionArns"),
        Some(&Value::Array(vec![Value::Table(Table::from_iter([
            ("from".into(), Value::String("createthinga".into())),
            ("path".into(), Value::String("SubscriptionArns".into())),
        ]))]))
    );
    Ok(())
}

#[test]
fn given_identifier_list_at_depth_five_when_drafted_then_step_is_commented() -> TestResult {
    let text = draft(nested_identifier(5)?)?;
    assert!(
        super::disabled_block(&text, "draft required input exceeds depth 4")
            .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
        "PutThing should be disabled with its own depth reason"
    );
    assert_eq!(
        input(&text, "PutThing")
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "enabled input"
    );
    Ok(())
}

#[test]
fn given_differently_named_arn_producer_when_drafted_then_suffix_does_not_resolve_reference()
-> TestResult {
    let text = draft(json!({
        "example#CreateThingA":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ProducerOutput"}},
        "example#ProducerOutput":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"TopicArn":{"target":"smithy.api#String"}}},
        "example#PutThing":{"type":"operation","input":{"target":"example#ConsumerInput"},"output":{"target":"example#Output"}},
        "example#ConsumerInput":{"type":"structure","members":{"SubscriptionArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
    }))?;
    assert!(
        super::disabled_block(
            &text,
            "identifier SubscriptionArn needs exactly one earlier producer; found 0"
        )
        .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
        "PutThing should be disabled with its own producer-count reason"
    );
    assert_eq!(
        input(&text, "PutThing")
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "enabled input"
    );
    Ok(())
}

fn remove_constraints(value: &mut inventory_core::record::Value) -> TestResult {
    use inventory_core::{
        catalog::CatalogKey,
        record::{Key, Value},
    };
    match value {
        Value::Map(fields) => {
            fields.remove(&Key::Field(CatalogKey::ValueConstraints.field()?));
            for child in fields.values_mut() {
                remove_constraints(child)?;
            }
        }
        Value::List(items) => {
            for child in items {
                remove_constraints(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn catalog_without_constraints() -> TestResult<Vec<u8>> {
    use inventory_core::{
        catalog::CatalogKey,
        record::{Key, ListRef, Record, Value, ValueRef, encode_list},
    };
    let original = Record::from_bytes(fixture(MODEL)?)?;
    let Value::Map(mut root) = ValueRef::Map(original.root()).to_value()? else {
        return Err("root map".into());
    };
    let services_key = Key::Field(CatalogKey::Services.field()?);
    let Some(Value::List(services)) = root.get_mut(&services_key) else {
        return Err("services".into());
    };
    let Some(Value::Bytes(bytes)) = services.first_mut() else {
        return Err("service bytes".into());
    };
    let record = Record::from_bytes(bytes.clone())?;
    let mut service = ValueRef::Map(record.root()).to_value()?;
    remove_constraints(&mut service)?;
    let Value::Map(fields) = &mut service else {
        return Err("service map".into());
    };
    let Some(Value::List(shapes)) = fields.get(&Key::Field(CatalogKey::Shapes.field()?)) else {
        return Err("shapes".into());
    };
    let encoded = encode_list(shapes)?;
    let offsets = ListRef::from_bytes(&encoded)?.encoded_item_offsets()?;
    fields.insert(
        Key::Field(CatalogKey::ShapeOffsets.field()?),
        Value::Bytes(
            offsets
                .iter()
                .flat_map(|offset| offset.to_be_bytes())
                .collect(),
        ),
    );
    *bytes = Record::encode(&service)?.into_bytes();
    Ok(Record::encode(&Value::Map(root))?.into_bytes())
}

#[test]
fn given_catalog_error_during_synthesis_when_appended_then_error_is_not_a_disabled_comment()
-> TestResult {
    let bytes = catalog_without_constraints()?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let operation = operation(service, "CreateThing")?;
    let candidate = Candidate {
        operation,
        identifier: identifier(service, "ThingId")?,
        leaf: leaves(service, operation.output()?, false)?
            .into_iter()
            .find(|leaf| leaf.name == "ThingId")
            .ok_or("output leaf")?,
        source: Source::Response,
        ambiguous: false,
    };
    let mut text = String::new();
    let mut earlier = Vec::new();
    let error = append(&mut text, &mut earlier, service, &candidate, &[])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "invalid catalog field ValueConstraints");
    assert!(matches!(
        error,
        SeedError::Catalog(inventory_core::catalog::CatalogError::Field(
            inventory_core::catalog::CatalogKey::ValueConstraints
        ))
    ));
    assert_eq!(text, "");
    assert_eq!(earlier.len(), 0);
    Ok(())
}
