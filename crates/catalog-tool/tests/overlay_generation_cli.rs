use serde_json::json;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SELECTION: &str = "catalog/overlay/generated.services";
const REPORT: &str = "catalog/aws-overlay-generated.report";

fn given_models() -> Result<tempfile::TempDir> {
    let root = tempfile::tempdir()?;
    for service in ["alpha", "beta"] {
        let directory = root
            .path()
            .join(format!("models/{service}/service/2026-01-01"));
        fs::create_dir_all(&directory)?;
        fs::write(directory.join("model.json"), serde_json::to_vec(&model())?)?;
    }
    fs::create_dir_all(root.path().join("catalog/overlay"))?;
    fs::write(root.path().join(SELECTION), "alpha\nbeta\n")?;
    Ok(root)
}

fn model() -> serde_json::Value {
    json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","operations":[{"target":"n#ListThings"}],"traits":{
            "aws.auth#sigv4":{"name":"sample"},
            "smithy.rules#endpointRuleSet":{"version":"1.0","parameters":{},"rules":[{
                "type":"endpoint","conditions":[],"endpoint":{"url":"https://sample.example"}}]},
            "smithy.rules#endpointTests":{"testCases":[{"params":{},"expect":{
                "endpoint":{"url":"https://sample.example"}}}]}}},
        "n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
        "n#Input":{"type":"structure","members":{}},
        "n#Output":{"type":"structure","members":{"Things":{"target":"n#Things"}}},
        "n#Things":{"type":"list","member":{"target":"n#Thing"}},
        "n#Thing":{"type":"structure","members":{"Arn":{"target":"smithy.api#String"}}}
    }})
}

fn when_generate(root: &Path, flags: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["overlays", "generate", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .args(flags)
        .output()?)
}

fn then_success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn generated(root: &Path, service: &str) -> std::path::PathBuf {
    root.join(format!("catalog/overlay/generated/{service}.toml"))
}

#[test]
fn given_two_models_when_written_then_check_passes_and_totals_are_exact() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    then_success(when_generate(root.path(), &["--check"])?);
    let report = fs::read_to_string(root.path().join(REPORT))?;
    assert!(report.contains("service alpha callable=1 deferred=0"));
    assert!(report.contains("totals enumerate_candidates=2 callable=2 deferred=0"));
    assert!(
        fs::read_to_string(generated(root.path(), "alpha"))?.contains("reviewer = \"generated\"")
    );
    Ok(())
}

#[test]
fn given_changed_file_when_checked_then_drift_fails_without_overwriting_it() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    let file = generated(root.path(), "alpha");
    fs::write(&file, "changed\n")?;
    assert!(!when_generate(root.path(), &["--check"])?.status.success());
    assert_eq!(fs::read_to_string(file)?, "changed\n");
    Ok(())
}

#[test]
fn given_removed_selection_when_written_then_stale_generated_file_is_deleted() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    fs::write(root.path().join(SELECTION), "alpha\n")?;
    then_success(when_generate(root.path(), &["--write"])?);
    assert!(!generated(root.path(), "beta").exists());
    assert!(generated(root.path(), "alpha").exists());
    then_success(when_generate(root.path(), &["--check"])?);
    Ok(())
}

#[test]
fn given_extra_or_missing_file_when_checked_then_exact_set_is_enforced() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    fs::write(generated(root.path(), "extra"), "extra\n")?;
    assert!(!when_generate(root.path(), &["--check"])?.status.success());
    fs::remove_file(generated(root.path(), "extra"))?;
    fs::remove_file(generated(root.path(), "alpha"))?;
    assert!(!when_generate(root.path(), &["--check"])?.status.success());
    Ok(())
}

#[test]
fn given_service_override_when_check_requested_then_arguments_are_refused() -> Result {
    let root = given_models()?;
    let output = when_generate(root.path(), &["--service", "alpha", "--check"])?;
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)?.contains("--service cannot be combined with --check")
    );
    assert!(!generated(root.path(), "alpha").exists());
    Ok(())
}

