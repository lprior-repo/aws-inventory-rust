use super::*;
use catalog_tool::smithy::{Registry, build_catalog};
use inventory_core::catalog::{Catalog, ValueConstraints};
use toml::map::Map;
type Table = Map<String, toml::Value>;
use toml::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;
pub(super) fn disabled_block(text: &str, reason: &str) -> Option<String> {
    let marker = format!("# Disabled: {reason}");
    let mut lines = text.lines().skip_while(|line| *line != marker);
    let first = lines.next()?;
    Some(
        std::iter::once(first)
            .chain(lines.take_while(|line| {
                !line.is_empty()
                    && !line.starts_with("[[step]]")
                    && !line.starts_with("# Disabled:")
            }))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub(super) const MODEL: &str = r#"{"smithy":"2.0","shapes":{
 "example#Svc":{"type":"service","version":"2020-01-01","operations":[{"target":"example#CreateThing"},{"target":"example#CreateChild"},{"target":"example#CreateSecret"}],"traits":{"aws.api#service":{"sdkId":"Example","endpointPrefix":"example"},"aws.protocols#restJson1":{}}},
 "example#CreateThing":{"type":"operation","input":{"target":"example#Input"},"output":{"target":"example#Output"}},
 "example#CreateChild":{"type":"operation","input":{"target":"example#Child"},"output":{"target":"example#Output"}},
 "example#CreateSecret":{"type":"operation","input":{"target":"example#SecretInput"},"output":{"target":"example#Output"}},
 "example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"State":{"target":"example#State","traits":{"smithy.api#required":{}}},"Settings":{"target":"example#Settings","traits":{"smithy.api#required":{}}}}},
 "example#Settings":{"type":"structure","members":{"Enabled":{"target":"smithy.api#Boolean","traits":{"smithy.api#required":{}}},"Count":{"target":"smithy.api#Integer","traits":{"smithy.api#required":{},"smithy.api#range":{"min":5}}},"Entries":{"target":"example#Entries","traits":{"smithy.api#required":{}}}}},
 "example#Entries":{"type":"list","member":{"target":"smithy.api#String"}},
 "example#State":{"type":"string","traits":{"smithy.api#enum":[{"value":"zulu"},{"value":"alpha"}]}},
 "example#Child":{"type":"structure","members":{"ParentId":{"target":"example#Id","traits":{"smithy.api#required":{}}}}},
 "example#SecretInput":{"type":"structure","members":{"Secret":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#sensitive":{}}}}},
 "example#Output":{"type":"structure","members":{"ThingId":{"target":"example#Id"},"ParentId":{"target":"example#Id"}}},
 "example#Id":{"type":"string"}
}}"#;

pub(super) fn fixture(model_text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let model = root.path().join("models/example/model.json");
    std::fs::create_dir_all(model.parent().ok_or("model has no parent")?)?;
    std::fs::write(model, model_text)?;
    let value: serde_json::Value = serde_json::from_str(model_text)?;
    let names = value
        .get("shapes")
        .and_then(serde_json::Value::as_object)
        .ok_or("shapes")?
        .values()
        .filter_map(|shape| shape.get("members").and_then(serde_json::Value::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let registry = Registry::parse(&format!("{names}\n"))?;
    Ok(build_catalog(root.path(), &registry, "seed-draft")?)
}

pub(super) fn operation<'a>(service: Service<'a>, name: &str) -> Result<Operation<'a>, SeedError> {
    service
        .operations()?
        .try_fold(None, |found, operation| {
            let operation = operation?;
            Ok::<_, SeedError>(if operation.name()? == name {
                Some(operation)
            } else {
                found
            })
        })?
        .ok_or(SeedError::Invalid("fixture operation is missing"))
}

pub(super) fn identifier(service: Service<'_>, name: &str) -> Result<Identifier, SeedError> {
    let output = operation(service, "CreateThing")?.output()?;
    let leaf = leaves(service, output, false)?
        .into_iter()
        .find(|leaf| leaf.name == name)
        .ok_or(SeedError::Invalid("fixture identity is missing"))?;
    Ok(Identifier {
        kind: String::from("AWS::Example::Thing"),
        name: leaf.name,
        scope: PlanScope::Region,
        root_list: false,
    })
}

fn candidate<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    identifiers: &[Identifier],
) -> Result<Option<Candidate<'a>>, SeedError> {
    match super::candidate(service, operation, identifiers)? {
        Some(DraftOperation::Ready(candidate)) => Ok(Some(candidate)),
        Some(DraftOperation::Disabled { reason, .. }) => Err(SeedError::Invalid(reason)),
        None => Ok(None),
    }
}

