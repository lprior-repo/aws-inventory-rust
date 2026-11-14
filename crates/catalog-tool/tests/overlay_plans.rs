use catalog_tool::overlay::Rule;
pub mod overlay_support;
use overlay_support::{TestResult, name, parse, reject};

#[test]
fn identity_requires_exactly_one_strategy_and_valid_placeholders() {
    // Given ambiguous or malformed identities, when parsed, then no strategy is inferred.
    [
        "{}",
        "{arn='A',key=['B']}",
        "{arn='A',template='{Id}'}",
        "{template='{Id}',key=['B']}",
        "{arn='A',template='{Id}',key=['B']}",
    ]
    .into_iter()
    .for_each(|s| {
        reject(
            &format!("identity={s}"),
            "identity",
            "arn/template/key",
            Rule::Identity,
        )
    });
    [
        "plain", "{}", "{Id", "Id}", "{{Id}}", "{A..B}", "{ Id}", "{é}",
    ]
    .into_iter()
    .for_each(|s| {
        reject(
            &format!("identity={{template='{s}'}}"),
            "identity.template",
            s,
            Rule::Template,
        )
    });
    reject(
        "identity={key=[]}",
        "identity.key",
        "0",
        Rule::Count { min: 1, max: 8 },
    );
    reject(
        "identity={key=['Id','Id']}",
        "identity.key",
        "Id",
        Rule::Duplicate,
    );
}
#[test]
fn paginator_requires_kind_specific_keys_and_distinct_inputs() {
    // Given mixed paginator shapes, when parsed, then only selected-kind fields are legal.
    reject("paginator={}", "paginator.kind", "", Rule::Missing);
    reject(
        "paginator={kind='other'}",
        "paginator.kind",
        "other",
        Rule::Value,
    );
    [
        ("none", "input='A'", "input", "A"),
        ("none", "output='B'", "output", "B"),
        ("none", "pairs=[]", "pairs", "array"),
        ("none", "truncated='T'", "truncated", "T"),
        ("token", "pairs=[]", "pairs", "array"),
        ("token", "truncated='T'", "truncated", "T"),
        ("tokens", "input='A'", "input", "A"),
        ("tokens", "output='B'", "output", "B"),
    ]
    .into_iter()
    .for_each(|(kind, field, key, value)| {
        reject(
            &format!("paginator={{kind='{kind}',{field}}}"),
            &format!("paginator.{key}"),
            value,
            Rule::PaginatorKey,
        )
    });
    reject(
        "paginator={kind='token',output='B'}",
        "paginator.input",
        "",
        Rule::Missing,
    );
    reject(
        "paginator={kind='token',input='A'}",
        "paginator.output",
        "",
        Rule::Missing,
    );
    reject(
        "paginator={kind='tokens'}",
        "paginator.pairs",
        "",
        Rule::Missing,
    );
    reject(
        "paginator={kind='tokens',pairs=[]}",
        "paginator.pairs",
        "0",
        Rule::Count { min: 1, max: 8 },
    );
    reject(
        "paginator={kind='tokens',pairs=[{input='A',output='B'},{input='A',output='C'}]}",
        "paginator.pairs.input",
        "A",
        Rule::DuplicateInput,
    );
    reject(
        "paginator={kind='tokens',pairs=[{input='A'}]}",
        "paginator.pairs.output",
        "",
        Rule::Missing,
    );
}
#[test]
fn bindings_require_one_complete_source_and_true_context_flags() {
    // Given partial or conflicting sources, when parsed, then the exact binding is rejected.
    [
        "{}",
        "{from='ListParents'}",
        "{path='Id'}",
        "{from='ListParents',path='Id',account=true}",
        "{account=true,region=true}",
        "{path='Id',region=true}",
    ]
    .into_iter()
    .for_each(|s| {
        reject(
            &format!("bind={{Id={s}}}"),
            "bind.Id",
            "from+path/account/region",
            Rule::Binding,
        )
    });
    reject(
        "bind={Id={account=false}}",
        "bind.Id.account",
        "false",
        Rule::Binding,
    );
    reject(
        "bind={Id={region=false}}",
        "bind.Id.region",
        "false",
        Rule::Binding,
    );
    reject(
        "bind={Id={from='ListThings',path='Id'}}",
        "bind.Id.from",
        "ListThings",
        Rule::SelfReference,
    );
    reject(
        "skip=['ListThings']",
        "skip",
        "ListThings",
        Rule::SelfReference,
    );
    reject(
        "batch={member='Ids',max=1,replaces='ListThings'}",
        "batch.replaces",
        "ListThings",
        Rule::SelfReference,
    );
}
#[test]
fn bind_cycles_of_two_and_three_are_rejected_but_external_parents_are_allowed() -> TestResult {
    // Given cyclic and acyclic graphs, when parsed, then only in-file cycles fail.
    [ ("[operation.A.bind]\nId={from='B',path='Id'}\n[operation.B.bind]\nId={from='A',path='Id'}", "A"),
        ("[operation.A.bind]\nId={from='B',path='Id'}\n[operation.B.bind]\nId={from='C',path='Id'}\n[operation.C.bind]\nId={from='A',path='Id'}", "A") ]
        .into_iter().for_each(|(body, op)| overlay_support::error(&format!("{}{body}", overlay_support::HEADER),
            Some(op), "bind", op, Rule::BindCycle));
    let o = parse(
        "[operation.A.bind]\nId={from='B',path='Id'}\n[operation.B.bind]\nId={from='External',path='Id'}",
    )?;
    assert_eq!(o.operations().len(), 2);
    assert_eq!(
        o.operations().get(&name("A")?).map(|op| op.bind.len()),
        Some(1)
    );
    Ok(())
}
#[test]
fn fixed_and_bind_map_keys_are_member_paths() {
    // Given invalid input paths, when parsed, then map keys receive typed errors.
    reject("fixed={'A..B'=1}", "fixed.A..B", "A..B", Rule::MemberPath);
    reject(
        "bind={'A[]'={account=true}}",
        "bind.A[]",
        "A[]",
        Rule::MemberPath,
    );
    reject(
        "fixed={Id=[1]}",
        "fixed.Id",
        "1",
        Rule::WrongType { expected: "string" },
    );
}
