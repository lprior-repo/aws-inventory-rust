pub mod endpoints_support;
use endpoints_support::{TestResult, assert_value, call};
use serde_json::json;

#[test]
fn boolean_string_negation_and_set_functions_observe_exact_types() -> TestResult {
    // Given independent hand-written expressions, when evaluated, then exact predicates match.
    [
        (call("booleanEquals", json!([true, false])), json!(false)),
        (call("booleanEquals", json!([false, false])), json!(true)),
        (call("stringEquals", json!(["AWS", "aws"])), json!(false)),
        (call("stringEquals", json!(["", ""])), json!(true)),
        (call("not", json!([false])), json!(true)),
        (call("not", json!([true])), json!(false)),
        (call("isSet", json!([{"ref":"Absent"}])), json!(false)),
        (call("isSet", json!([false])), json!(true)),
        (call("isSet", json!([""])), json!(true)),
    ]
    .into_iter()
    .try_for_each(|(expr, expected)| assert_value(expr, expected))
}

#[test]
fn host_labels_check_each_domain_segment_and_hyphen_boundaries() -> TestResult {
    // Given labels with boundary violations, when checked, then only RFC1123 labels match.
    [
        ("a", false, true),
        ("A-b9", false, true),
        ("a.b", false, false),
        ("a.b", true, true),
        ("a..b", true, false),
        ("-a", false, false),
        ("a-", false, false),
        ("a_b", false, false),
        ("", true, false),
        ("é", false, false),
    ]
    .into_iter()
    .try_for_each(|(s, sub, expected)| {
        assert_value(call("isValidHostLabel", json!([s, sub])), json!(expected))
    })?;
    assert_value(
        call("isValidHostLabel", json!(["a".repeat(63), false])),
        json!(true),
    )?;
    assert_value(
        call("isValidHostLabel", json!(["a".repeat(64), false])),
        json!(false),
    )
}
