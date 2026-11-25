use super::{Detail, Outcome, child, sampled};
use inventory_core::catalog::runtime::{CallablePlan, RuntimeCatalog};
use inventory_engine::limiter::ApiError;
use inventory_sweep::{ChildProbeOutcome, SweepError};
use std::fmt::Write;

type TestResult = Result<(), std::fmt::Error>;

#[test]
fn given_control_and_whitespace_when_formatted_then_detail_is_one_line() -> TestResult {
    let mut detail = Detail(String::new());
    write!(detail, "denied\nsecret\r\tvalue\u{2028}end")?;
    assert_eq!(detail.0, "denied secret  value end");
    Ok(())
}

#[test]
fn given_multibyte_boundary_when_formatted_then_detail_stays_bounded_and_valid() -> TestResult {
    let mut detail = Detail("a".repeat(1023));
    detail.write_str("λsuffix")?;
    assert_eq!(detail.0, "a".repeat(1023));
    detail.write_str("xy")?;
    assert_eq!(detail.0.len(), 1024);
    assert!(detail.0.ends_with('x'));
    detail.write_str("ignored")?;
    assert_eq!(detail.0.len(), 1024);
    Ok(())
}

#[test]
fn given_absent_feature_when_probed_then_empty_row_keeps_the_reason()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = include_bytes!("../../../../../catalog/aws-api.catalog");
    let catalog = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(bytes)?;
    let plan = catalog
        .lookup("organizations", "ListAccounts")
        .ok_or("plan absent")?;
    let error = inventory_sweep::SweepError::Absent("AWSOrganizationsNotInUseException".into());
    let outcome = super::failed(plan, &error);
    match outcome {
        super::Outcome::EmptyReason(reason) => {
            assert!(reason.contains("AWSOrganizationsNotInUseException"))
        }
        _ => return Err("absent feature must produce empty-with-reason".into()),
    }
    Ok(())
}

#[test]
fn given_mixed_sample_outcomes_when_settled_then_hard_failure_precedes_items_and_unsupported()
-> Result<(), Box<dyn std::error::Error>> {
    let catalog = catalog()?;
    let plan = plan(&catalog)?;
    let hard = SweepError::Api(ApiError::Denied("policy".into()));
    let unsupported = SweepError::Api(ApiError::UnsupportedType("unavailable".into()));
    let absent = SweepError::Absent("not configured".into());
    assert!(matches!(
        child(plan, ChildProbeOutcome::Called { items: 2, errors: vec![unsupported, absent, hard] }),
        Outcome::Failed { class: "denied", detail }
            if detail == "API executor error: access denied: policy"
    ));
    let unsupported = SweepError::Api(ApiError::UnsupportedType("unavailable".into()));
    let absent = SweepError::Absent("not configured".into());
    let errors = [absent, unsupported];
    assert!(matches!(sampled(plan, 2, &errors), Outcome::Items(2)));
    assert!(matches!(
        sampled(plan, 0, &errors),
        Outcome::Unsupported(detail)
            if detail == "API executor error: provider does not support this type: unavailable"
    ));
    let absent = SweepError::Absent("not configured".into());
    assert!(matches!(
        sampled(plan, 0, &[absent]),
        Outcome::EmptyReason(detail) if detail == "inventory feature or singleton absent: not configured"
    ));
    assert!(matches!(sampled(plan, 0, &[]), Outcome::Empty));
    Ok(())
}

fn catalog()
-> Result<RuntimeCatalog<'static>, inventory_core::catalog::runtime::RuntimeCatalogError> {
    RuntimeCatalog::from_bytes(include_bytes!("../../../../../catalog/aws-api.catalog"))
}

fn plan<'a>(catalog: &'a RuntimeCatalog<'_>) -> Result<&'a CallablePlan<'a>, &'static str> {
    catalog
        .lookup("organizations", "ListAccounts")
        .ok_or("plan absent")
}
