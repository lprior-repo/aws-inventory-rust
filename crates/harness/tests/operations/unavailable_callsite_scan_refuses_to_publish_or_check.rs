//! Behavior: an unavailable callsite scan cannot be published as exercised coverage.
use super::reviewed_additions::overlay_fixture;
use super::*;

#[test]
fn unavailable_callsite_scan_refuses_to_publish_or_check_coverage() -> TestResult {
    let directory = overlay_fixture()?;
    let root = directory.path();
    std::fs::write(
        root.join("crates/aws/src/collect.rs"),
        "fn collect(alpha, beta, cloud) {\n    alpha.list_things().send();\n}\n",
    )?;
    let inspection = operation_coverage(root)?.to_value()?;
    let status = pointer(&inspection, "/sources/callsite_scan/status")?
        .as_str()
        .ok_or("missing callsite scan status")?;
    assert!(status.starts_with("unavailable:"), "{status}");
    assert_eq!(
        pointer(&inspection, "/reviewed_additions/0/execution_status")?,
        &json!("unexercised")
    );
    for (write, check) in [
        (Some(Path::new("docs/OPERATION_COVERAGE.md")), false),
        (None, true),
    ] {
        let failure = match run_operation_coverage(root, write, check) {
            Ok(_) => return Err("expected coverage publication refusal".into()),
            Err(error) => error.to_string(),
        };
        assert!(
            failure.contains("exercised count is unavailable"),
            "{failure}"
        );
        assert!(failure.contains("callsite scan"), "{failure}");
    }
    Ok(())
}
