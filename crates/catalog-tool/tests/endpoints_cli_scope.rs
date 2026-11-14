pub mod endpoints_support;

use catalog_tool::endpoints::{Case, SkipReason, classify_case};
use endpoints_support::TestResult;
use endpoints_support::report::{REPORT, fixture};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

#[test]
fn cli_check_rejects_same_length_mutation_and_never_rewrites_the_report() -> TestResult {
    // Given canonical bytes written through the real CLI dispatch.
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-endpoints.report");
    let written = cli(dir.path(), "--write")?;
    assert!(written.status.success());
    assert_eq!(written.stdout, REPORT.as_bytes());
    assert_eq!(fs::read(&path)?, REPORT.as_bytes());
    let mut mutation = REPORT.as_bytes().to_vec();
    let first = mutation.first_mut().ok_or("empty fixture report")?;
    *first = b'C';
    fs::write(&path, &mutation)?;
    // When checking a one-byte difference of identical length.
    let checked = cli(dir.path(), "--check")?;
    // Then the complete canonical report is printed, failure is named, and bytes are untouched.
    assert_eq!(checked.status.code(), Some(1));
    assert_eq!(checked.stdout, REPORT.as_bytes());
    assert_eq!(
        String::from_utf8(checked.stderr)?,
        format!("endpoint report differs: {}\n", path.display())
    );
    assert_eq!(fs::read(&path)?, mutation);
    Ok(())
}

fn cli(root: &Path, flag: &str) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "endpoints", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .arg(flag)
        .output()
}

#[test]
fn operation_inputs_take_precedence_over_partition_and_feature_exclusions() -> TestResult {
    let case: Case = serde_json::from_value(json!({
        "params":{"Region":"cn-north-1","UseFIPS":true,"UseDualStack":true,
            "Endpoint":"https://custom.test","Bucket":"b"},
        "operationInputs":[{"operationName":"List"}], "expect":{"error":"unused"}
    }))?;
    assert_eq!(classify_case(&case), Some(SkipReason::OperationInputs));
    Ok(())
}

#[test]
fn any_explicit_endpoint_value_is_a_custom_endpoint_exclusion() -> TestResult {
    for endpoint in [Value::Null, json!(""), json!(false), json!(123), json!([])] {
        let case: Case = serde_json::from_value(json!({
            "params":{"Region":"us-west-2","Endpoint":endpoint,"Bucket":"b"},
            "expect":{"error":"unused"}
        }))?;
        assert_eq!(classify_case(&case), Some(SkipReason::CustomEndpoint));
    }
    Ok(())
}
