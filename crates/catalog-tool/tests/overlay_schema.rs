use catalog_tool::overlay::{OverlayError, Rule, ServiceOverlay};
pub mod overlay_support;
use overlay_support::{HEADER, TestResult, error, reject};

#[test]
fn unknown_keys_are_contextual_at_every_structured_table() {
    // Given an unknown key at each depth, when parsed, then its full location survives.
    error(
        &format!("{HEADER}extra='bad'"),
        None,
        "extra",
        "bad",
        Rule::UnknownKey,
    );
    [
        ("extra='bad'", "extra"),
        ("identity={extra='bad'}", "identity.extra"),
        ("paginator={extra='bad'}", "paginator.extra"),
        (
            "paginator={kind='tokens',pairs=[{input='A',output='B',extra='bad'}]}",
            "paginator.pairs.extra",
        ),
        ("bind={Id={extra='bad'}}", "bind.Id.extra"),
        ("batch={extra='bad'}", "batch.extra"),
    ]
    .into_iter()
    .for_each(|(body, key)| reject(body, key, "bad", Rule::UnknownKey));
}
#[test]
fn wrong_types_and_missing_root_keys_are_typed() {
    // Given incompatible primitive/container types, when parsed, then key and value are exact.
    [
        ("items=1", "items", "1", "string"),
        ("page_size=true", "page_size", "true", "integer"),
        ("skip=1", "skip", "1", "array"),
        ("bind=1", "bind", "1", "table"),
        ("fixed=1", "fixed", "1", "table"),
        ("identity=1", "identity", "1", "table"),
        ("paginator=1", "paginator", "1", "table"),
        ("batch=1", "batch", "1", "table"),
        ("bind={Id={account=1}}", "bind.Id.account", "1", "boolean"),
        (
            "fixed={Id=1.5}",
            "fixed.Id",
            "1.5",
            "string, integer, boolean or string array",
        ),
    ]
    .into_iter()
    .for_each(|(b, k, v, expected)| reject(b, k, v, Rule::WrongType { expected }));
    error(
        "reviewed='2026-11-13'\nreviewer='x'",
        None,
        "format",
        "",
        Rule::Missing,
    );
    error(
        "format=1\nreviewer='x'",
        None,
        "reviewed",
        "",
        Rule::Missing,
    );
    error(
        "format=1\nreviewed='2026-11-13'",
        None,
        "reviewer",
        "",
        Rule::Missing,
    );
    error(
        "format='1'",
        None,
        "format",
        "1",
        Rule::WrongType {
            expected: "integer",
        },
    );
    error(
        &format!("{HEADER}operation=1"),
        None,
        "operation",
        "1",
        Rule::WrongType { expected: "table" },
    );
}
#[test]
fn syntax_errors_keep_original_line_column_and_span() -> TestResult {
    // Given invalid TOML on line four, when parsed, then the library error stays intact.
    let text = format!("{HEADER}bad = @\n");
    match ServiceOverlay::parse("ec2", &text) {
        Err(OverlayError::Syntax { service, source }) => {
            assert_eq!(service, "ec2");
            assert!(source.to_string().contains("line 4, column 7"));
            assert_eq!(
                source.span().map(|s| s.start),
                Some(HEADER.len().saturating_add(6))
            );
        }
        _ => return Err("expected syntax error".into()),
    }
    Ok(())
}

#[test]
fn operation_values_and_token_pair_values_must_be_tables() {
    // Given scalar operation/pair values, when parsed, then the enclosing operation survives.
    error(
        &format!("{HEADER}[operation]\nListThings=1"),
        Some("ListThings"),
        "",
        "1",
        Rule::WrongType { expected: "table" },
    );
    reject(
        "paginator={kind='tokens',pairs=[1]}",
        "paginator.pairs",
        "1",
        Rule::WrongType { expected: "table" },
    );
}
