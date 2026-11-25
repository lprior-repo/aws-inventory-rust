use super::Detail;
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
