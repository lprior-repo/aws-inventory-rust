use super::{TestResult, filters, with_field};
use super::super::MODEL;
use catalog_tool::{classes::DenyList, overlay::{self, FixedValue}};

type CheckResult = Result<overlay::CheckedOverlays, Box<dyn std::error::Error>>;

fn check(literal: FixedValue, attributes: FixedValue) -> CheckResult {
    let directory = tempfile::tempdir()?;
    let model = directory.path().join("models/fixed/service/2020-01-01");
    std::fs::create_dir_all(&model)?;
    std::fs::write(model.join("model.json"), MODEL.replace("PROTOCOL", "aws.protocols#awsJson1_0"))?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new("Id")?]));
    plan.paginator = Some(overlay::Paginator::None);
    plan.fixed.insert(overlay::MemberPath::new("Filters")?, literal);
    plan.fixed.insert(overlay::MemberPath::new("Attributes")?, attributes);
    let policies = [overlay::ServiceOverlay::new("fixed", vec![("ListClusters", plan)])?];
    Ok(overlay::check_policies(directory.path(), &policies, &DenyList::builtin())?)
}
fn attributes() -> Result<FixedValue, overlay::NameError> {
    FixedValue::map([("owner", FixedValue::String("inventory".into()))])
}
fn accepted(result: CheckResult) -> TestResult {
    let checked = result?;
    assert_eq!(checked.services().len(), 1);
    let service = checked.services().first().ok_or("checked service absent")?;
    assert_eq!(service.service(), "fixed");
    assert_eq!(service.callable().len(), 1);
    let operation = service.callable().first().ok_or("checked operation absent")?;
    assert_eq!(operation.operation(), "ListClusters");
    assert_eq!(operation.plan().fixed.len(), 2);
    Ok(())
}
fn rejected(result: CheckResult) -> TestResult {
    let error = result.err().ok_or("invalid enum accepted before augment")?;
    let Some(overlay::CheckError::Violations(violations)) = error.downcast_ref::<overlay::CheckError>() else {
        return Err(format!("expected source InvalidValue, got {error}").into());
    };
    assert_eq!(violations.iter().map(|violation| violation.rule()).collect::<Vec<_>>(), vec![overlay::ViolationRule::InvalidValue]);
    let violation = violations.first().ok_or("violation absent")?;
    assert_eq!(violation.service(), "fixed");
    assert_eq!(violation.operation(), "ListClusters");
    Ok(())
}

#[test]
fn given_integer_enum_list_when_policy_checked_before_augment_then_unknown_element_is_invalid_value() -> TestResult {
    accepted(check(with_field("Items", FixedValue::list([FixedValue::Integer(1), FixedValue::Integer(7)]))?, attributes()?))?;
    rejected(check(with_field("Items", FixedValue::list([FixedValue::Integer(6)]))?, attributes()?))
}

#[test]
fn given_integer_enum_map_value_when_policy_checked_before_augment_then_unknown_value_is_invalid_value() -> TestResult {
    accepted(check(with_field("Attributes", FixedValue::map([("owner", FixedValue::Integer(7))])?)?, attributes()?))?;
    rejected(check(with_field("Attributes", FixedValue::map([("owner", FixedValue::Integer(6))])?)?, attributes()?))
}

#[test]
fn given_nested_enum_map_key_when_policy_checked_before_augment_then_unknown_key_is_invalid_value() -> TestResult {
    accepted(check(with_field("Attributes", FixedValue::map([("owner", FixedValue::Integer(1))])?)?, attributes()?))?;
    rejected(check(with_field("Attributes", FixedValue::map([("not-an-owner", FixedValue::Integer(1))])?)?, attributes()?))
}

#[test]
fn given_top_level_enum_map_key_when_policy_checked_before_augment_then_unknown_key_is_invalid_value() -> TestResult {
    accepted(check(filters("neptune")?, attributes()?))?;
    rejected(check(filters("neptune")?, FixedValue::map([("not-an-owner", FixedValue::String("inventory".into()))])?))
}
