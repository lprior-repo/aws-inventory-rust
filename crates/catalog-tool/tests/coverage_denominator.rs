use std::path::PathBuf;

use catalog_tool::{
    ReportMode,
    coverage::{self, Count, Error, Funnel, Stage},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const INPUTS: [&str; 7] = [
    "catalog/aws-models.report",
    "catalog/aws-plans.report",
    "catalog/aws-classes.report",
    "catalog/aws-overlay.report",
    "catalog/aws-operations.json",
    "catalog/cloudcontrol-capabilities.json",
    "catalog/cloudcontrol-resources.json",
];

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Copies only the artifacts the coverage denominator reads, so tampering never touches the repo.
fn copy_inputs() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    for path in INPUTS {
        let target = dir.path().join(path);
        std::fs::create_dir_all(target.parent().ok_or("input without parent")?)?;
        std::fs::copy(real_root().join(path), target)?;
    }
    Ok(dir)
}

fn recorded(name: &str, count: u64) -> Stage {
    Stage {
        name: name.to_owned(),
        count: Count::Recorded(count),
    }
}

fn funnel(stages: Vec<Stage>) -> Funnel {
    Funnel {
        name: "test",
        stages,
        excluded: Vec::new(),
        notes: Vec::new(),
        basis: Vec::new(),
    }
}

#[test]
fn committed_catalog_stages_never_increase() -> TestResult {
    let report = coverage::build(&real_root())?;
    assert!(!report.funnels.is_empty());
    for funnel in &report.funnels {
        assert_eq!(funnel.first_violation(), None, "funnel {}", funnel.name);
    }
    Ok(())
}

#[test]
fn a_stage_above_its_predecessor_is_reported() {
    let increasing = funnel(vec![recorded("known", 10), recorded("selected", 11)]);
    assert_eq!(increasing.first_violation(), Some("selected"));
}

#[test]
fn equal_and_decreasing_recorded_stages_pass() {
    let ok = funnel(vec![
        recorded("known", 10),
        recorded("selected", 10),
        recorded("tested", 3),
    ]);
    assert_eq!(ok.first_violation(), None);
}

#[test]
fn not_recorded_stages_are_skipped_by_monotone_check() {
    let skipped = funnel(vec![
        recorded("known", 5),
        Stage {
            name: "fixture_tested".to_owned(),
            count: Count::NotRecorded,
        },
        recorded("emulator_tested", 5),
    ]);
    assert_eq!(skipped.first_violation(), None);
}

#[test]
fn rendered_report_is_byte_stable_across_builds() -> TestResult {
    let first = coverage::build(&real_root())?.render()?;
    let second = coverage::build(&real_root())?.render()?;
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn committed_report_matches_a_fresh_build() -> TestResult {
    let rendered = coverage::build(&real_root())?.render()?;
    let committed = std::fs::read_to_string(real_root().join(coverage::REPORT_PATH))?;
    assert_eq!(committed, rendered);
    Ok(())
}

#[test]
fn digest_changes_when_a_catalog_input_changes() -> TestResult {
    let dir = copy_inputs()?;
    let before = coverage::build(dir.path())?;

    let target = dir.path().join("catalog/cloudcontrol-capabilities.json");
    let mut bytes = std::fs::read(&target)?;
    bytes.push(b'\n');
    std::fs::write(&target, bytes)?;

    let after = coverage::build(dir.path())?;
    assert_ne!(before.digest, after.digest);
    assert_eq!(before.funnels, after.funnels);
    Ok(())
}

#[test]
fn digest_is_stable_for_identical_inputs() -> TestResult {
    let first = coverage::build(&real_root())?;
    let dir = copy_inputs()?;
    let copied = coverage::build(dir.path())?;
    assert_eq!(first.digest, copied.digest);
    assert_eq!(first.render()?, copied.render()?);
    Ok(())
}

#[test]
fn overlay_totals_that_disagree_with_rows_are_refused() -> TestResult {
    let dir = copy_inputs()?;
    let target = dir.path().join("catalog/aws-overlay.report");
    let text = std::fs::read_to_string(&target)?;
    assert!(text.contains("callable_enumerate=1565"));
    std::fs::write(
        &target,
        text.replacen("callable_enumerate=1565", "callable_enumerate=1566", 1),
    )?;
    assert!(matches!(
        coverage::build(dir.path()),
        Err(Error::Invalid { .. })
    ));
    Ok(())
}

#[test]
fn missing_catalog_input_is_refused() -> TestResult {
    let dir = copy_inputs()?;
    std::fs::remove_file(dir.path().join("catalog/aws-operations.json"))?;
    assert!(matches!(coverage::build(dir.path()), Err(Error::Io { .. })));
    Ok(())
}

#[test]
fn print_mode_never_writes_the_report() -> TestResult {
    let dir = copy_inputs()?;
    coverage::run(dir.path(), ReportMode::Print)?;
    assert!(!dir.path().join(coverage::REPORT_PATH).exists());
    Ok(())
}

#[test]
fn check_mode_refuses_a_missing_report() -> TestResult {
    let dir = copy_inputs()?;
    assert!(coverage::run(dir.path(), ReportMode::Check).is_err());
    Ok(())
}
