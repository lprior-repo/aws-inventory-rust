use catalog_tool::overlay::{OverlayError, Rule, ServiceOverlay};
pub mod overlay_support;
use overlay_support::{HEADER, TestResult, error, reject};

#[test]
fn format_date_reviewer_and_service_rules_reject_exact_values() -> TestResult {
    // Given invalid metadata, when parsed, then each specific reason survives.
    [0, 2, -1].into_iter().for_each(|n| {
        error(
            &format!("format={n}"),
            None,
            "format",
            &n.to_string(),
            Rule::Format,
        )
    });
    [
        "2026-13-01",
        "2026-02-30",
        "1900-02-29",
        "0000-01-01",
        "2026-00-01",
        "2026-01-00",
        "2026-1-01",
        "+026-01-01",
    ]
    .into_iter()
    .for_each(|s| {
        error(
            &format!("format=1\nreviewed='{s}'"),
            None,
            "reviewed",
            s,
            Rule::Date,
        )
    });
    ["".to_owned(), "x".repeat(65), "é".repeat(33)]
        .into_iter()
        .for_each(|s| {
            error(
                &format!("format=1\nreviewed='2026-01-01'\nreviewer='{s}'"),
                None,
                "reviewer",
                &s,
                Rule::Reviewer,
            )
        });
    ["", "Ec2", "-ec2", "ec_2", "ec.2", "é"]
        .into_iter()
        .try_for_each(|s| -> TestResult {
            match ServiceOverlay::parse(s, HEADER) {
                Err(OverlayError::Rule {
                    service,
                    operation,
                    key,
                    value,
                    rule,
                }) => assert_eq!(
                    (service, operation, key, value, rule),
                    (s.into(), None, "service".into(), s.into(), Rule::Service)
                ),
                _ => return Err("expected service rule".into()),
            }
            Ok(())
        })
}
#[test]
fn valid_metadata_and_calendar_boundaries_parse() -> TestResult {
    // Given boundary dates and byte lengths, when parsed, then spellings are preserved.
    [
        "0001-01-01",
        "9999-12-31",
        "2000-02-29",
        "2024-02-29",
        "2026-04-30",
    ]
    .into_iter()
    .try_for_each(|date| -> TestResult {
        let s = format!("format=1\nreviewed='{date}'\nreviewer='{}'", "é".repeat(32));
        let o = ServiceOverlay::parse("0-storage-gateway", &s)?;
        assert_eq!(
            (o.reviewed(), o.reviewer().len(), o.operations().len()),
            (date, 64, 0)
        );
        Ok(())
    })
}
#[test]
fn deny_and_defer_have_no_plans_and_reason_is_exclusive() {
    // Given conflicting classifications, when parsed, then callable plans are forbidden.
    reject("class='deny'", "deny_reason", "", Rule::DenyReason);
    [
        "",
        "class='enumerate'\n",
        "class='expand'\n",
        "class='defer'\n",
    ]
    .into_iter()
    .for_each(|s| {
        reject(
            &format!("{s}deny_reason='content'"),
            "deny_reason",
            "content",
            Rule::DenyReason,
        )
    });
    reject(
        "class='deny'\ndeny_reason='unknown'",
        "deny_reason",
        "unknown",
        Rule::Value,
    );
    ["deny", "defer"].into_iter().for_each(|class| {
        [
            "scope='region'",
            "items='A'",
            "page_size=1",
            "exclude_from_hash=[]",
            "skip=[]",
            "quota_group='a'",
            "queryable=[]",
            "tags='A'",
            "fixed={}",
            "paginator={}",
            "identity={}",
            "bind={}",
            "batch={}",
        ]
        .into_iter()
        .for_each(|plan| no_plan(class, plan));
    });
}
fn no_plan(class: &str, plan: &str) {
    let reason = if class == "deny" {
        "deny_reason='content'\n"
    } else {
        ""
    };
    let split = plan.split_once('=');
    assert!(split.is_some(), "fixture `{plan}` has no `=`");
    let Some((key, value)) = split else { return };
    let value = match value {
        "[]" => "array",
        "{}" => "table",
        v => v.trim_matches('\''),
    };
    reject(
        &format!("class='{class}'\n{reason}{plan}"),
        key,
        value,
        Rule::NoPlan,
    );
}
#[test]
fn enum_quota_and_identifier_spellings_are_exact() {
    // Given names outside each grammar, when parsed, then values are not normalized.
    [
        ("class='Enumerate'", "class", "Enumerate"),
        ("scope='global'", "scope", "global"),
        ("quota_group='A'", "quota_group", "A"),
        ("quota_group='a_'", "quota_group", "a_"),
        ("quota_group=''", "quota_group", ""),
    ]
    .into_iter()
    .for_each(|(b, k, v)| reject(b, k, v, Rule::Value));
    ["", "A..B", "_A", "1A", "A[B]", "A.é"]
        .into_iter()
        .for_each(|s| reject(&format!("items='{s}'"), "items", s, Rule::MemberPath));
    ["_List", "List_A", "1List", "é"].into_iter().for_each(|s| {
        error(
            &format!("{HEADER}[operation.'{s}']"),
            Some(s),
            "operation",
            s,
            Rule::OperationName,
        )
    });
    reject("skip=['List_A']", "skip", "List_A", Rule::OperationName);
}
