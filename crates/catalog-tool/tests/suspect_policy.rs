use catalog_tool::overlay::{self, FixedValue, MemberPath, OverlayClass};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_reviewed_suspect_claims_when_loading_policy_then_safety_decisions_are_explicit() -> TestResult {
    let services = overlay::embedded()?;
    include_str!("suspect_policy.tsv")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .try_for_each(|line| check_line(&services, line))
}

fn check_line(services: &[overlay::ServiceOverlay], line: &str) -> TestResult {
    let mut fields = line.split('\t');
    let columns: [_; 6] = std::array::from_fn(|_| fields.next());
    let [Some(service), Some(operation), Some(decision), Some(member), Some(value), None] = columns else {
        return Err(std::io::Error::other("invalid suspect policy contract").into());
    };
    let policy = services.iter().find(|entry| entry.service() == service)
        .and_then(|entry| entry.operations().get(operation))
        .ok_or_else(|| std::io::Error::other(format!("missing {service}/{operation}")))?;
    check_decision(policy, decision, member, value)?;
    assert!(policy.class.is_some(), "{service}/{operation}");
    Ok(())
}

fn check_decision(
    policy: &overlay::OperationOverlay, decision: &str, member: &str, value: &str,
) -> TestResult {
    match decision {
        "string" => check_fixed(policy, member, FixedValue::String(value.to_owned()))?,
        "boolean" => check_fixed(policy, member, FixedValue::Boolean(value.parse()?))?,
        "page_size" => assert_eq!(policy.page_size, Some(value.parse()?)),
        "tags" => check_tags(policy, member)?,
        _ => assert_eq!(policy.class, Some(expected_class(decision)?)),
    }
    Ok(())
}

fn check_fixed(policy: &overlay::OperationOverlay, member: &str, value: FixedValue) -> TestResult {
    assert_eq!(policy.fixed.get(&MemberPath::new(member)?), Some(&value));
    Ok(())
}

fn check_tags(policy: &overlay::OperationOverlay, member: &str) -> TestResult {
    let expected = if member == "-" { None } else { Some(MemberPath::new(member)?) };
    assert_eq!(policy.tags, expected);
    Ok(())
}

fn expected_class(decision: &str) -> Result<OverlayClass, std::io::Error> {
    use catalog_tool::classes::DenyReason;
    match decision {
        "defer" => Ok(OverlayClass::Defer),
        "secret" => Ok(OverlayClass::Deny(DenyReason::Secret)),
        "content" => Ok(OverlayClass::Deny(DenyReason::Content)),
        "reference" => Ok(OverlayClass::Deny(DenyReason::ReferenceData)),
        _ => Err(std::io::Error::other(format!("unknown contract {decision}"))),
    }
}
