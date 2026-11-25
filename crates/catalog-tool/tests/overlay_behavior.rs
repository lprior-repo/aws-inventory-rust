use catalog_tool::overlay::{
    self, CheckError, EffectiveClass, ItemSelector, OperationOverlay, OverlayClass, ServiceOverlay,
    Violation, ViolationRule,
};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;
#[path = "overlay_behavior/accepted_bind.rs"]
mod overlay_accepted_bind;
#[path = "overlay_behavior/extra.rs"]
mod overlay_behavior_extra;
#[path = "overlay_behavior/bind_paths.rs"]
mod overlay_bind_paths;
#[path = "overlay_behavior/capability_parity.rs"]
mod overlay_capability_parity;
#[path = "overlay_behavior/deny_properties.rs"]
mod overlay_deny_properties;
#[path = "overlay_behavior/enum_fixed.rs"]
mod overlay_enum_fixed;
#[path = "overlay_behavior/fixed_paths.rs"]
mod overlay_fixed_paths;
#[path = "overlay_behavior/page_size.rs"]
mod overlay_page_size;
#[path = "overlay_behavior/primitive_alias.rs"]
mod overlay_primitive_alias;
#[path = "overlay_behavior/scalar_items.rs"]
mod overlay_scalar_items;
#[path = "overlay_behavior/token_lists.rs"]
mod overlay_token_lists;
#[path = "overlay_behavior/token_maps.rs"]
mod overlay_token_maps;
#[path = "overlay_behavior/typed_bindings.rs"]
mod overlay_typed_bindings;
#[path = "overlay_behavior/unclassified.rs"]
mod overlay_unclassified;

const BASIC: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

struct Fixture<'a> {
    root: TempDir,
    deny: &'a str,
}

impl Fixture<'_> {
    fn path(&self) -> &Path {
        self.root.path()
    }
}

fn fixture<'a>(model: &str, deny: &'a str) -> Result<Fixture<'a>, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("catalog"))?;
    fs::create_dir_all(root.path().join("source/models/things/service/2020-01-01"))?;
    fs::write(
        root.path()
            .join("source/models/things/service/2020-01-01/things.json"),
        model,
    )?;
    Ok(Fixture { root, deny })
}

fn source(root: &Path) -> std::path::PathBuf {
    root.join("source")
}

fn checked(
    root: &Fixture<'_>,
    operations: Vec<(&str, OperationOverlay)>,
) -> Result<overlay::CheckedOverlays, Box<CheckError>> {
    let policies = [ServiceOverlay::new("things", operations)
        .map_err(|error| Box::new(CheckError::from(error)))?];
    let deny = catalog_tool::classes::DenyList::parse(root.deny)
        .map_err(|error| Box::new(CheckError::from(error)))?;
    overlay::check_policies(&source(root.path()), &policies, &deny).map_err(Box::new)
}

fn violations(
    root: &Fixture<'_>,
    operations: Vec<(&str, OperationOverlay)>,
) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    match checked(root, operations).map_err(|error| *error) {
        Err(CheckError::Violations(found)) => Ok(found),
        Err(other) => Err(other.into()),
        Ok(_) => Err("expected semantic violations".into()),
    }
}

fn command(root: &Path) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["overlay", "--root"])
        .arg(root)
        .args(["--source"])
        .arg(source(root))
        .arg("--write")
        .output()
}

fn inferred() -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    Ok(plan)
}

fn complete() -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = inferred()?;
    plan.items = Some(ItemSelector::Member(overlay::MemberPath::new("Items")?));
    plan.paginator = Some(overlay::Paginator::None);
    Ok(plan)
}

fn deferred() -> OperationOverlay {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Defer);
    plan.defer_reason = Some("Requires a separate reviewed inventory plan".to_owned());
    plan
}

fn token(input: &str, output: &str) -> Result<overlay::Paginator, overlay::NameError> {
    Ok(overlay::Paginator::Token(overlay::TokenPair::new(
        overlay::MemberPath::new(input)?,
        overlay::MemberPath::new(output)?,
    )))
}

fn batch(replaces: &str) -> Result<overlay::Batch, overlay::NameError> {
    Ok(overlay::Batch::new(
        overlay::MemberPath::new("Ids")?,
        10,
        overlay::OperationName::new(replaces)?,
    ))
}

fn binding(operation: &str, path: &str) -> Result<overlay::Binding, overlay::NameError> {
    Ok(overlay::Binding::From {
        operation: overlay::OperationName::new(operation)?,
        path: overlay::MemberPath::selected(path)?,
    })
}

#[test]
fn callable_handle_contains_exact_reviewed_plan_and_class() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture(BASIC, "")?;
    let checked = checked(&root, vec![("ListThings", complete()?)])?;
    let service = checked
        .services()
        .first()
        .ok_or("missing reviewed service")?;
    let callable = service
        .callable()
        .first()
        .ok_or("missing callable handle")?;
    assert_eq!(service.service(), "things");
    assert_eq!(callable.service(), "things");
    assert_eq!(callable.operation(), "ListThings");
    assert_eq!(callable.class(), EffectiveClass::Enumerate);
    assert_eq!(
        callable.plan().items,
        Some(ItemSelector::Member(overlay::MemberPath::new("Items")?))
    );
    assert_eq!(callable.plan().paginator, Some(overlay::Paginator::None));
    assert_eq!(
        callable.plan().identity,
        Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
            "Id"
        )?]))
    );
    assert!(checked.report().contains("callable_enumerate=1"));
    assert!(
        checked
            .report()
            .contains("operation\tthings\tListThings\tenumerate")
    );
    Ok(())
}

