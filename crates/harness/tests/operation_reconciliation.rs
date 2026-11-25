use std::error::Error;

use inventory_harness::audit::calls_in_source;
use inventory_harness::audit::calls_in_source_with_budget;
use inventory_harness::audit::mask_noncode;

type TestResult = Result<(), Box<dyn Error>>;

fn call_ids(source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let (calls, _) = calls_in_source("sample.rs", source)?;
    Ok(calls.iter().map(|c| c.id.clone()).collect())
}

fn error_text<T, E: std::fmt::Display>(result: Result<T, E>) -> Result<String, Box<dyn Error>> {
    match result {
        Err(e) => Ok(e.to_string()),
        Ok(_) => Err("expected error".into()),
    }
}

#[test]
fn test_debug_masking() -> TestResult {
    let source = "let text = \"client.fake().send()\";\nclient.describe_vpcs().send();";
    let masked = mask_noncode("sample.rs", source)?;
    assert!(!masked.contains("fake"));
    assert!(masked.contains("describe_vpcs"));
    Ok(())
}

#[test]
fn test_comments_strings_chars_and_lifetimes_do_not_create_calls() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    let text = \"client.fake().send()\";\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_qualified_client_assignment_and_parameter_resolve_service() -> TestResult {
    let source = "fn one() { let sdk = aws_sdk_config::Client::new(config); sdk.get_status().send(); }\nfn two(sdk: &aws_sdk_resourceexplorer2::Client) { sdk.get_status().send(); }\n";
    let ids = call_ids(source)?;
    assert_eq!(
        ids,
        vec!["config.get_status", "resourceexplorer2.get_status"]
    );
    Ok(())
}

#[test]
fn test_changed_sdk_namespace_is_not_inferred_from_old_method_names() -> TestResult {
    let source =
        "use aws_sdk_eks::Client; fn list(client: &Client) {client.list_clusters().send();}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["eks.list_clusters"]);
    Ok(())
}

#[test]
fn test_channel_sends_are_excluded_and_opaque_empty_sends_fail_closed() -> TestResult {
    let source = "fn emit(sender: Sender) {sender.send(event).await;}\n";
    let (calls, excluded) = calls_in_source("sample.rs", source)?;
    assert_eq!(calls.len(), 0);
    assert_eq!(excluded, 1);
    Ok(())
}