#[test]
fn given_length_bounds_when_strings_drafted_then_expanded_values_fit_exact_bounds() -> TestResult {
    let target = ValueConstraints {
        min_length: Some(8),
        max_length: Some(8),
        pattern: None,
        ..ValueConstraints::default()
    };
    assert_eq!(
        values::string("topic", target, ValueConstraints::default())?,
        "inv-top1"
    );
    let member = ValueConstraints {
        min_length: Some(15),
        max_length: Some(15),
        pattern: None,
        ..ValueConstraints::default()
    };
    assert_eq!(
        values::string("topic", ValueConstraints::default(), member)?,
        "inv-topic-1aaa1"
    );
    let pattern = ValueConstraints {
        min_length: None,
        max_length: None,
        pattern: Some("^[A-Z]{4}$"),
        ..ValueConstraints::default()
    };
    assert_eq!(
        values::string("topic", pattern, ValueConstraints::default())?,
        "AAAA"
    );
    for (constraints, expected) in [(target, "inv-top1"), (member, "inv-topic-1aaa1")] {
        let template = values::string("topic", constraints, ValueConstraints::default())?;
        assert_eq!(expanded_strings(&template)?, vec![expected.to_owned(); 10]);
    }
    let error = values::string("topic", target, member)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft string constraints cannot be satisfied"
    );
    Ok(())
}

#[test]
fn given_enum_and_nested_required_members_when_drafted_then_exact_values_are_synthesized()
-> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "thing",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    let input = synthesis.input(operation(service, "CreateThing")?.input()?)?;
    assert_eq!(input.get("State").and_then(Value::as_str), Some("zulu"));
    assert_eq!(
        input.get("Name").and_then(Value::as_str),
        Some("inv-thing-{n}")
    );
    let settings = input.get("Settings").ok_or("settings")?;
    assert_eq!(settings.get("Enabled"), Some(&Value::Boolean(false)));
    assert_eq!(settings.get("Count"), Some(&Value::Integer(5)));
    assert_eq!(
        settings.get("Entries"),
        Some(&Value::Array(vec![Value::String("inv-thing-{n}".into())]))
    );
    Ok(())
}

#[test]
fn given_missing_identifier_producer_when_drafted_then_whole_step_is_commented_with_reason()
-> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = vec![
        identifier(service, "ThingId")?,
        identifier(service, "ParentId")?,
    ];
    let child =
        candidate(service, operation(service, "CreateChild")?, &identifiers)?.ok_or("candidate")?;
    let mut text = String::new();
    append(&mut text, &mut Vec::new(), service, &child, &identifiers)?;
    assert_eq!(
        disabled_block(
            &text,
            "identifier ParentId needs exactly one earlier producer; found 0"
        )
        .as_deref(),
        Some(
            "# Disabled: identifier ParentId needs exactly one earlier producer; found 0\n# [[step]]\n# id = \"createchild\"\n# operation = \"CreateChild\"\n# kind = \"AWS::Example::Thing\"\n# identity = \"ParentId\"\n# scope = \"region\""
        )
    );
    Ok(())
}

#[test]
fn given_unique_earlier_producer_when_drafted_then_reference_and_seed_reader_agree() -> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = vec![
        identifier(service, "ThingId")?,
        identifier(service, "ParentId")?,
    ];
    let parent =
        candidate(service, operation(service, "CreateThing")?, &identifiers)?.ok_or("parent")?;
    let child =
        candidate(service, operation(service, "CreateChild")?, &identifiers)?.ok_or("child")?;
    let mut text = String::from("format = 1\nservice = 'example'\n");
    let mut earlier = Vec::new();
    append(&mut text, &mut earlier, service, &parent, &identifiers)?;
    append(&mut text, &mut earlier, service, &child, &identifiers)?;
    let document: Value = toml::from_str(&text)?;
    let child = document
        .get("step")
        .and_then(Value::as_array)
        .and_then(|steps| steps.get(1))
        .ok_or("child step")?;
    let reference = child
        .get("input")
        .and_then(|input| input.get("ParentId"))
        .ok_or("reference")?;
    assert_eq!(
        reference.get("from").and_then(Value::as_str),
        Some("creatething")
    );
    assert_eq!(
        reference.get("path").and_then(Value::as_str),
        Some("ParentId")
    );
    let scenario = super::super::load(&text)?;
    assert_eq!(
        scenario
            .steps()
            .iter()
            .map(|step| step.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["CreateThing", "CreateChild"]
    );
    Ok(())
}

