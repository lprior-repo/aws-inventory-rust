use crate::overlay::{self, Binding, MemberPath, OperationName, Paginator, TokenPair};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn operation<'a>(
    policies: &'a [crate::overlay::ServiceOverlay],
    service: &str,
    name: &str,
) -> Result<&'a crate::overlay::OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .and_then(|policy| policy.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

#[test]
fn eventbridge_list_rules_uses_token_pagination_with_event_bus_binding() -> TestResult {
    let policies = overlay::embedded()?;
    let plan = operation(&policies, "eventbridge", "ListRules")?;
    assert_eq!(
        plan.paginator,
        Some(Paginator::Token(TokenPair {
            input: MemberPath::new("NextToken")?,
            output: MemberPath::new("NextToken")?,
            optional: false,
        }))
    );
    assert_eq!(
        plan.bind.get(&MemberPath::new("EventBusName")?),
        Some(&Binding::From {
            operation: OperationName::new("ListEventBuses")?,
            path: MemberPath::new("Name")?,
        })
    );
    Ok(())
}

#[test]
fn eventbridge_describe_rule_binds_event_bus_name_from_list_rules() -> TestResult {
    let policies = overlay::embedded()?;
    let plan = operation(&policies, "eventbridge", "DescribeRule")?;
    assert_eq!(
        plan.bind.get(&MemberPath::new("Name")?),
        Some(&Binding::From {
            operation: OperationName::new("ListRules")?,
            path: MemberPath::new("Name")?,
        })
    );
    assert_eq!(
        plan.bind.get(&MemberPath::new("EventBusName")?),
        Some(&Binding::From {
            operation: OperationName::new("ListRules")?,
            path: MemberPath::new("EventBusName")?,
        })
    );
    Ok(())
}
