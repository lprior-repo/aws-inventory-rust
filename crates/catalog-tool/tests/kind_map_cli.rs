use std::{fs, path::Path, process::{Command, Output}};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListWidgets"}],"traits":{
"aws.api#service":{"sdkId":"Example Service","cloudFormationName":"Example"}}},
"n#ListWidgets":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{}},
"n#Output":{"type":"structure","members":{"Widgets":{"target":"n#Widgets"}}},
"n#Widgets":{"type":"list","member":{"target":"n#Widget"}},
"n#Widget":{"type":"structure","members":{"Arn":{"target":"smithy.api#String"},"Name":{"target":"smithy.api#String"}}}
}}"#;
const POLICY: &str = "format = 1\nreviewed = '2026-11-15'\nreviewer = 'generated'\n[operation.ListWidgets]\nclass = 'enumerate'\nscope = 'region'\nitems = 'Widgets'\npaginator = { kind = 'none' }\nidentity = { arn = 'Arn' }\n";

fn given_root() -> Result<tempfile::TempDir> {
    let root = tempfile::tempdir()?;
    let model = root.path().join("models/example/service/2026-01-01");
    let policy = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&model)?;
    fs::create_dir_all(&policy)?;
    fs::write(model.join("model.json"), MODEL)?;
    fs::write(policy.join("example.toml"), POLICY)?;
    fs::write(root.path().join("catalog/cloudcontrol-universal.json"),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"))?;
    Ok(root)
}

fn when_generate(root: &Path, mode: &str) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["kinds", "generate", "--root"]).arg(root)
        .arg("--source").arg(root).arg(mode).output()?)
}

fn then_success(output: Output) -> Result {
    assert!(output.status.success(), "{}", String::from_utf8(output.stderr)?);
    Ok(())
}

#[test]
fn kind_map_given_model_and_policy_when_written_then_check_and_regeneration_are_identical() -> Result {
    let root = given_root()?;
    then_success(when_generate(root.path(), "--write")?)?;
    let path = root.path().join("catalog/kinds.generated.tsv");
    let before = fs::read(&path)?;
    assert!(std::str::from_utf8(&before)?.contains("AWS::ApiOnly::ExampleServiceWidget"));
    then_success(when_generate(root.path(), "--check")?)?;
    then_success(when_generate(root.path(), "--write")?)?;
    assert_eq!(fs::read(path)?, before);
    Ok(())
}

#[test]
fn kind_map_given_changed_generated_file_when_checked_then_drift_is_not_overwritten() -> Result {
    let root = given_root()?;
    then_success(when_generate(root.path(), "--write")?)?;
    let path = root.path().join("catalog/kinds.generated.tsv");
    fs::write(&path, "changed\n")?;
    let output = when_generate(root.path(), "--check")?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("generated kinds differ"));
    assert_eq!(fs::read(path)?, b"changed\n");
    Ok(())
}

fn given_conflicting_models(root: &Path) -> Result {
    let mut value: serde_json::Value = serde_json::from_str(MODEL)?;
    let shapes = value.get_mut("shapes").and_then(serde_json::Value::as_object_mut).ok_or("shapes absent")?;
    for (name, shape) in [
        ("n#ListOtherWidgets", serde_json::json!({"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#OtherOutput"}})),
        ("n#OtherOutput", serde_json::json!({"type":"structure","members":{"Widgets":{"target":"n#OtherWidgets"}}})),
        ("n#OtherWidgets", serde_json::json!({"type":"list","member":{"target":"n#WidgetDescription"}})),
        ("n#WidgetDescription", serde_json::json!({"type":"structure","members":{"Arn":{"target":"smithy.api#String"}}})),
    ] {
        shapes.insert(name.to_owned(), shape);
    }
    value.pointer_mut("/shapes/n#Service/operations").and_then(serde_json::Value::as_array_mut)
        .ok_or("operations absent")?.push(serde_json::json!({"target":"n#ListOtherWidgets"}));
    let traits = value.pointer_mut("/shapes/n#Service/traits").and_then(serde_json::Value::as_object_mut).ok_or("traits absent")?;
    traits.insert("aws.auth#sigv4".to_owned(), serde_json::json!({"name":"example"}));
    traits.insert("smithy.rules#endpointRuleSet".to_owned(), serde_json::json!({
        "version":"1.0","parameters":{},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://example.test"}}]}));
    traits.insert("smithy.rules#endpointTests".to_owned(), serde_json::json!({
        "testCases":[{"params":{},"expect":{"endpoint":{"url":"https://example.test"}}}]}));
    fs::write(root.join("models/example/service/2026-01-01/model.json"), serde_json::to_vec(&value)?)?;
    Ok(())
}

#[test]
fn kind_map_given_conflicting_item_shapes_when_overlays_generated_then_second_is_deferred() -> Result {
    use catalog_tool::{classes::DenyList, overlay::{OperationName, generate::Generator}};
    let root = given_root()?;
    given_conflicting_models(root.path())?;
    let generator = Generator::read(root.path())?;
    let generated = generator.generate(&std::collections::BTreeSet::from(["example".to_owned()]), &DenyList::builtin())?;
    let policy = generated.policies().first().ok_or("generated policy absent")?;
    let deferred = policy.operations().get(&OperationName::new("ListWidgets")?).ok_or("operation absent")?;
    assert_eq!(deferred.defer_reason.as_deref(), Some(catalog_tool::kind_map::generate::CONFLICT), "{} {:?}", generated.report(), policy.operations());
    assert!(generated.report().contains("totals enumerate_candidates=2 callable=1 deferred=1"));
    Ok(())
}