#[test]
fn given_sensitive_required_member_when_drafted_then_no_value_is_written() -> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = vec![identifier(service, "ThingId")?];
    let secret = candidate(service, operation(service, "CreateSecret")?, &identifiers)?
        .ok_or("candidate")?;
    let mut text = String::from("format = 1\nservice = 'example'\n");
    append(&mut text, &mut Vec::new(), service, &secret, &identifiers)?;
    assert_eq!(
        disabled_block(&text, "sensitive member Secret").as_deref(),
        Some(
            "# Disabled: sensitive member Secret\n# [[step]]\n# id = \"createsecret\"\n# operation = \"CreateSecret\"\n# kind = \"AWS::Example::Thing\"\n# identity = \"ThingId\"\n# scope = \"region\""
        )
    );
    assert_eq!(
        super::super::load(&text)
            .err()
            .ok_or("disabled-only draft should reject when loaded")?
            .to_string(),
        "invalid emulator seed configuration: seed file requires at least one step"
    );
    assert!(!text.contains("inv-createsecret"));
    Ok(())
}

fn catalog_bytes() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"))
}

#[test]
fn given_sns_catalog_when_public_draft_generated_then_topic_is_reader_and_model_accepted()
-> TestResult {
    let bytes = catalog_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, "sns")?;
    let scenario = super::super::load(&text)?;
    assert!(scenario.steps().iter().all(|step| {
        ["Create", "Put", "Register"]
            .iter()
            .any(|prefix| step.operation.starts_with(prefix))
    }));
    let topic = scenario
        .steps()
        .iter()
        .find(|step| step.operation == "CreateTopic")
        .ok_or("topic draft")?;
    assert_eq!(topic.kind, "AWS::SNS::Topic");
    super::super::model::SeedModels::load(&runtime, &scenario)?;
    assert!(!text.contains("the model's range was not available"));
    Ok(())
}

#[test]
fn given_ec2_outside_thin_slice_when_public_draft_written_then_vpc_recipe_is_readable() -> TestResult
{
    let root = tempfile::tempdir()?;
    let output = root.path().join("ec2.toml");
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog");
    run("ec2", &catalog, Some(&output))?;
    let text = std::fs::read_to_string(output)?;
    let scenario = super::super::load(&text)?;
    let vpc = scenario
        .steps()
        .iter()
        .find(|step| step.operation == "CreateVpc")
        .ok_or("VPC draft")?;
    assert_eq!(vpc.kind, "AWS::EC2::VPC");

    Ok(())
}
#[test]
fn given_output_under_checked_in_seeds_when_run_then_refused_before_catalog_read() -> TestResult {
    let seeds = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    let output = seeds.join(format!("draft-run-guard-{}.toml", std::process::id()));
    let missing_catalog = seeds.join("missing-catalog-for-draft-run-guard.catalog");
    assert!(!output.exists(), "guard test output should start absent");
    let error = run("ec2", &missing_catalog, Some(&output))
        .err()
        .ok_or("expected seeds output refusal")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft output must be outside crates/harness/seeds"
    );
    assert!(!output.exists(), "refused output must not be created");
    Ok(())
}

#[test]
fn given_checked_in_seed_output_when_validated_then_refused() -> TestResult {
    let error =
        validate_output(&Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds/draft-test.toml"))
            .err()
            .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft output must be outside crates/harness/seeds"
    );
    Ok(())
}

