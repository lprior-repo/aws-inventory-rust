use super::*;
use crate::overlay::{MemberPath, OperationName, Origin, TokenPair};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn policy() -> Result<ServiceOverlay> {
    let callable = OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Items")?),
        identity: Some(Identity::Arn(MemberPath::new("Arn")?)),
        paginator: Some(Paginator::None),
        ..OperationOverlay::default()
    };
    Ok(ServiceOverlay::new(
        "sample",
        vec![("ListThings", callable)],
    )?)
}

fn deferred(reason: &str) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some(reason.to_owned()),
        ..OperationOverlay::default()
    }
}

#[test]
fn given_complete_policy_when_rendered_twice_then_bytes_and_roundtrip_are_stable() -> Result {
    let policy = policy()?;
    let first = render(&policy)?;
    assert_eq!(first, render(&policy)?);
    let parsed = ServiceOverlay::parse("sample", &first, Origin::Generated)?;
    assert_eq!(parsed.operations(), policy.operations());
    assert!(first.contains("reviewed = \"2026-11-15\""));
    Ok(())
}

#[test]
fn given_quotes_and_backslashes_when_rendered_then_reason_roundtrips() -> Result {
    let policy = ServiceOverlay::new(
        "sample",
        vec![("ListThings", deferred("a \"quote\" and \\path"))],
    )?;
    let parsed = ServiceOverlay::parse("sample", &render(&policy)?, Origin::Generated)?;
    assert_eq!(parsed.operations(), policy.operations());
    Ok(())
}

#[test]
fn given_token_and_key_when_rendered_then_optional_inferred_fields_remain_absent() -> Result {
    let mut policy = policy()?;
    let entry = policy
        .operations
        .get_mut("ListThings")
        .ok_or("missing entry")?;
    entry.items = None;
    entry.identity = Some(Identity::Key(vec![MemberPath::selected("$")?]));
    entry.paginator = Some(Paginator::Token(TokenPair::new(
        MemberPath::new("Start")?,
        MemberPath::new("Next")?,
    )));
    let parsed = ServiceOverlay::parse("sample", &render(&policy)?, Origin::Generated)?;
    assert_eq!(parsed.operations(), policy.operations());
    Ok(())
}

#[test]
fn given_control_character_when_rendered_then_invalid_toml_is_refused() -> Result {
    let mut policy = policy()?;
    let entry = policy
        .operations
        .get_mut("ListThings")
        .ok_or("missing entry")?;
    entry.defer_reason = Some("line\nbreak".to_owned());
    assert!(render(&policy).is_err());
    Ok(())
}

#[test]
fn given_4097_operations_when_rendered_then_operation_limit_is_enforced() -> Result {
    let mut policy = policy()?;
    policy.operations.clear();
    for index in 0..4097 {
        let name = OperationName::new(&format!("List{index}"))?;
        policy.operations.insert(name, deferred("reason"));
    }
    assert!(render(&policy).is_err());
    Ok(())
}

#[test]
fn given_output_exceeding_one_mib_when_rendered_then_no_bytes_are_published() -> Result {
    let mut policy = policy()?;
    policy.operations.clear();
    let reason = "x".repeat(1024);
    for index in 0..4096 {
        let name = OperationName::new(&format!("List{index}"))?;
        policy.operations.insert(name, deferred(&reason));
    }
    assert!(render(&policy).is_err());
    Ok(())
}

#[test]
fn given_unresolved_endpoint_when_generated_then_all_enumeration_is_deferred() -> Result {
    let value = super::super::tests::given_model();
    let model = crate::plans::Model::parse(&value, "sample")?;
    let generated = super::super::service::generate(
        "sample",
        &model,
        None,
        &crate::plans::Vocabulary::default(),
        &crate::classes::DenyList::builtin(),
        None,
    )?;
    assert_eq!(generated.enumerate, 1);
    assert_eq!(generated.callable, 0);
    assert_eq!(generated.reasons.get("endpoint: not resolved"), Some(&1));
    let parsed = ServiceOverlay::parse("sample", &render(&generated.policy)?, Origin::Generated)?;
    assert_eq!(parsed.operations(), generated.policy.operations());
    Ok(())
}

#[test]
fn hand_skip_evidence_roundtrips_without_shape_inference() -> Result {
    let text = r#"format = 1
reviewed = "2026-11-15"
reviewer = "test"
[operation.ListThings]
class = "enumerate"
scope = "region"
items = "Things"
skip = ["DescribeThing"]
[operation.ListThings.skip_evidence.DescribeThing]
service = "sample"
list = "ListThings"
describe = "DescribeThing"
inputs_sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
compared = "ListThings.Things[] == DescribeThing.Thing"
resources = 3
when = "2026-11-24T00:00:00Z"
"#;
    let policy = ServiceOverlay::parse("sample", text, Origin::Hand)?;
    let rendered = render(&policy)?;
    let parsed = ServiceOverlay::parse("sample", &rendered, Origin::Generated)?;
    assert_eq!(policy.operations(), parsed.operations());
    let without_evidence = text
        .split("[operation.ListThings.skip_evidence")
        .next()
        .ok_or("missing text")?;
    assert!(ServiceOverlay::parse("sample", without_evidence, Origin::Hand).is_err());
    Ok(())
}
