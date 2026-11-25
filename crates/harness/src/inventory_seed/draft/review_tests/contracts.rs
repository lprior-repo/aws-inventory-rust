use super::*;

fn sample(extra: serde_json::Value, earlier: &[Produced]) -> TestResult<Value> {
    let bytes = extended_fixture(
        &[("PutSample", "example#SampleInput", "smithy.api#Unit")],
        extra,
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    Ok(values::Synthesis {
        service,
        step: "sample",
        identifiers: &[],
        earlier,
        own_input: None,
    }
    .input(operation(service, "PutSample")?.input()?)?)
}

fn reference(step: &str) -> Value {
    Value::Table(Table::from_iter([
        ("from".into(), Value::String(step.into())),
        ("path".into(), Value::String("Data.TopicArn".into())),
    ]))
}

#[test]
fn given_unregistered_arn_when_synthesized_then_exactly_one_producer_is_required() -> TestResult {
    let extra = json!({"example#SampleInput":{"type":"structure","members":{
        "TopicArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}
    }}});
    for count in 0..=2 {
        let earlier = (0..count)
            .map(|index| Produced {
                name: "TopicArn".into(),
                path: "Data.TopicArn".into(),
                step: format!("producer{index}"),
            })
            .collect::<Vec<_>>();
        let actual = sample(extra.clone(), &earlier);
        if count == 1 {
            assert_eq!(actual?.get("TopicArn"), Some(&reference("producer0")));
        } else {
            assert_eq!(
                actual.err().ok_or("expected an error")?.to_string(),
                format!(
                    "emulator seed error: identifier TopicArn needs exactly one earlier producer; found {count}"
                )
            );
        }
    }
    Ok(())
}

#[test]
fn given_integer_range_minima_when_synthesized_then_shape_and_member_bounds_are_retained()
-> TestResult {
    for (shape_traits, member_traits, expected) in [
        (json!({"smithy.api#range":{"min":10}}), json!({}), 10),
        (json!({}), json!({}), 1),
        (json!({"smithy.api#range":{"min":-7}}), json!({}), -7),
        (
            json!({"smithy.api#range":{"min":-7}}),
            json!({"smithy.api#range":{"min":5}}),
            5,
        ),
    ] {
        let mut traits = member_traits.as_object().ok_or("member traits")?.clone();
        traits.insert("smithy.api#required".into(), json!({}));
        let actual = sample(
            json!({
                "example#SampleInput":{"type":"structure","members":{"Count":{"target":"example#Count","traits":traits}}},
                "example#Count":{"type":"integer","traits":shape_traits}
            }),
            &[],
        )?;
        assert_eq!(actual.get("Count"), Some(&Value::Integer(expected)));
    }
    Ok(())
}

#[test]
fn given_integer_minimum_above_i64_when_synthesized_then_step_is_refused_without_truncation()
-> TestResult {
    let error = sample(
        json!({"example#SampleInput":{"type":"structure","members":{"Count":{"target":"example#Count","traits":{"smithy.api#required":{}}}}},
            "example#Count":{"type":"long","traits":{"smithy.api#range":{"min":9223372036854775808_u64}}}}),
        &[],
    ).err().ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft integer range is not representable as i64"
    );
    Ok(())
}

#[test]
fn given_non_simple_patterns_when_synthesized_then_an_exact_disabled_reason_is_returned()
-> TestResult {
    for pattern in ["^[a-z]{4}-[0-9]{2}$", "^arn:aws:.*$"] {
        let error = sample(
            json!({
                "example#SampleInput":{"type":"structure","members":{"Name":{"target":"example#Name","traits":{"smithy.api#required":{}}}}},
                "example#Name":{"type":"string","traits":{"smithy.api#length":{"min":8,"max":8},"smithy.api#pattern":pattern}}
            }),
            &[],
        ).err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "invalid emulator seed configuration: draft pattern is not a simple character class"
        );
    }
    Ok(())
}

#[test]
fn given_uppercase_suffixes_when_synthesized_then_non_identifiers_stay_plain_values() -> TestResult
{
    for name in ["UUID", "PAID"] {
        let earlier = [Produced {
            name: name.into(),
            path: name.into(),
            step: "producer".into(),
        }];
        let actual = sample(
            json!({"example#SampleInput":{"type":"structure","members":{
                (name):{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}
            }}}),
            &earlier,
        )?;
        assert_eq!(
            actual.get(name),
            Some(&Value::String("inv-sample-{n}".into()))
        );
    }
    Ok(())
}

