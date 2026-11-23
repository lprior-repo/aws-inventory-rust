#[path = "support/required_parent.rs"]
mod fixture;
use fixture::{Case, TestResult};

#[test]
fn given_type_versions_when_compiled_then_private_type_arn_is_required() -> TestResult {
    fixture::verify(Case {
        service: "cloudformation",
        date: "2010-05-15",
        operation: "ListTypeVersions",
        parent: "ListTypes",
        input: "Arn",
        output: "TypeArn",
    })
}

#[test]
fn given_registry_types_when_planned_then_visibility_is_account_private() -> TestResult {
    let policies = catalog_tool::overlay::embedded()?;
    let policy = policies
        .iter()
        .find(|policy| policy.service() == "cloudformation")
        .ok_or("cloudformation missing")?;
    let operation = catalog_tool::overlay::OperationName::new("ListTypes")?;
    let types = policy
        .operations()
        .get(&operation)
        .ok_or("ListTypes missing")?;
    let visibility = catalog_tool::overlay::MemberPath::new("Visibility")?;
    assert_eq!(
        types.fixed.get(&visibility),
        Some(&catalog_tool::overlay::FixedValue::String(
            "PRIVATE".to_owned()
        ))
    );
    Ok(())
}
