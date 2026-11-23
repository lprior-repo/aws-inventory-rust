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