#[test]
fn given_two_identifier_producers_when_drafted_then_ambiguity_is_an_exact_error() -> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let earlier = ["first", "second"].map(|step| Produced {
        step: step.into(),
        name: String::from("ParentId"),
        path: String::from("ParentId"),
    });
    let synthesis = values::Synthesis {
        service,
        step: "child",
        identifiers: &[],
        earlier: &earlier,
        own_input: None,
    };
    let error = synthesis
        .input(operation(service, "CreateChild")?.input()?)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "emulator seed error: identifier ParentId needs exactly one earlier producer; found 2"
    );
    Ok(())
}

#[test]
fn given_simple_escaped_character_class_when_drafted_then_only_legal_characters_are_used()
-> TestResult {
    let constraints = ValueConstraints {
        min_length: Some(4),
        max_length: Some(4),
        pattern: Some(r"^[\d]{4}$"),
        ..ValueConstraints::default()
    };
    assert_eq!(
        values::string("topic", constraints, ValueConstraints::default())?,
        "0001"
    );
    let template = values::string("topic", constraints, ValueConstraints::default())?;
    assert_eq!(expanded_strings(&template)?, vec![String::from("0001"); 10]);
    let constraints = ValueConstraints {
        pattern: Some("^[a-z]{4}$"),
        ..constraints
    };
    assert_eq!(
        values::string("topic", constraints, ValueConstraints::default())?,
        "inva"
    );
    let constraints = ValueConstraints {
        pattern: Some(r"^[\w-]+$"),
        ..constraints
    };
    assert_eq!(
        values::string("topic", constraints, ValueConstraints::default())?,
        "inv1"
    );
    Ok(())
}

pub(super) fn expanded_strings(template: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let step = Table::from_iter([
        (String::from("id"), Value::String("sample".into())),
        (
            String::from("operation"),
            Value::String("CreateThing".into()),
        ),
        (
            String::from("kind"),
            Value::String("AWS::Example::Thing".into()),
        ),
        (String::from("identity"), Value::String("ThingId".into())),
        (String::from("repeat"), Value::Integer(10)),
        (
            String::from("input"),
            Value::Table(Table::from_iter([(
                String::from("Name"),
                Value::String(template.into()),
            )])),
        ),
    ]);
    let document = Table::from_iter([
        (String::from("format"), Value::Integer(1)),
        (String::from("service"), Value::String("example".into())),
        (String::from("step"), Value::Array(vec![Value::Table(step)])),
    ]);
    let scenario = super::super::load(&toml::to_string(&document)?)?;
    scenario
        .steps()
        .iter()
        .map(|step| match &step.inputs.first().ok_or("input")?.value {
            super::super::SeedInputValue::Text(text) => Ok(text.clone()),
            _ => Err("expected text".into()),
        })
        .collect()
}

pub(super) fn extended_fixture(
    operations: &[(&str, &str, &str)],
    extra_shapes: serde_json::Value,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("shapes")?;
    for (name, shape) in extra_shapes.as_object().ok_or("extra shapes")? {
        shapes.insert(name.clone(), shape.clone());
    }
    for (name, input, output) in operations {
        shapes.insert(
            format!("example#{name}"),
            serde_json::json!({
                "type": "operation", "input": {"target": input}, "output": {"target": output}
            }),
        );
        shapes
            .get_mut("example#Svc")
            .and_then(|service| service.get_mut("operations"))
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("operations")?
            .push(serde_json::json!({"target": format!("example#{name}")}));
    }
    fixture(&serde_json::to_string(&model)?)
}

