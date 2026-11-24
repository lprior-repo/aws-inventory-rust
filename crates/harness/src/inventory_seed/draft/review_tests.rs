use super::tests::{
    MODEL, disabled_block, expanded_strings, extended_fixture, fixture, identifier, operation,
};
use super::*;
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog};
use inventory_core::catalog::{Catalog, ValueConstraints};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn runtime_model(extra: serde_json::Value) -> TestResult<String> {
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("shapes")?;
    let mut additions = json!({
        "example#ListThings":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#Output"}},
        "example#ListAddresses":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#RootLists"}},
        "example#ListStatus":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#RootLists"}},
        "example#ListAttributes":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#MapOutput"}},
        "example#DescribeEndpoints":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#Output"}},
        "example#RootLists":{"type":"structure","members":{"Addresses":{"target":"example#Strings"},"Status":{"target":"example#Strings"}}},
        "example#Strings":{"type":"list","member":{"target":"smithy.api#String"}},
        "example#MapOutput":{"type":"structure","members":{"Attributes":{"target":"example#Attributes"}}},
        "example#Attributes":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}
    });
    additions
        .as_object_mut()
        .ok_or("additions")?
        .extend(extra.as_object().ok_or("extra")?.clone());
    let additions = additions.as_object().ok_or("additions")?;
    for (name, shape) in additions {
        shapes.insert(name.clone(), shape.clone());
    }
    let mut targets: Vec<_> = additions
        .iter()
        .filter(|(_, shape)| {
            shape.get("type").and_then(serde_json::Value::as_str) == Some("operation")
        })
        .map(|(name, _)| json!({"target": name}))
        .collect();
    targets.extend(
        shapes
            .iter()
            .filter(|(name, shape)| {
                !additions.contains_key(*name)
                    && shape.get("type").and_then(serde_json::Value::as_str) == Some("operation")
            })
            .map(|(name, _)| json!({"target": name})),
    );
    let service = shapes
        .get_mut("example#Svc")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("service")?;
    service.insert("operations".into(), serde_json::Value::Array(targets));
    service
        .get_mut("traits")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("traits")?
        .insert("aws.auth#sigv4".into(), json!({"name":"example"}));
    Ok(serde_json::to_string(&model)?)
}

fn enumeration(
    items: &str,
    identity: overlay::Identity,
    scope: overlay::Scope,
) -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(scope);
    plan.items = Some(overlay::ItemSelector::parse(items)?);
    plan.paginator = Some(overlay::Paginator::None);
    plan.identity = Some(identity);
    Ok(plan)
}

fn fixture_policies() -> TestResult<Vec<overlay::ServiceOverlay>> {
    let mut discovery = overlay::OperationOverlay::default();
    discovery
        .additional_resources
        .push(overlay::ResourceEmission::new(
            "ignored_parent",
            overlay::ItemSelector::Root,
            overlay::Identity::Key(vec![overlay::MemberPath::selected("ParentId")?]),
        ));
    discovery.class = Some(overlay::OverlayClass::Enumerate);
    discovery.scope = Some(overlay::Scope::Region);
    discovery.endpoint_discovery = true;
    let key = |path: &str| -> Result<_, overlay::NameError> {
        Ok(overlay::Identity::Key(vec![overlay::MemberPath::selected(
            path,
        )?]))
    };
    Ok(vec![overlay::ServiceOverlay::new(
        "example",
        vec![
            (
                "ListThings",
                enumeration("$", key("ThingId")?, overlay::Scope::Region)?,
            ),
            (
                "ListAddresses",
                enumeration("Addresses", key("$")?, overlay::Scope::Account)?,
            ),
            (
                "ListStatus",
                enumeration("Status", key("$")?, overlay::Scope::Region)?,
            ),
            (
                "ListAttributes",
                enumeration(
                    "$",
                    overlay::Identity::MapArn {
                        path: overlay::MemberPath::selected("Attributes")?,
                        key: "TopicArn".into(),
                        resource_kind: overlay::ArnResourceKind::SnsTopic,
                        request_url: None,
                    },
                    overlay::Scope::Account,
                )?,
            ),
            ("DescribeEndpoints", discovery),
        ],
    )?])
}