#[test]
fn given_key_and_key_pair_outputs_when_selected_then_the_whole_operation_noun_wins() -> TestResult {
    let bytes = extended_fixture(
        &[("CreateKeyPair", "smithy.api#Unit", "example#PairOutput")],
        json!({"example#PairOutput":{"type":"structure","members":{
            "KeyId":{"target":"smithy.api#String"},"KeyPairId":{"target":"smithy.api#String"}
        }}}),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [
        Identifier {
            name: "KeyId".into(),
            kind: "AWS::KMS::Key".into(),
            scope: PlanScope::Region,
            root_list: false,
        },
        Identifier {
            name: "KeyPairId".into(),
            kind: "AWS::EC2::KeyPair".into(),
            scope: PlanScope::Region,
            root_list: false,
        },
    ];
    let selected =
        select(service, operation(service, "CreateKeyPair")?, &identifiers)?.ok_or("candidate")?;
    assert_eq!(selected.identifier.kind, "AWS::EC2::KeyPair");
    assert_eq!(selected.leaf.name, "KeyPairId");
    assert!(!selected.ambiguous);
    Ok(())
}

#[test]
fn given_named_root_list_identifier_when_drafted_then_exact_output_name_is_enabled() -> TestResult {
    let bytes = fixture_runtime(json!({
        "example#Strings":{"type":"list","member":{"target":"example#FooId"}},
        "example#RootLists":{"type":"structure","members":{"Addresses":{"target":"example#Strings"},"Status":{"target":"example#PrimitiveStrings"}}},
        "example#PrimitiveStrings":{"type":"list","member":{"target":"smithy.api#String"}},
        "example#FooId":{"type":"string"},
        "example#CreateFoo":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#FooOutput"}},
        "example#FooOutput":{"type":"structure","members":{"FooId":{"target":"example#FooId"}}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    assert_eq!(
        super::super::super::load(&text)?
            .steps()
            .iter()
            .map(|step| step.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["CreateFoo", "CreateThing"]
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn given_dangling_external_symlink_when_guarded_then_reason_names_unresolvable_target() -> TestResult
{
    let root = tempfile::tempdir()?;
    let output = root.path().join("draft.toml");
    std::os::unix::fs::symlink(root.path().join("missing.toml"), &output)?;
    let error = validate_output(&output).err().ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft output symlink target cannot be resolved"
    );
    Ok(())
}

#[test]
fn given_string_only_step_when_written_then_exact_output_is_emitted() -> TestResult {
    let mut text = String::new();
    write_step(
        &mut text,
        Table::from_iter([("operation".into(), Value::String("CreateFoo".into()))]),
        None,
    )?;
    assert_eq!(text, "[[step]]\noperation = \"CreateFoo\"\n");
    Ok(())
}

#[test]
fn given_identifier_list_when_synthesized_then_exactly_one_producer_is_required() -> TestResult {
    for count in [0, 1, 2] {
        let earlier = (0..count)
            .map(|index| Produced {
                name: "TopicArns".into(),
                path: "Topics.Arn".into(),
                step: format!("producer{index}"),
            })
            .collect::<Vec<_>>();
        let result = sample(
            json!({
                "example#SampleInput":{"type":"structure","members":{"TopicArns":{"target":"example#TopicArns","traits":{"smithy.api#required":{}}}}},
                "example#TopicArns":{"type":"list","member":{"target":"smithy.api#String"}}
            }),
            &earlier,
        );
        if count == 1 {
            assert_eq!(
                result?.get("TopicArns"),
                Some(&Value::Array(vec![Value::Table(Table::from_iter([
                    ("from".into(), Value::String("producer0".into())),
                    ("path".into(), Value::String("Topics.Arn".into())),
                ]))]))
            );
        } else {
            assert_eq!(
                result.err().ok_or("expected an error")?.to_string(),
                format!(
                    "emulator seed error: identifier TopicArns needs exactly one earlier producer; found {count}"
                )
            );
        }
    }
    Ok(())
}

#[test]
fn given_integer_list_element_bounds_when_synthesized_then_the_tighter_minimum_is_used()
-> TestResult {
    let actual = sample(
        json!({
            "example#SampleInput":{"type":"structure","members":{"Counts":{"target":"example#Counts","traits":{"smithy.api#required":{}}}}},
            "example#Counts":{"type":"list","member":{"target":"example#Count","traits":{"smithy.api#range":{"min":5}}}},
            "example#Count":{"type":"integer","traits":{"smithy.api#range":{"min":2}}}
        }),
        &[],
    )?;
    assert_eq!(
        actual.get("Counts"),
        Some(&Value::Array(vec![Value::Integer(5)]))
    );
    Ok(())
}

#[test]
fn given_unregistered_arn_producers_when_drafted_then_only_the_unique_reference_is_enabled()
-> TestResult {
    for count in [0, 1, 2] {
        let mut extra = json!({
            "example#PutThing":{"type":"operation","input":{"target":"example#SubscriptionInput"},"output":{"target":"example#Output"}},
            "example#SubscriptionInput":{"type":"structure","members":{"SubscriptionArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
            "example#ProducerOutput":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"SubscriptionArn":{"target":"smithy.api#String"}}}
        });
        let additions = extra.as_object_mut().ok_or("additions")?;
        for name in ["CreateThingA", "CreateThingB"].into_iter().take(count) {
            additions.insert(format!("example#{name}"), json!({
                "type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ProducerOutput"}
            }));
        }
        let bytes = fixture_runtime(extra)?;
        let text = generate(&RuntimeCatalog::from_bytes(&bytes)?, "example")?;
        if count == 1 {
            let document = text.parse::<Table>()?;
            let step = document
                .get("step")
                .and_then(Value::as_array)
                .ok_or("steps")?
                .iter()
                .find(|step| step.get("operation").and_then(Value::as_str) == Some("PutThing"))
                .ok_or("subscription step")?;
            assert_eq!(
                step.get("input")
                    .and_then(|input| input.get("SubscriptionArn")),
                Some(&Value::Table(Table::from_iter([
                    ("from".into(), Value::String("createthinga".into())),
                    ("path".into(), Value::String("SubscriptionArn".into())),
                ])))
            );
        } else {
            assert!(
                super::disabled_block(
                    &text,
                    &format!("identifier SubscriptionArn needs exactly one earlier producer; found {count}")
                )
                .is_some_and(|block| block.contains("# operation = \"PutThing\"")),
                "PutThing should be disabled with its own producer-count reason"
            );
            let loaded = super::super::super::load(&text)?;
            assert!(
                !loaded
                    .steps()
                    .iter()
                    .any(|step| step.operation == "PutThing"),
                "disabled PutThing should not be loaded"
            );
        }
    }
    Ok(())
}
