use catalog_tool::overlay::{
    self, CheckError, EffectiveClass, ItemSelector, Violation, ViolationRule,
};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;
#[path = "overlay_behavior/accepted_bind.rs"]
mod overlay_accepted_bind;
#[path = "overlay_behavior/extra.rs"]
mod overlay_behavior_extra;
#[path = "overlay_behavior/bind_paths.rs"]
mod overlay_bind_paths;
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
#[path = "overlay_behavior/token_lists.rs"]
mod overlay_token_lists;
#[path = "overlay_behavior/token_maps.rs"]
mod overlay_token_maps;
#[path = "overlay_behavior/unclassified.rs"]
mod overlay_unclassified;
#[path = "overlay_behavior/scalar_items.rs"]
mod overlay_scalar_items;

const BASIC: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn fixture(model: &str, deny: &str) -> Result<TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("catalog/overlay"))?;
    fs::create_dir_all(root.path().join("source/models/things/service/2020-01-01"))?;
    fs::write(root.path().join("catalog/overlay/must-deny.txt"), deny)?;
    fs::write(
        root.path()
            .join("source/models/things/service/2020-01-01/things.json"),
        model,
    )?;
    Ok(root)
}

fn source(root: &Path) -> std::path::PathBuf {
    root.join("source")
}

fn overlay(root: &Path, text: &str) -> std::io::Result<()> {
    fs::write(root.join("catalog/overlay/things.toml"), text)
}

fn violations(root: &Path) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    match overlay::check(root, &source(root)) {
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

const HEADER: &str = "format=1\nreviewed='2026-11-13'\nreviewer='fixture'\n";
const COMPLETE: &str = "[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n";

#[test]
fn callable_handle_contains_exact_reviewed_plan_and_class() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture(BASIC, "")?;
    overlay(root.path(), &format!("{HEADER}{COMPLETE}"))?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
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
        for class in ["enumerate", "expand"] {
            let model = BASIC.replace("ListThings", operation);
            let root = fixture(&model, deny)?;
            let plan = COMPLETE
                .replace("ListThings", operation)
                .replace("class='enumerate'", &format!("class='{class}'"));
            overlay(root.path(), &format!("{HEADER}{plan}"))?;
            let errors = violations(root.path())?;
            assert!(
                errors.iter().any(|error| error.operation() == operation
                    && error.key() == "class"
                    && error.value() == "enumerate/expand"
                    && error.rule() == ViolationRule::OverlayPromotesDenied),
                "{operation}/{class}: {errors:?}"
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
    let text = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.fixed]\n'Parent.Child'='x'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
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
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
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
    let text = format!(
        "{HEADER}[operation.DescribeA]\nclass='expand'\nitems='Items'\n[operation.DescribeA.paginator]\nkind='none'\n[operation.DescribeA.identity]\nkey=['ParentA']\n[operation.DescribeB]\nclass='expand'\nitems='Items'\n[operation.DescribeB.paginator]\nkind='none'\n[operation.DescribeB.identity]\nkey=['ParentB']\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
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
    let text = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Missing'\nqueryable=['Absent']\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Absent']\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
    let keys: Vec<_> = errors.iter().map(Violation::key).collect();
    assert_eq!(keys, ["identity", "items", "queryable"]);
    fs::write(root.path().join("catalog/aws-overlay.report"), "previous\n")?;
    let output = command(root.path())?;
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        fs::read(root.path().join("catalog/aws-overlay.report"))?,
        b"previous\n"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr
            .find("identity=")
            .ok_or("missing identity diagnostic")?
            < stderr.find("items=").ok_or("missing items diagnostic")?
    );
    assert!(
        stderr.find("items=").ok_or("missing items diagnostic")?
            < stderr
                .find("queryable=")
                .ok_or("missing queryable diagnostic")?
    );
    Ok(())
}