fn fixture_endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"example","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://example.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("draft-fixture-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://example.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("example"))
        .map_err(|failure| format!("fixture endpoint evidence failed: {failure:?}").into())
}

fn fixture_runtime(extra: serde_json::Value) -> TestResult<Vec<u8>> {
    let model = runtime_model(extra)?;
    let root = tempfile::tempdir()?;
    let path = root.path().join("models/example/model.json");
    std::fs::create_dir_all(path.parent().ok_or("model parent")?)?;
    std::fs::write(path, &model)?;
    let policies = fixture_policies()?;
    let checked = overlay::check_policies(
        root.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let kinds = fixture_kinds()?;
    let endpoints = BTreeMap::from([("example".to_owned(), fixture_endpoint()?)]);
    Ok(runtime_catalog::augment(
        &fixture(&model)?,
        &checked,
        &kinds,
        &endpoints,
    )?)
}

fn fixture_kinds() -> TestResult<kind_map::KindMap> {
    let api_only = [
        "AWS::ApiOnly::Thing",
        "AWS::ApiOnly::Address",
        "AWS::ApiOnly::Endpoint",
        "AWS::ApiOnly::Ignored",
        "AWS::ApiOnly::State",
        "AWS::ApiOnly::Map",
    ]
    .map(|kind| kind_map::ApiOnlyMapping {
        kind,
        reason: "Seed fixture",
    });
    let operations = [
        ("ListThings", "AWS::ApiOnly::Thing"),
        ("ListAddresses", "AWS::ApiOnly::Address"),
        ("DescribeEndpoints", "AWS::ApiOnly::Endpoint"),
        ("ListStatus", "AWS::ApiOnly::State"),
        ("ListAttributes", "AWS::ApiOnly::Map"),
    ]
    .map(|(operation, kind)| kind_map::OperationMapping {
        service: "example",
        operation,
        kind,
    });
    Ok(kind_map::MappingSource {
        api_only: &api_only,
        operations: &operations,
        additional_emissions: &[kind_map::AdditionalEmissionMapping {
            service: "example",
            operation: "DescribeEndpoints",
            emitter_key: "ignored_parent",
            kind: "AWS::ApiOnly::Ignored",
        }],
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?)
}

#[test]
fn given_enumeration_root_lists_and_map_arn_when_identifiers_loaded_then_exact_names_kinds_and_scopes_survive()
-> TestResult {
    let bytes = fixture_runtime(json!({}))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let discovery = runtime
        .plans()
        .iter()
        .find(|plan| matches!(plan.purpose(), PlanPurpose::EndpointDiscovery))
        .ok_or("non-enumeration plan")?;
    assert_eq!(
        discovery
            .resource_emissions()
            .iter()
            .map(inventory_core::catalog::runtime::ResourceEmissionPlan::key)
            .collect::<Vec<_>>(),
        vec!["primary", "ignored_parent"]
    );
    let additional = discovery
        .resource_emissions()
        .iter()
        .find(|emission| emission.key() == "ignored_parent")
        .ok_or("non-enumeration resource identity")?;
    assert_eq!(additional.kind().as_str(), "AWS::ApiOnly::Ignored");
    let identifiers = identifiers(&runtime, "example")?;
    assert_eq!(
        identifiers
            .iter()
            .map(|id| (id.name.as_str(), id.kind.as_str(), id.scope))
            .collect::<Vec<_>>(),
        vec![
            ("Addresses", "AWS::ApiOnly::Address", PlanScope::Account),
            ("TopicArn", "AWS::ApiOnly::Map", PlanScope::Account),
            ("Status", "AWS::ApiOnly::State", PlanScope::Region),
            ("ThingId", "AWS::ApiOnly::Thing", PlanScope::Region),
        ]
    );
    Ok(())
}

#[test]
fn given_home_region_and_exact_root_names_when_drafted_then_only_the_scope_is_rejected()
-> TestResult {
    let bytes = fixture_runtime(json!({
        "example#PutAddresses":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#PluralAddress"}},
        "example#PutStatuses":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#StatusOutput"}},
        "example#PluralAddress":{"type":"structure","members":{"Addresses":{"target":"smithy.api#String"}}},
        "example#StatusOutput":{"type":"structure","members":{"Status":{"target":"smithy.api#String"}}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    assert_eq!(
        text.matches(
            "# Disabled: root-list identifier singular form does not match a create input or output"
        )
        .count(),
        0
    );
    let document = text.parse::<Table>()?;
    let steps = document
        .get("step")
        .and_then(Value::as_array)
        .ok_or("steps")?;
    for name in ["PutAddresses", "PutStatuses"] {
        assert_eq!(
            steps
                .iter()
                .filter_map(|step| step.get("operation").and_then(Value::as_str))
                .filter(|operation| *operation == name)
                .count(),
            1
        );
    }
    let service = runtime.catalog().service("example")?.ok_or("service")?;
    let mut identifier = identifier(service, "ThingId")?;
    identifier.scope = PlanScope::HomeRegion;
    let draft = candidate(service, operation(service, "CreateThing")?, &[identifier])?
        .ok_or("candidate")?;
    let (operation, kind, reason) = match draft {
        DraftOperation::Disabled {
            operation,
            kind,
            reason,
        } => (operation, kind, reason),
        DraftOperation::Ready(_) => return Err("expected disabled home-region draft".into()),
    };
    assert_eq!(operation.name()?, "CreateThing");
    assert_eq!(
        reason,
        "home-region scope is not representable in seed files"
    );
    assert!(
        kind.is_none(),
        "home-region disabled step should not have a kind"
    );
    Ok(())
}

#[test]
fn given_recursive_optional_output_when_drafted_then_other_operations_and_identity_remain_enabled()
-> TestResult {
    let bytes = fixture_runtime(json!({
        "example#CreateRecursive":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#Node"}},
        "example#Node":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"Entries":{"target":"example#Nodes"}}},
        "example#Nodes":{"type":"list","member":{"target":"example#Node"}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    let document: Value = toml::from_str(&text)?;
    let steps = document
        .get("step")
        .and_then(Value::as_array)
        .ok_or("steps")?;
    assert_eq!(
        steps
            .iter()
            .map(|step| step.get("operation").and_then(Value::as_str))
            .collect::<Vec<_>>(),
        vec![Some("CreateRecursive"), Some("CreateThing")]
    );
    assert_eq!(
        steps
            .first()
            .and_then(|step| step.get("identity"))
            .and_then(Value::as_str),
        Some("ThingId")
    );
    super::super::load(&text)?;
    Ok(())
}

#[test]
fn given_fan_out_beyond_identity_budget_when_drafted_then_only_that_operation_is_disabled()
-> TestResult {
    let fields = (0..4097)
        .map(|index| {
            (
                format!("Field{index}"),
                json!({"target":"smithy.api#String"}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let bytes = fixture_runtime(json!({
        "example#CreateHuge":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#Huge"}},
        "example#Huge":{"type":"structure","members":fields}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    let loaded = super::super::load(&text)?;
    assert!(
        disabled_block(&text, "draft identity traversal exceeds node budget")
            .is_some_and(|block| block.contains("# operation = \"CreateHuge\"")),
        "CreateHuge should be disabled with its own budget reason and operation"
    );
    assert!(
        !loaded
            .steps()
            .iter()
            .any(|step| step.operation == "CreateHuge"),
        "disabled step should not be loaded"
    );
    let enabled = loaded
        .steps()
        .iter()
        .map(|step| step.operation.as_str())
        .collect::<Vec<_>>();
    assert_eq!(enabled, vec!["CreateThing"]);
    Ok(())
}

#[test]
fn given_prelude_integer_and_string_identifiers_when_selected_then_only_text_matches() -> TestResult
{
    let identifiers = [Identifier {
        name: "Id".into(),
        kind: "AWS::ApiOnly::Thing".into(),
        scope: PlanScope::Region,
        root_list: false,
    }];
    for (target, expected) in [
        ("smithy.api#Integer", None),
        ("smithy.api#String", Some("Id")),
    ] {
        let bytes = extended_fixture(
            &[("PutNumber", "example#Numeric", "smithy.api#Unit")],
            json!({"example#Numeric":{"type":"structure","members":{
                "Id":{"target":target,"traits":{"smithy.api#required":{}}}
            }}}),
        )?;
        let service = Catalog::borrow(&bytes)?
            .service("example")?
            .ok_or("service")?;
        let actual = candidate(service, operation(service, "PutNumber")?, &identifiers)?
            .map(|draft| match draft {
                DraftOperation::Ready(candidate) => Ok(candidate.identifier.name),
                DraftOperation::Disabled { kind, reason, .. } => {
                    if kind.is_some() {
                        Err("has kind")
                    } else {
                        Err(reason)
                    }
                }
            })
            .transpose()?;
        assert_eq!(actual.as_deref(), expected);
    }
    Ok(())
}

#[test]
fn given_identifier_without_suffix_when_synthesized_then_the_exact_earlier_reference_is_used()
-> TestResult {
    let bytes = extended_fixture(
        &[("PutUrl", "example#UrlInput", "smithy.api#Unit")],
        json!({
            "example#UrlInput":{"type":"structure","members":{"QueueUrl":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [Identifier {
        name: "QueueUrl".into(),
        kind: "AWS::ApiOnly::Queue".into(),
        scope: PlanScope::Region,
        root_list: false,
    }];
    let earlier = [Produced {
        step: "queue".into(),
        name: "QueueUrl".into(),
        path: "Data.QueueUrl".into(),
    }];
    let synthesis = values::Synthesis {
        service,
        step: "url",
        identifiers: &identifiers,
        earlier: &earlier,
        own_input: None,
    };
    let input = synthesis.input(operation(service, "PutUrl")?.input()?)?;
    assert_eq!(
        input.get("QueueUrl"),
        Some(&Value::Table(Table::from_iter([
            ("from".into(), Value::String("queue".into())),
            ("path".into(), Value::String("Data.QueueUrl".into()))
        ])))
    );
    Ok(())
}

#[test]
fn given_string_maximum_sixty_four_when_repeated_then_the_reader_expands_every_placeholder()
-> TestResult {
    let constraints = ValueConstraints {
        max_length: Some(64),
        ..ValueConstraints::default()
    };
    let template = values::string("topic", constraints, ValueConstraints::default())?;
    assert_eq!(template, "inv-topic-{n}");
    assert_eq!(
        expanded_strings(&template)?,
        [
            "inv-topic-1",
            "inv-topic-2",
            "inv-topic-3",
            "inv-topic-4",
            "inv-topic-5",
            "inv-topic-6",
            "inv-topic-7",
            "inv-topic-8",
            "inv-topic-9",
            "inv-topic-10"
        ]
        .map(String::from)
        .to_vec()
    );
    Ok(())
}

fn wide_fixture(nested: bool) -> TestResult<Vec<u8>> {
    let members = (0..64)
        .map(|index| {
            let target = if nested {
                String::from("example#Inner")
            } else {
                format!("example#Field{index}Value")
            };
            (
                format!("Field{index}"),
                json!({"target":target,"traits":{"smithy.api#required":{}}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let inner = (0..64)
        .map(|index| {
            (
                format!("Field{index}"),
                json!({"target":"smithy.api#String","traits":{"smithy.api#required":{}}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let mut shapes = json!({
        "example#Wide":{"type":"structure","members":members},
        "example#Inner":{"type":"structure","members":inner}
    });
    if !nested {
        let shapes = shapes.as_object_mut().ok_or("shapes")?;
        for index in 0..64 {
            shapes.insert(
                format!("example#Field{index}Value"),
                json!({
                    "type":"string","traits":{"smithy.api#enum":[{"value":format!("Field{index}")}]}
                }),
            );
        }
    }
    extended_fixture(&[("PutWide", "example#Wide", "smithy.api#Unit")], shapes)
}

#[test]
fn given_sixty_four_required_members_when_synthesized_then_all_exact_values_are_retained()
-> TestResult {
    let bytes = wide_fixture(false)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "wide",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    let input = synthesis.input(operation(service, "PutWide")?.input()?)?;
    let expected = (0..64)
        .map(|index| {
            (
                format!("Field{index}"),
                Value::String(format!("Field{index}")),
            )
        })
        .collect::<Table>();
    assert_eq!(input, Value::Table(expected));
    Ok(())
}

#[test]
fn given_input_exceeding_max_nodes_when_synthesized_then_exact_budget_error_is_returned()
-> TestResult {
    let bytes = wide_fixture(true)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "wide",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    let error = synthesis
        .input(operation(service, "PutWide")?.input()?)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft input exceeds node budget"
    );
    Ok(())
}

#[test]
fn given_required_list_with_an_identifier_name_when_synthesized_then_it_stays_a_list() -> TestResult
{
    let bytes = extended_fixture(
        &[("PutTopics", "example#Topics", "smithy.api#Unit")],
        json!({
            "example#Topics":{"type":"structure","members":{"TopicArns":{"target":"example#Arns","traits":{"smithy.api#required":{}}}}},
            "example#Arns":{"type":"list","member":{"target":"smithy.api#String"}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let earlier = [Produced {
        step: "producer".into(),
        name: "TopicArns".into(),
        path: "TopicArns".into(),
    }];
    let synthesis = values::Synthesis {
        service,
        step: "topics",
        identifiers: &[],
        earlier: &earlier,
        own_input: None,
    };
    assert_eq!(
        synthesis.input(operation(service, "PutTopics")?.input()?)?,
        Value::Table(Table::from_iter([(
            "TopicArns".into(),
            Value::Array(vec![Value::Table(Table::from_iter([
                ("from".into(), Value::String("producer".into())),
                ("path".into(), Value::String("TopicArns".into())),
            ]))])
        )]))
    );
    Ok(())
}

#[test]
fn given_root_lists_when_drafted_then_non_candidate_is_omitted_and_sensitive_step_is_disabled()
-> TestResult {
    let bytes = fixture_runtime(json!({
        "example#CreateAddress":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#AddressOutput"}},
        "example#CreateStatus":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#StatusOutput"}},
        "example#CreateNothing":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"smithy.api#Unit"}},
        "example#AddressOutput":{"type":"structure","members":{"Address":{"target":"smithy.api#String"}}},
        "example#StatusOutput":{"type":"structure","members":{"Status":{"target":"smithy.api#String"}}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    assert!(!text.contains("CreateNothing"));
    let loaded = super::super::load(&text)?;
    let operations = loaded
        .steps()
        .iter()
        .map(|step| step.operation.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        vec!["CreateAddress", "CreateStatus", "CreateThing"]
    );
    assert!(
        disabled_block(&text, "sensitive member Secret")
            .is_some_and(|block| block.contains("# operation = \"CreateSecret\"")),
        "CreateSecret should be disabled with its sensitive-member reason"
    );
    Ok(())
}

#[test]
fn given_a_named_list_element_when_drafted_then_its_modeled_singular_wins() -> TestResult {
    let bytes = fixture_runtime(json!({
        "example#Strings":{"type":"list","member":{"target":"example#PostalLocation"}},
        "example#RootLists":{"type":"structure","members":{"Addresses":{"target":"example#Strings"},"Status":{"target":"example#PrimitiveStrings"}}},
        "example#PrimitiveStrings":{"type":"list","member":{"target":"smithy.api#String"}},
        "example#PostalLocation":{"type":"string"},
        "example#CreatePostalLocation":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#PostalOutput"}},
        "example#PostalOutput":{"type":"structure","members":{"PostalLocation":{"target":"example#PostalLocation"}}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    assert_eq!(
        super::super::load(&text)?
            .steps()
            .iter()
            .map(|step| step.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["CreatePostalLocation", "CreateThing"]
    );
    Ok(())
}

#[test]
fn given_plural_rules_when_matching_then_only_supported_singulars_match() -> TestResult {
    for (plural, singular, expected) in [
        ("Addresses", "Address", true),
        ("Addresses", "Addresse", false),
        ("Policies", "Policy", true),
        ("Class", "Class", true),
        ("Class", "Clas", false),
        ("Status", "Status", true),
        ("Status", "Statu", false),
        ("Topics", "Topic", true),
        ("Cookies", "Cookie", true),
        ("Movies", "Movie", true),
        ("Series", "Series", true),
        ("Aliases", "Alias", true),
        ("Statuses", "Status", true),
        ("Buses", "Bus", true),
        ("Heroes", "Hero", true),
    ] {
        assert_eq!(singular_matches(plural, singular), expected);
    }
    Ok(())
}

#[test]
fn given_root_list_identifier_with_unmatched_create_noun_when_drafted_then_step_is_disabled()
-> TestResult {
    let bytes = fixture_runtime(json!({
        "example#CreateThing":{"type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#ThingOutput"}},
        "example#ThingOutput":{"type":"structure","members":{"Address":{"target":"smithy.api#String"}}}
    }))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "example")?;
    assert!(
        disabled_block(
            &text,
            "root-list identifier singular form does not match a create input or output"
        )
        .is_some_and(|block| block.contains("# operation = \"CreateThing\"")),
        "CreateThing should be disabled with its own reason"
    );
    assert_eq!(
        super::super::load(&text)
            .err()
            .ok_or("disabled-only draft should reject when loaded")?
            .to_string(),
        "invalid emulator seed configuration: seed file requires at least one step"
    );
    Ok(())
}

fn identity_budget_fixture(chain: bool) -> TestResult<Vec<u8>> {
    let mut shapes = serde_json::Map::new();
    let output = if chain {
        "example#Chain0"
    } else {
        "example#WideOutput"
    };
    shapes.insert(
        "example#CreateBounded".into(),
        json!({
            "type":"operation","input":{"target":"smithy.api#Unit"},"output":{"target":"example#BoundedOutput"}
        }),
    );
    shapes.insert(
        "example#BoundedOutput".into(),
        json!({"type":"structure","members":{"ThingId":{"target":"smithy.api#String"},"Data":{"target":output}}}),
    );
    if chain {
        for index in 0..4096usize {
            let target = if index == 4095 {
                String::from("smithy.api#String")
            } else {
                format!("example#Chain{}", index.saturating_add(1))
            };
            shapes.insert(
                format!("example#Chain{index}"),
                json!({"type":"list","member":{"target":target}}),
            );
        }
    } else {
        for (name, target) in [
            ("example#WideOutput", "example#InnerOutput"),
            ("example#InnerOutput", "smithy.api#String"),
        ] {
            let members = (0..64)
                .map(|index| (format!("Field{index}"), json!({"target":target})))
                .collect::<serde_json::Map<_, _>>();
            shapes.insert(name.into(), json!({"type":"structure","members":members}));
        }
    }
    fixture_runtime(serde_json::Value::Object(shapes))
}

#[test]
fn given_branching_and_deep_outputs_when_drafted_then_both_identity_limits_preserve_other_steps()
-> TestResult {
    // 64 x 64 reaches the visit limit; 4096 nested lists reach pending + visited.
    for chain in [false, true] {
        let bytes = identity_budget_fixture(chain)?;
        let runtime = RuntimeCatalog::from_bytes(&bytes)?;
        let text = generate(&runtime, "example")?;
        assert!(
            disabled_block(&text, "draft identity traversal exceeds node budget")
                .is_some_and(|block| block.contains("# operation = \"CreateBounded\"")),
            "CreateBounded should be disabled with its own budget reason"
        );

        assert_eq!(
            super::super::load(&text)?
                .steps()
                .iter()
                .map(|step| step.operation.as_str())
                .collect::<Vec<_>>(),
            vec!["CreateThing"]
        );
    }
    Ok(())
}

#[test]
fn given_exact_reserved_string_space_when_drafted_then_the_placeholder_boundary_is_inclusive()
-> TestResult {
    for (maximum, expected) in [(14, "inv-topic-1"), (15, "inv-topic-{n}")] {
        let constraints = ValueConstraints {
            max_length: Some(maximum),
            ..ValueConstraints::default()
        };
        let template = values::string("topic", constraints, ValueConstraints::default())?;
        assert_eq!(template, expected);
        assert_eq!(
            expanded_strings(&template)?.last().map(String::as_str),
            Some(if maximum == 15 {
                "inv-topic-10"
            } else {
                "inv-topic-1"
            })
        );
    }
    Ok(())
}

#[test]
fn given_a_shared_sibling_shape_when_collecting_leaves_then_both_paths_are_preserved() -> TestResult
{
    let bytes = extended_fixture(
        &[("CreateShared", "smithy.api#Unit", "example#SharedOutput")],
        json!({
            "example#SharedOutput":{"type":"structure","members":{"A":{"target":"example#Inner"},"B":{"target":"example#Inner"}}},
            "example#Inner":{"type":"structure","members":{"ThingId":{"target":"smithy.api#String"}}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    assert_eq!(
        leaves(
            service,
            operation(service, "CreateShared")?.output()?,
            false
        )?
        .iter()
        .map(|leaf| (leaf.name.as_str(), leaf.path.as_str()))
        .collect::<Vec<_>>(),
        vec![("ThingId", "A.ThingId"), ("ThingId", "B.ThingId")]
    );
    Ok(())
}

fn exact_input_budget_fixture(overflow: bool) -> TestResult<Vec<u8>> {
    let members = (0..64)
        .map(|index| {
            let target = if overflow && index == 63 {
                "smithy.api#String"
            } else if (overflow && index < 62) || (!overflow && index < 31) {
                "example#Inner31"
            } else {
                "example#Inner30"
            };
            (
                format!("Field{index}"),
                json!({"target":target,"traits":{"smithy.api#required":{}}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let mut shapes = json!({"example#ExactInput":{"type":"structure","members":members}});
    for count in [30, 31] {
        let members = (0..count)
            .map(|index| {
                (
                    format!("Value{index}"),
                    json!({"target":"smithy.api#String","traits":{"smithy.api#required":{}}}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        shapes.as_object_mut().ok_or("shapes")?.insert(
            format!("example#Inner{count}"),
            json!({"type":"structure","members":members}),
        );
    }
    extended_fixture(
        &[("PutExact", "example#ExactInput", "smithy.api#Unit")],
        shapes,
    )
}

#[test]
fn given_exact_node_budget_when_synthesized_then_4096_tasks_succeed_and_4097_fail() -> TestResult {
    // Success: 2 + 3*64 + 2*(31*31 + 33*30) = 4096.
    // Failure: 2 + 2 + 3*63 + 2*(62*31 + 30) = 4097.
    for overflow in [false, true] {
        let bytes = exact_input_budget_fixture(overflow)?;
        let service = Catalog::borrow(&bytes)?
            .service("example")?
            .ok_or("service")?;
        let synthesis = values::Synthesis {
            service,
            step: "exact",
            identifiers: &[],
            earlier: &[],
            own_input: None,
        };
        let result = synthesis.input(operation(service, "PutExact")?.input()?);
        if overflow {
            assert_eq!(
                result.err().ok_or("expected an error")?.to_string(),
                "invalid emulator seed configuration: draft input exceeds node budget"
            );
        } else {
            let inner = |count| {
                Value::Table(
                    (0..count)
                        .map(|index| {
                            (
                                format!("Value{index}"),
                                Value::String("inv-exact-{n}".into()),
                            )
                        })
                        .collect(),
                )
            };
            let expected = (0..64)
                .map(|index| {
                    (
                        format!("Field{index}"),
                        inner(if index < 31 { 31 } else { 30 }),
                    )
                })
                .collect::<Table>();
            assert_eq!(result?, Value::Table(expected));
        }
    }
    Ok(())
}

mod contracts;
mod round_six;