#[test]
fn given_print_mode_when_generated_then_no_artifacts_are_created() -> Result {
    let root = given_models()?;
    let output = when_generate(root.path(), &[])?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("enumerate_candidates=2"));
    assert!(!root.path().join(REPORT).exists());
    assert!(!generated(root.path(), "alpha").exists());
    Ok(())
}

#[test]
fn given_empty_selection_when_checked_then_no_generated_directory_is_needed() -> Result {
    let root = given_models()?;
    fs::write(root.path().join(SELECTION), "")?;
    then_success(when_generate(root.path(), &["--write"])?);
    then_success(when_generate(root.path(), &["--check"])?);
    assert!(!generated(root.path(), "alpha").exists());
    Ok(())
}

#[test]
fn given_service_override_when_written_then_selection_file_is_ignored() -> Result {
    let root = given_models()?;
    then_success(when_generate(
        root.path(),
        &["--service", "beta", "--write"],
    )?);
    assert!(!generated(root.path(), "alpha").exists());
    assert!(generated(root.path(), "beta").exists());
    let report = fs::read_to_string(root.path().join(REPORT))?;
    assert!(report.contains("totals enumerate_candidates=1 callable=1 deferred=0"));
    Ok(())
}

#[test]
fn given_all_selection_when_generated_then_each_model_is_selected() -> Result {
    let root = given_models()?;
    fs::write(root.path().join(SELECTION), "*\n")?;
    then_success(when_generate(root.path(), &["--write"])?);
    then_success(when_generate(root.path(), &["--check"])?);
    assert!(generated(root.path(), "alpha").exists());
    assert!(generated(root.path(), "beta").exists());
    Ok(())
}

#[test]
fn given_invalid_selection_when_generated_then_no_policy_is_published() -> Result {
    let root = given_models()?;
    for selection in [
        "beta\nalpha\n",
        "alpha\nalpha\n",
        "\n",
        "alpha",
        "unknown\n",
    ] {
        fs::write(root.path().join(SELECTION), selection)?;
        let output = when_generate(root.path(), &["--write"])?;
        assert!(!output.status.success(), "{selection:?}");
        assert!(!generated(root.path(), "alpha").exists());
    }
    Ok(())
}

#[test]
fn given_new_root_deny_when_regenerated_then_previously_callable_entry_is_absent() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    fs::write(
        root.path().join("catalog/must-deny.txt"),
        "alpha/ListThings\tsecret\n",
    )?;
    then_success(when_generate(root.path(), &["--write"])?);
    then_success(when_generate(root.path(), &["--check"])?);
    let policy = fs::read_to_string(generated(root.path(), "alpha"))?;
    assert!(!policy.contains("[operation.ListThings]"));
    let report = fs::read_to_string(root.path().join(REPORT))?;
    assert!(report.contains("totals enumerate_candidates=1 callable=1 deferred=0"));
    Ok(())
}

#[test]
fn given_changed_report_when_checked_then_report_drift_fails() -> Result {
    let root = given_models()?;
    then_success(when_generate(root.path(), &["--write"])?);
    fs::write(root.path().join(REPORT), "changed\n")?;
    let output = when_generate(root.path(), &["--check"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("report differs or is missing"));
    Ok(())
}

#[test]
fn given_root_deny_when_generated_policy_is_tampered_then_overlay_gate_rejects_it() -> Result {
    let root = given_models()?;
    fs::write(
        root.path().join("catalog/must-deny.txt"),
        "alpha/ListThings\tsecret\n",
    )?;
    then_success(when_generate(root.path(), &["--write"])?);
    let path = generated(root.path(), "alpha");
    let mut text = fs::read_to_string(&path)?;
    text.push_str("\n[operation.ListThings]\nclass = \"enumerate\"\n");
    fs::write(path, text)?;
    match catalog_tool::overlay::check(root.path(), root.path()) {
        Err(catalog_tool::overlay::CheckError::Violations(violations)) => {
            assert!(violations.iter().any(|violation| violation.rule()
                == catalog_tool::overlay::ViolationRule::OverlayPromotesDenied));
        }
        other => return Err(format!("unexpected gate result: {other:?}").into()),
    }
    Ok(())
}
