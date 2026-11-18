use super::{Exceptions, NameError, TestResult, sql_name};

const INVALID_PAIRS: [(&str, &str); 14] = [
    ("", "iot"),
    ("IoT", ""),
    ("IoT", "Iot"),
    ("Io-T", "iot"),
    ("IöT", "iot"),
    ("IoT", "iöt"),
    ("IoT", "iot\r"),
    ("IoT", "_iot"),
    ("IoT", "iot_"),
    ("IoT", "i__ot"),
    ("IoT", "i ot"),
    ("IoT", "iot\textra"),
    (" IoT", "iot"),
    ("IoT\n", "iot"),
];

#[test]
fn constructor_rejects_invalid_pairs_at_their_original_position() -> TestResult {
    INVALID_PAIRS.iter().for_each(|pair| {
        assert_eq!(
            Exceptions::from_entries(&[*pair]),
            Err(NameError::InvalidException { line: 1 })
        );
        assert_eq!(
            Exceptions::from_entries(&[("IP", "ip"), *pair]),
            Err(NameError::InvalidException { line: 2 })
        );
    });
    Ok(())
}

#[test]
fn constructor_reports_duplicate_spelling_on_the_second_occurrence() -> TestResult {
    assert_eq!(
        Exceptions::from_entries(&[("IoT", "iot"), ("IP", "ip"), ("IoT", "other")]),
        Err(NameError::DuplicateException { line: 3 })
    );
    Ok(())
}

#[test]
fn constructor_accepts_empty_entries_and_equal_replacements() -> TestResult {
    assert_eq!(Exceptions::from_entries(&[])?, Exceptions::none());
    let exceptions = Exceptions::from_entries(&[("X", "same_word2"), ("Y", "same_word2")])?;
    assert_eq!(sql_name("A::X", &exceptions)?.as_str(), "a_same_word2");
    assert_eq!(sql_name("A::Y", &exceptions)?.as_str(), "a_same_word2");
    Ok(())
}

#[test]
fn constructor_accepts_digit_spellings_inside_parts() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("2X", "two_x")])?;
    assert_eq!(sql_name("AWS::A2X", &exceptions)?.as_str(), "aws_a_two_x");
    Ok(())
}
