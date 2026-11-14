use catalog_tool::overlay::{Binding, Rule};
pub mod overlay_support;
use overlay_support::{TestResult, name, parse, reject};

#[test]
fn batch_and_paginator_pairs_report_each_missing_required_key() {
    reject(
        "batch={max=1,replaces='Other'}",
        "batch.member",
        "",
        Rule::Missing,
    );
    reject(
        "batch={member='Ids',replaces='Other'}",
        "batch.max",
        "",
        Rule::Missing,
    );
    reject(
        "batch={member='Ids',max=1}",
        "batch.replaces",
        "",
        Rule::Missing,
    );
    reject(
        "paginator={kind='tokens',pairs=[{output='B'}]}",
        "paginator.pairs.input",
        "",
        Rule::Missing,
    );
    reject(
        "paginator={kind='tokens',pairs=[{}]}",
        "paginator.pairs.input",
        "",
        Rule::Missing,
    );
}
#[test]
fn root_and_scalar_types_do_not_coerce_values() {
    [
        ("class=1", "class"),
        ("scope=1", "scope"),
        ("quota_group=1", "quota_group"),
        ("tags=1", "tags"),
        ("identity={arn=1}", "identity.arn"),
        ("paginator={kind=1}", "paginator.kind"),
        ("bind={Id={from=1,path='Id'}}", "bind.Id.from"),
    ]
    .into_iter()
    .for_each(|(body, key)| reject(body, key, "1", Rule::WrongType { expected: "string" }));
    overlay_support::error(
        "format=1\nreviewed=1",
        None,
        "reviewed",
        "1",
        Rule::WrongType { expected: "string" },
    );
    overlay_support::error(
        "format=1\nreviewed='2026-01-01'\nreviewer=1",
        None,
        "reviewer",
        "1",
        Rule::WrongType { expected: "string" },
    );
    reject(
        "class='deny'\ndeny_reason=1",
        "deny_reason",
        "1",
        Rule::WrongType { expected: "string" },
    );
}
#[test]
fn branching_dependencies_are_acyclic_only_when_every_parent_can_finish() -> TestResult {
    let body = "[operation.A.bind]\nId={from='B',path='Id'}\nOther={from='C',path='Id'}\nAccount={account=true}\n[operation.B]\n[operation.C.bind]\nId={from='External',path='Id'}";
    let o = parse(body)?;
    assert_eq!(o.operations().len(), 3);
    let op = o.operations().get(&name("A")?).ok_or("missing branch")?;
    assert_eq!(op.bind.len(), 3);
    assert_eq!(
        op.bind
            .values()
            .filter(|b| matches!(b, Binding::From { .. }))
            .count(),
        2
    );
    let cyclic = format!("{body}\n[operation.External.bind]\nId={{from='A',path='Id'}}");
    overlay_support::error(
        &format!("{}{cyclic}", overlay_support::HEADER),
        Some("A"),
        "bind",
        "A",
        Rule::BindCycle,
    );
    Ok(())
}
#[test]
fn deeply_bound_operation_chain_is_iterative_at_the_operation_bound() -> TestResult {
    let body = (0..catalog_tool::overlay::MAX_OPERATIONS)
        .map(|i| {
            let parent = i.saturating_add(1);
            format!("[operation.A{i}.bind]\nId={{from='A{parent}',path='Id'}}\n")
        })
        .collect::<String>();
    let o = parse(&body)?;
    assert_eq!(o.operations().len(), catalog_tool::overlay::MAX_OPERATIONS);
    assert_eq!(
        o.operations()
            .values()
            .map(|op| op.bind.len())
            .sum::<usize>(),
        catalog_tool::overlay::MAX_OPERATIONS
    );
    Ok(())
}