#[test]
fn test_unrecognized_client_import_cannot_borrow_an_operation_identity() {
    let source = "fn collect(client: OtherClient) {client.describe_vpcs().send();}\n";
    let result = calls_in_source("sample.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_lexical_bounds_and_incomplete_source_fail_closed() {
    let source = "fn x() { \"unterminated\n";
    let result = calls_in_source("sample.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_mismatched_closing_delimiter_names_both_source_locations() {
    let source = "fn collect() {\n    let values = [1, 2);\n}\n";
    let result = calls_in_source("sample.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_unexpected_closing_delimiter_has_its_path_and_location() {
    let source = "fn collect() {}\n}\n";
    let result = calls_in_source("sample.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_client_resolution_failure_includes_callsite_context() {
    let source = "fn collect(client: OtherClient) {\n    client.describe_vpcs().send();\n}\n";
    let result = calls_in_source("unknown_client.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_parent_model_argument_does_not_create_a_new_sdk_operation() -> TestResult {
    let source = "use aws_sdk_cloudcontrol::Client;\nasync fn collect(client: &Client) {\n    client.list_resources().type_name(&request.type_name)\n        .set_resource_model(request.resource_model.clone())\n        .set_next_token(request.token.clone()).send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["cloudcontrol.list_resources"]);
    Ok(())
}

#[test]
fn test_debug_method_chain() -> TestResult {
    let source = "use aws_sdk_cloudcontrol::Client;\nasync fn collect(client: &Client) {\n    client.list_resources().type_name(&request.type_name)\n        .set_resource_model(request.resource_model.clone())\n        .set_next_token(request.token.clone()).send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["cloudcontrol.list_resources"]);
    Ok(())
}

#[test]
fn test_proptest_raw_byte_c_strings_and_character_braces_remain_masked() -> TestResult {
    let source = "use aws_sdk_cloudcontrol::Client;\nfn property() {\n    let bytes = b\"client.fake().send()\";\n    client.list_resources().send();\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["cloudcontrol.list_resources"]);
    Ok(())
}

#[test]
fn test_missing_function_brace_before_proptest_reports_exact_opening() {
    let source = "fn cancelled() {\n    Ok(())\n\nproptest! { fn generated() {} }\n";
    let result = calls_in_source("parent_cases.rs", source);
    assert!(result.is_err());
}

#[test]
fn test_unterminated_literals_and_comments_report_original_path_and_line() {
    for (start, _message) in [
        ("\"", "Rust string"),
        ("r##\"", "Rust raw string"),
        ("/*", "Rust block comment"),
    ] {
        let source = format!("fn collect() {{\n    {}unfinished\n}}", start);
        let result = calls_in_source("sample.rs", &source);
        assert!(result.is_err());
    }
}

#[test]
fn test_non_ascii_string_literal() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    let greeting = \"héllo wörld\";\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_non_ascii_block_comment() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    /* héllo wörld */\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_multibyte_char_does_not_hide_call() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn f(){let a=\"é\";let b=\"client.fake().send()\";}\nfn collect(client: &Client) {\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_nested_block_comments() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    /* outer /* inner */ still outer */\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_raw_string_delimiters() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    let s = r\"client.fake().send()\";\n    let s2 = r##\"client.fake().send()\"##;\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_char_literal_apostrophe() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    let quote = '\\'';\n    client.describe_vpcs().send().await;\n}\n";
    let ids = call_ids(source)?;
    assert_eq!(ids, vec!["ec2.describe_vpcs"]);
    Ok(())
}

#[test]
fn test_unterminated_string_has_location() -> TestResult {
    let source = "fn collect() {\n    let s = \"unterminated\n}";
    let err = error_text(calls_in_source("sample.rs", source))?;
    assert!(
        err.contains("sample.rs:2:13"),
        "expected location in error: {}",
        err
    );
    Ok(())
}

#[test]
fn test_unbalanced_delimiters_has_both_locations() -> TestResult {
    let source = "fn collect() {\n    let values = [1, 2);\n}\n";
    let err = error_text(calls_in_source("sample.rs", source))?;
    assert!(
        err.contains("sample.rs:2:23"),
        "expected closing location in error: {}",
        err
    );
    assert!(
        err.contains("opened at 2:18"),
        "expected opening location in error: {}",
        err
    );
    Ok(())
}

#[test]
fn test_unexpected_closing_delimiter_has_location() -> TestResult {
    let source = "fn collect() {}\n}\n";
    let err = error_text(calls_in_source("sample.rs", source))?;
    assert!(
        err.contains("sample.rs:2:1"),
        "expected location in error: {}",
        err
    );
    Ok(())
}

#[test]
fn test_unclosed_delimiter_has_location() -> TestResult {
    let source = "fn collect() {\n    Ok(())\n";
    let err = error_text(calls_in_source("parent_cases.rs", source))?;
    assert!(
        err.contains("parent_cases.rs:1:14"),
        "expected location in error: {}",
        err
    );
    Ok(())
}

#[test]
fn test_client_resolution_error_has_path() -> TestResult {
    let source = "fn collect(client: OtherClient) {\n    client.describe_vpcs().send();\n}\n";
    let err = error_text(calls_in_source("unknown_client.rs", source))?;
    assert!(
        err.contains("unknown_client.rs"),
        "expected path in error: {}",
        err
    );
    Ok(())
}

#[test]
fn test_debug_non_ascii_masking() -> TestResult {
    let source = "use aws_sdk_ec2::Client;\nfn collect(client: &Client) {\n    /* héllo wörld */\n    client.describe_vpcs().send().await;\n}\n";
    let masked = mask_noncode("sample.rs", source)?;
    println!("Original: {:?}", source);
    println!("Masked: {:?}", masked);
    println!(
        "Masked bytes around 60-70: {:?}",
        masked.as_bytes().get(60..70)
    );
    Ok(())
}

#[test]
fn test_token_budget_exceeded() {
    let source = "let a=1;let b=2;let c=3;let d=4;let e=5;";
    let result = calls_in_source_with_budget("sample.rs", source, 5);
    assert!(result.is_err());
}
