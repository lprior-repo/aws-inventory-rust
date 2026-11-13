//! Audit contracts: lexical callsites, boundary controls and evidence binding.
use inventory_harness::HarnessError;
use inventory_harness::audit::{
    CallSite, calls_in_source, historical_operations, source_inventory, verify_callsites,
};
use serde_json::{Value, json};
type BoxError = Box<dyn std::error::Error>;
type TestResult = Result<(), BoxError>;
fn failure<T>(result: Result<T, HarnessError>) -> Result<String, BoxError> {
    match result {
        Err(error) => Ok(error.to_string()),
        Ok(_) => Err(String::from("expected failure").into()),
    }
}
fn contain(text: &str, needle: &str) -> TestResult {
    assert!(text.contains(needle), "missing {needle:?} in {text:?}");
    Ok(())
}
fn ids(source: &str) -> Result<Vec<String>, BoxError> {
    let (calls, _) = calls_in_source("sample.rs", source)?;
    Ok(calls.iter().map(|call| call.id.clone()).collect())
}
#[test]
fn scanner_masks_literals_resolves_receivers_and_excludes_channel_sends() -> TestResult {
    let source = r######"
use aws_sdk_ec2::Client;
// client.fake().send()
/* outer /* nested client.fake().send() */ comment */
fn collect<'a>(client: &'a Client) {
    let text = "client.fake().send()";
    let raw = r###"client.fake().send() /* not a comment */"###;
    let quote = '\'';
    client.describe_vpcs().set_next_token(Some(format!("{}", text))).send().await;
}
"######;
    let (calls, excluded) = calls_in_source("sample.rs", source)?;
    assert_eq!(excluded, 0);
    let call = calls.first().ok_or_else(|| String::from("one call"))?;
    assert_eq!(
        (call.id.as_str(), call.function.as_str(), call.send_line),
        ("ec2.describe_vpcs", "collect", 9)
    );
    assert_eq!(
        ids(
            "\nfn one() { let sdk = aws_sdk_config::Client::new(config); sdk.get_status().send(); }\nfn two(sdk: &aws_sdk_resourceexplorer2::Client) { sdk.get_status().send(); }\n"
        )?,
        ["config.get_status", "resourceexplorer2.get_status"]
    );
    assert_eq!(
        ids("use aws_sdk_eks::Client; fn list(client: &Client) {client.list_clusters().send();}")?,
        ["eks.list_clusters"]
    );
    let (calls, excluded) = calls_in_source(
        "sample.rs",
        "fn emit(sender: Sender) {sender.send(event).await;}",
    )?;
    assert!(calls.is_empty() && excluded == 1);
    Ok(())
}
#[test]
fn scanner_fails_closed_with_exact_locations() -> TestResult {
    contain(
        &failure(calls_in_source(
            "sample.rs",
            "fn collect() {builder.send().await;}",
        ))?,
        "unrecognized empty send",
    )?;
    for (source, expected) in [
        (
            "fn collect() {\n    \"unfinished\n}",
            "sample.rs:2:5: unterminated Rust string",
        ),
        (
            "fn collect() {\n    r##\"unfinished\n}",
            "sample.rs:2:5: unterminated Rust raw string",
        ),
        (
            "fn collect() {\n    /* unfinished\n}",
            "sample.rs:2:5: unterminated Rust block comment",
        ),
        (
            "fn collect() {}\n}\n",
            "sample.rs:2:1: unexpected closing Rust delimiter '}'",
        ),
    ] {
        contain(&failure(calls_in_source("sample.rs", source))?, expected)?;
    }
    assert_eq!(
        failure(calls_in_source(
            "sample.rs",
            "fn collect() {\n    let values = [1, 2);\n}\n"
        ))?,
        "sample.rs:2:23: unbalanced Rust delimiters: ')' closes '[' opened at 2:18; expected ']'"
    );
    assert_eq!(
        failure(calls_in_source(
            "parent_cases.rs",
            "\nfn cancelled() {\n    Ok(())\n\nproptest! { fn generated() {} }\n"
        ))?,
        "parent_cases.rs:2:16: unclosed Rust delimiter '{'; expected '}' before end of file"
    );
    let error = failure(calls_in_source(
        "unknown_client.rs",
        "\nfn collect(client: OtherClient) {\n    client.describe_vpcs().send();\n}",
    ))?;
    contain(&error, "unknown_client.rs:3:")?;
    contain(&error, "cannot uniquely resolve")
}
#[test]
fn callsite_boundaries_operation_ids_and_source_inventory_are_exact() -> TestResult {
    let directory = tempfile::tempdir()?;
    let sources = directory.path().join("crates/aws/src");
    std::fs::create_dir_all(&sources)?;
    std::fs::write(
        sources.join("lib.rs"),
        "use aws_sdk_ec2::Client; fn run() {client.describe_vpcs().send();}",
    )?;
    std::fs::write(
        sources.join("tests.rs"),
        "use aws_sdk_ec2::Client; #[test] fn e() {client.describe_subnets().send();}",
    )?;
    let (files, calls, excluded) = source_inventory(directory.path())?;
    assert_eq!((files.len(), calls.len(), excluded), (2, 2, 0));
    let source = "use aws_sdk_ec2::Client; fn collect(client: &Client) { client.describe_vpcs().send(); client.describe_subnets().send(); }";
    let (calls, _) = calls_in_source("a.rs", source)?;
    let expected: Vec<Value> = calls.iter().map(CallSite::to_value).collect();
    let (one, _) = calls_in_source(
        "a.rs",
        "use aws_sdk_ec2::Client; fn collect(client: &Client) { client.describe_vpcs().send(); }",
    )?;
    assert_eq!(
        failure(verify_callsites(&expected, &one))?,
        "SDK operation boundary changes require review: missing={('a.rs', 'collect', 'ec2.describe_subnets'): 1} added={}"
    );
    let expected_one: Vec<Value> = one.iter().map(CallSite::to_value).collect();
    contain(
        &failure(verify_callsites(&expected_one, &calls))?,
        "added={('a.rs', 'collect', 'ec2.describe_subnets'): 1}",
    )?;
    assert!(verify_callsites(&expected, &calls).is_ok());
    let operation = json!({"id": "example.list", "sdk_crate": "aws-sdk-cloudcontrol",
                           "sdk_version": "1.122.0", "api_operation": "ListResources", "source_calls": []});
    assert_eq!(
        failure(historical_operations(
            &json!({"operations": [operation, operation]})
        ))?,
        "duplicate or excessive historical operation IDs"
    );
    Ok(())
}