#[test]
fn promotion_rejects_global_scoped_and_no_read_verb_denials()
-> Result<(), Box<dyn std::error::Error>> {
    for (operation, deny) in [
        ("ListThings", "ListThings\tcontent\n"),
        ("ListThings", "things/ListThings\tsecret\n"),
        ("CreateThings", ""),
    ] {
        for class in [OverlayClass::Enumerate, OverlayClass::Expand] {
            let model = BASIC.replace("ListThings", operation);
            let root = fixture(&model, deny)?;
            let mut plan = complete()?;
            plan.class = Some(class);
            let errors = violations(&root, vec![(operation, plan)])?;
            assert!(
                errors.iter().any(|error| error.operation() == operation
                    && error.key() == "class"
                    && error.value() == "enumerate/expand"
                    && error.rule() == ViolationRule::OverlayPromotesDenied),
                "{operation}/{class:?}: {errors:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn one_fixed_nested_child_does_not_satisfy_its_required_sibling()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Parent":{"target":"n#Parent","traits":{"smithy.api#required":{}}}}},
"n#Parent":{"type":"structure","members":{"Child":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"Sibling":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;
    let root = fixture(model, "")?;
    let mut plan = complete()?;
    plan.fixed.insert(
        overlay::MemberPath::new("Parent.Child")?,
        overlay::FixedValue::String("x".to_owned()),
    );
    let errors = violations(&root, vec![("ListThings", plan.clone())])?;
    let required: Vec<_> = errors
        .iter()
        .filter(|error| error.key() == "required")
        .map(Violation::value)
        .collect();
    assert_eq!(required, ["Parent.Sibling"]);
    let optional_model = model.replace(
        "\"Parent\":{\"target\":\"n#Parent\",\"traits\":{\"smithy.api#required\":{}}}",
        "\"Parent\":{\"target\":\"n#Parent\"}",
    );
    let root = fixture(&optional_model, "")?;
    let errors = violations(&root, vec![("ListThings", plan)])?;
    let required: Vec<_> = errors
        .iter()
        .filter(|error| error.key() == "required")
        .map(Violation::value)
        .collect();
    assert_eq!(required, ["Parent.Sibling"]);
    Ok(())
}

#[test]
fn inferred_effective_bindings_cannot_form_a_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#DescribeA"},{"target":"n#DescribeB"}]},
"n#DescribeA":{"type":"operation","input":{"target":"n#InputA"},"output":{"target":"n#Output"}},
"n#DescribeB":{"type":"operation","input":{"target":"n#InputB"},"output":{"target":"n#Output"}},
"n#InputA":{"type":"structure","members":{"ParentA":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
"n#InputB":{"type":"structure","members":{"ParentB":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Rows"}}},
"n#Rows":{"type":"list","member":{"target":"n#Row"}},
"n#Row":{"type":"structure","members":{"ParentA":{"target":"smithy.api#String"},"ParentB":{"target":"smithy.api#String"}}},
"n#ResourceA":{"type":"resource","identifiers":{"ParentA":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeA"},"list":{"target":"n#DescribeB"}},
"n#ResourceB":{"type":"resource","identifiers":{"ParentB":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeB"},"list":{"target":"n#DescribeA"}}
}}"#;
    let root = fixture(model, "")?;
    let mut a = complete()?;
    a.class = Some(OverlayClass::Expand);
    a.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "ParentA",
    )?]));
    let mut b = complete()?;
    b.class = Some(OverlayClass::Expand);
    b.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "ParentB",
    )?]));
    let errors = violations(&root, vec![("DescribeA", a), ("DescribeB", b)])?;
    let cycles: Vec<_> = errors
        .iter()
        .filter(|error| error.rule() == ViolationRule::BindingCycle)
        .map(|error| error.operation())
        .collect();
    assert_eq!(cycles, ["DescribeA", "DescribeB"]);
    Ok(())
}

#[test]
fn semantic_errors_are_sorted_and_failed_write_preserves_report_and_stdout()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture(BASIC, "")?;
    let mut plan = complete()?;
    plan.items = Some(ItemSelector::Member(overlay::MemberPath::new("Missing")?));
    plan.queryable = vec![overlay::MemberPath::new("Absent")?];
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Absent",
    )?]));
    let errors = violations(&root, vec![("ListThings", plan)])?;
    let keys: Vec<_> = errors.iter().map(Violation::key).collect();
    assert_eq!(keys, ["identity", "items", "queryable"]);
    fs::write(root.path().join("catalog/aws-overlay.report"), "previous\n")?;
    fs::remove_dir_all(source(root.path()))?;
    let output = command(root.path())?;
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        fs::read(root.path().join("catalog/aws-overlay.report"))?,
        b"previous\n"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("model"), "{stderr}");
    Ok(())
}