#[test]
fn given_non_create_and_multiple_kinds_when_selected_then_only_preferred_kind_is_enabled()
-> TestResult {
    let bytes = extended_fixture(
        &[("DescribeThing", "example#Input", "example#Output")],
        serde_json::json!({}),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let mut other = identifier(service, "ParentId")?;
    other.kind = String::from("AWS::Example::Widget");
    let identifiers = [identifier(service, "ThingId")?, other];
    assert_eq!(
        candidate(service, operation(service, "DescribeThing")?, &identifiers)?
            .map(|c| c.identifier.kind),
        None
    );
    let preferred =
        candidate(service, operation(service, "CreateThing")?, &identifiers)?.ok_or("preferred")?;
    assert_eq!(preferred.identifier.kind, "AWS::Example::Thing");
    assert!(!preferred.ambiguous);
    let ambiguous =
        candidate(service, operation(service, "CreateChild")?, &identifiers)?.ok_or("ambiguous")?;
    let mut text = String::from("format = 1\nservice = 'example'\n");
    assert!(!append(
        &mut text,
        &mut Vec::new(),
        service,
        &ambiguous,
        &identifiers
    )?);
    assert!(
        disabled_block(
            &text,
            "create identifier matches multiple enumeration kinds"
        )
        .is_some_and(|block| block.contains("# operation = \"CreateChild\"")),
        "the disabled reason and operation should belong to the same block"
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

#[test]
fn given_required_request_identity_and_account_scope_when_drafted_then_reader_preserves_both()
-> TestResult {
    let bytes = extended_fixture(
        &[("PutThing", "example#Child", "smithy.api#Unit")],
        serde_json::json!({}),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let mut identifier = identifier(service, "ParentId")?;
    identifier.scope = PlanScope::Account;
    let identifiers = [identifier];
    let candidate =
        candidate(service, operation(service, "PutThing")?, &identifiers)?.ok_or("candidate")?;
    let mut text = String::from("format = 1\nservice = 'example'\n");
    assert!(append(
        &mut text,
        &mut Vec::new(),
        service,
        &candidate,
        &identifiers
    )?);
    let document: Value = toml::from_str(&text)?;
    let step = document
        .get("step")
        .and_then(Value::as_array)
        .and_then(|steps| steps.first())
        .ok_or("step")?;
    assert_eq!(
        step.get("identity")
            .and_then(|identity| identity.get("input"))
            .and_then(Value::as_str),
        Some("ParentId")
    );
    assert_eq!(step.get("scope").and_then(Value::as_str), Some("account"));
    let scenario = super::super::load(&text)?;
    assert_eq!(
        scenario.steps().first().ok_or("step")?.scope,
        super::super::SeedScope::Account
    );
    Ok(())
}

#[test]
fn given_optional_request_identifier_when_selected_then_no_unbound_identity_is_emitted()
-> TestResult {
    let bytes = extended_fixture(
        &[("PutThing", "example#Optional", "smithy.api#Unit")],
        serde_json::json!({
            "example#Optional": {"type":"structure","members":{"ThingId":{"target":"example#Id"}}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [identifier(service, "ThingId")?];
    assert_eq!(
        candidate(service, operation(service, "PutThing")?, &identifiers)?.map(|c| c.leaf.path),
        None
    );
    Ok(())
}

#[test]
fn given_earlier_plain_name_when_drafted_then_name_is_not_an_identifier_reference() -> TestResult {
    let bytes = fixture(MODEL)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let earlier = [Produced {
        step: "unrelated".into(),
        name: "Name".into(),
        path: "Name".into(),
    }];
    let synthesis = values::Synthesis {
        service,
        step: "thing",
        identifiers: &[],
        earlier: &earlier,
        own_input: None,
    };
    let input = synthesis.input(operation(service, "CreateThing")?.input()?)?;
    assert_eq!(
        input.get("Name").and_then(Value::as_str),
        Some("inv-thing-{n}")
    );
    Ok(())
}

#[test]
fn given_echoed_output_identifier_when_drafted_then_required_input_uses_earlier_producer()
-> TestResult {
    let bytes = extended_fixture(
        &[("CreateEcho", "example#Child", "example#Echo")],
        serde_json::json!({
            "example#Echo":{"type":"structure","members":{"ParentId":{"target":"example#Id"}}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [identifier(service, "ParentId")?];
    let candidate =
        candidate(service, operation(service, "CreateEcho")?, &identifiers)?.ok_or("candidate")?;
    let mut earlier = vec![Produced {
        step: "parent".into(),
        name: "ParentId".into(),
        path: "ParentId".into(),
    }];
    let mut text = String::new();
    assert!(append(
        &mut text,
        &mut earlier,
        service,
        &candidate,
        &identifiers
    )?);
    let document: Value = toml::from_str(&text)?;
    let reference = document
        .get("step")
        .and_then(Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("input"))
        .and_then(|input| input.get("ParentId"))
        .ok_or("reference")?;
    assert_eq!(
        reference.get("from").and_then(Value::as_str),
        Some("parent")
    );
    assert_eq!(
        reference.get("path").and_then(Value::as_str),
        Some("ParentId")
    );
    Ok(())
}

#[test]
fn given_list_request_identity_when_drafted_then_element_path_is_its_own_input() -> TestResult {
    let bytes = extended_fixture(
        &[("PutThing", "example#RulesInput", "smithy.api#Unit")],
        serde_json::json!({
            "example#RulesInput":{"type":"structure","members":{"Rules":{"target":"example#Rules","traits":{"smithy.api#required":{}}}}},
            "example#Rules":{"type":"list","member":{"target":"example#Rule"}},
            "example#Rule":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
        }),
    )?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [Identifier {
        kind: "AWS::Example::Thing".into(),
        name: "Name".into(),
        scope: PlanScope::Region,
        root_list: false,
    }];
    let candidate =
        candidate(service, operation(service, "PutThing")?, &identifiers)?.ok_or("candidate")?;
    assert_eq!(candidate.leaf.path, "Rules.0.Name");
    let mut text = String::new();
    assert!(append(
        &mut text,
        &mut Vec::new(),
        service,
        &candidate,
        &identifiers
    )?);
    let document: Value = toml::from_str(&text)?;
    let rule = document
        .get("step")
        .and_then(Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("input"))
        .and_then(|input| input.get("Rules"))
        .and_then(Value::as_array)
        .and_then(|rules| rules.first())
        .ok_or("rule")?;
    assert_eq!(
        rule.get("Name").and_then(Value::as_str),
        Some("inv-putthing-{n}")
    );
    Ok(())
}

fn sensitive_fixture() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    extended_fixture(
        &[
            ("PutShape", "example#ShapeSecret", "example#Output"),
            ("PutList", "example#ListSecret", "example#Output"),
            ("PutWireList", "example#WireListSecret", "example#Output"),
            ("PutNested", "example#NestedSecret", "example#Output"),
        ],
        serde_json::json!({
            "example#Password":{"type":"string","traits":{"smithy.api#sensitive":{}}},
            "example#ShapeSecret":{"type":"structure","members":{"Secret":{"target":"example#Password","traits":{"smithy.api#required":{}}}}},
            "example#Passwords":{"type":"list","member":{"target":"example#Password"}},
            "example#ListSecret":{"type":"structure","members":{"Entries":{"target":"example#Passwords","traits":{"smithy.api#required":{}}}}},
            "example#WirePasswords":{"type":"list","member":{"target":"smithy.api#String","traits":{"smithy.api#sensitive":{}}}},
            "example#WireListSecret":{"type":"structure","members":{"Entries":{"target":"example#WirePasswords","traits":{"smithy.api#required":{}}}}},
            "example#NestedSecret":{"type":"structure","members":{"Settings":{"target":"example#SecretInput","traits":{"smithy.api#required":{}}}}}
        }),
    )
}

#[test]
fn given_shape_list_and_nested_sensitive_inputs_when_drafted_then_every_step_is_disabled()
-> TestResult {
    let bytes = sensitive_fixture()?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let identifiers = [identifier(service, "ThingId")?];
    for (name, expected_reason) in [
        ("PutShape", "sensitive member Secret"),
        ("PutList", "sensitive input shape"),
        ("PutWireList", "sensitive list element"),
        ("PutNested", "sensitive member Settings.Secret"),
    ] {
        let candidate =
            candidate(service, operation(service, name)?, &identifiers)?.ok_or("candidate")?;
        let mut text = String::new();
        assert!(!append(
            &mut text,
            &mut Vec::new(),
            service,
            &candidate,
            &identifiers,
        )?);
        assert!(
            disabled_block(&text, expected_reason)
                .is_some_and(|block| block.contains(&format!("# operation = \"{name}\""))),
            "{name} should be disabled with its own reason and operation"
        );
    }
    Ok(())
}

#[test]
fn given_negated_and_variable_classes_when_synthesized_then_exact_legal_values_are_returned()
-> TestResult {
    for (pattern, expected) in [
        ("^[^0-9]{4}$", "inva"),
        ("^[a-z]+$", "invatopicaa"),
        ("^[a-z]*$", "invatopicaa"),
        ("^[a-z]{4,}$", "invatopicaa"),
        (r"^[\s]{4}$", "    "),
    ] {
        let constraints = ValueConstraints {
            pattern: Some(pattern),
            ..ValueConstraints::default()
        };
        let value = values::string("topic", constraints, ValueConstraints::default())?;
        assert_eq!(value, expected);
        assert_eq!(expanded_strings(&value)?, vec![expected.to_owned(); 10]);
    }
    let constraints = ValueConstraints {
        pattern: Some("^[z-a]+$"),
        ..ValueConstraints::default()
    };
    let error = values::string("topic", constraints, ValueConstraints::default())
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft pattern range is invalid"
    );
    Ok(())
}

#[test]
fn given_no_create_operations_and_unit_input_when_drafted_then_empty_draft_is_rejected_and_unit_is_empty()
-> TestResult {
    let bytes = catalog_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let error = generate(&runtime, "sts").err().ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft has no enabled steps"
    );
    let service = runtime.catalog().service("sts")?.ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "unit",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    assert_eq!(
        synthesis.input(Target::Prelude("unit"))?,
        Value::Table(Table::new())
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn given_symlink_destinations_when_guarded_then_checked_in_seeds_are_never_targeted() -> TestResult
{
    let root = tempfile::tempdir()?;
    let seeds = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    assert!(seeds.join("sns.toml").is_file());
    for (name, target, suffix) in [
        ("file", seeds.join("sns.toml"), None),
        ("directory", seeds.clone(), Some("draft.toml")),
        (
            "dangling",
            seeds.join("nonexistent-draft-target.toml"),
            None,
        ),
    ] {
        let link = root.path().join(name);
        std::os::unix::fs::symlink(target, &link)?;
        let output = suffix.map_or(link.clone(), |suffix| link.join(suffix));
        let error = validate_output(&output).err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            if name == "dangling" {
                "invalid emulator seed configuration: draft output symlink target cannot be resolved"
            } else {
                "invalid emulator seed configuration: draft output must be outside crates/harness/seeds"
            }
        );
    }
    Ok(())
}

#[test]
fn given_depth_four_and_five_when_required_input_is_synthesized_then_only_depth_four_succeeds()
-> TestResult {
    let bytes = deep_fixture(4)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "deep",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    let input = synthesis.input(operation(service, "PutDeep")?.input()?)?;
    assert_eq!(
        input
            .get("A")
            .and_then(|v| v.get("B"))
            .and_then(|v| v.get("C"))
            .and_then(|v| v.get("D"))
            .and_then(Value::as_str),
        Some("inv-deep-{n}")
    );
    let bytes = deep_fixture(5)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let synthesis = values::Synthesis {
        service,
        step: "deep",
        identifiers: &[],
        earlier: &[],
        own_input: None,
    };
    let input = operation(service, "PutDeep")?.input()?;
    let error = synthesis.input(input).err().ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: draft required input exceeds depth 4"
    );
    assert_eq!(
        leaves(service, input, true)?
            .iter()
            .map(|leaf| leaf.path.as_str())
            .collect::<Vec<_>>(),
        vec!["A.B.C.D.E"]
    );
    Ok(())
}

fn deep_fixture(depth: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut shapes = serde_json::Map::new();
    for (index, name) in ["A", "B", "C", "D", "E"]
        .into_iter()
        .take(depth)
        .enumerate()
    {
        let next = index.saturating_add(1);
        let target = if next == depth {
            String::from("smithy.api#String")
        } else {
            format!("example#Level{next}")
        };
        let members = serde_json::Map::from_iter([(
            name.to_owned(),
            serde_json::json!({
                "target": target, "traits": {"smithy.api#required": {}}
            }),
        )]);
        shapes.insert(
            format!("example#Level{index}"),
            serde_json::json!({
                "type": "structure", "members": members
            }),
        );
    }
    extended_fixture(
        &[("PutDeep", "example#Level0", "smithy.api#Unit")],
        serde_json::Value::Object(shapes),
    )
}

#[test]
fn given_sixty_five_required_members_when_synthesized_then_exact_budget_error_is_returned()
-> TestResult {
    let members = (0..65)
        .map(|index| {
            (
                format!("Field{index}"),
                serde_json::json!({
                    "target": "smithy.api#String", "traits": {"smithy.api#required": {}}
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let bytes = extended_fixture(
        &[("PutWide", "example#Wide", "smithy.api#Unit")],
        serde_json::json!({
            "example#Wide": {"type": "structure", "members": members}
        }),
    )?;
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
        "invalid emulator seed configuration: draft structure exceeds input budget"
    );
    Ok(())
}
