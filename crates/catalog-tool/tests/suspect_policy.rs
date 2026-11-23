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
        "binding" => check_binding(policy, member, value)?,
        "structure" => check_structure(policy, member, value)?,
        "correlated" => check_correlated(policy, member, value)?,
        "unset" => assert!(!policy.fixed.contains_key(&MemberPath::new(member)?)),
        "scope" => assert_eq!(policy.scope, Some(if value == "account" { overlay::Scope::Account } else { overlay::Scope::Region })),
        "account" => assert_eq!(policy.bind.get(&MemberPath::new(member)?), Some(&overlay::Binding::Account)),
        "filter" => check_filter(policy, member, value)?,
        "branches" => check_branches(policy, member, value)?,
        "features" => check_features(policy, member, value)?,
        _ => check_class(policy, expected_class(decision)?),
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
        "enumerate" => Ok(OverlayClass::Enumerate),
        "expand" => Ok(OverlayClass::Expand),
        "defer" => Ok(OverlayClass::Defer),
        "secret" => Ok(OverlayClass::Deny(DenyReason::Secret)),
        "content" => Ok(OverlayClass::Deny(DenyReason::Content)),
        "reference" => Ok(OverlayClass::Deny(DenyReason::ReferenceData)),
        _ => Err(std::io::Error::other(format!("unknown contract {decision}"))),
    }
}

fn check_binding(policy: &overlay::OperationOverlay, member: &str, value: &str) -> TestResult {
    let (parent, source) = value.split_once(':').ok_or("invalid binding contract")?;
    let Some(overlay::Binding::From { operation, path }) = policy.bind.get(&MemberPath::new(member)?) else {
        return Err("missing parent binding".into());
    };
    assert_eq!(operation.as_str(), parent);
    assert_eq!(path.as_str(), source);
    assert_eq!(policy.class, Some(OverlayClass::Expand));
    Ok(())
}

fn check_structure(policy: &overlay::OperationOverlay, member: &str, value: &str) -> TestResult {
    let (field, expected) = value.split_once('=').ok_or("invalid structure contract")?;
    let Some(FixedValue::Structure(fields)) = policy.fixed.get(&MemberPath::new(member)?) else {
        return Err("missing fixed structure".into());
    };
    let Some(FixedValue::String(actual)) = fields.get(field) else {
        return Err("missing structure string".into());
    };
    assert_eq!(actual, expected);
    Ok(())
}

fn check_correlated(policy: &overlay::OperationOverlay, request: &str, selected: &str) -> TestResult {
    let fields = std::iter::once(MemberPath::new(request).map(overlay::IdentityField::Request))
        .chain(selected.split(',').map(|field| MemberPath::selected(field).map(overlay::IdentityField::Selected)))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(policy.identity, Some(overlay::Identity::CorrelatedKey(fields)));
    Ok(())
}

fn check_class(policy: &overlay::OperationOverlay, expected: OverlayClass) {
    assert_eq!(policy.class.as_ref(), Some(&expected));
    if matches!(expected, OverlayClass::Deny(_) | OverlayClass::Defer) {
        assert!(policy.items.is_none());
        assert!(policy.identity.is_none());
    }
}

fn check_filter(policy: &overlay::OperationOverlay, member: &str, value: &str) -> TestResult {
    let (key, values) = value.split_once(':').ok_or("invalid filter contract")?;
    let (field, name) = key.split_once('=').ok_or("invalid filter key")?;
    check_fixed(policy, member, FixedValue::list([FixedValue::structure([
        (field, FixedValue::String(name.to_owned())),
        ("Values", FixedValue::Strings(vec![values.to_owned()])),
    ])?]))
}

fn check_branches(policy: &overlay::OperationOverlay, input: &str, value: &str) -> TestResult {
    let (parent, selected) = value.split_once(':').ok_or("invalid branch contract")?;
    assert_eq!(policy.branches.len(), 2);
    assert!(policy.bind.is_empty());
    for branch in &policy.branches {
        if input != "ResourceArn" {
            assert_eq!(branch.fixed.get(&MemberPath::new("Scope")?), Some(&FixedValue::String(branch.discriminator.clone())));
        }
        assert_eq!(branch.bind.get(&MemberPath::new(input)?), Some(&overlay::Binding::FromBranch {
            operation: overlay::OperationName::new(parent)?,
            branch: branch.discriminator.clone(),
            path: MemberPath::new(selected)?,
        }));
    }
    Ok(())
}

fn check_features(policy: &overlay::OperationOverlay, input: &str, values: &str) -> TestResult {
    let expected = values.split(',').collect::<Vec<_>>();
    assert_eq!(policy.branches.len(), expected.len());
    for (branch, value) in policy.branches.iter().zip(expected) {
        assert_eq!(branch.fixed.get(&MemberPath::new(input)?), Some(&FixedValue::String(value.to_owned())));
    }
    Ok(())
}
