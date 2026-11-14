use super::{Exceptions, NameError, TestResult, sql_name};

const INVALID_ROWS: [(&str, usize); 19] = [
    ("IoT\n", 1),
    ("\n", 1),
    ("IoT\tiot\n\n", 2),
    ("IoT\tiot", 1),
    ("IoT\tiot\nIP\tip", 2),
    ("IoT\tIot\n", 1),
    ("IoT\tiot\nIP\tIP\n", 2),
    ("\tiot\n", 1),
    ("IoT\t\n", 1),
    ("Io-T\tiot\n", 1),
    ("IöT\tiot\n", 1),
    ("IoT\tiöt\n", 1),
    ("IoT\tiot\r\n", 1),
    ("IoT\t_iot\n", 1),
    ("IoT\tiot_\n", 1),
    ("IoT\ti__ot\n", 1),
    ("IoT\ti ot\n", 1),
    ("IoT\tiot\textra\n", 1),
    (" IoT\tiot\n", 1),
];

#[test]
fn parser_rejects_malformed_rows_with_their_line_number() -> TestResult {
    INVALID_ROWS.iter().for_each(|(text, line)| {
        assert_eq!(
            Exceptions::parse(text),
            Err(NameError::InvalidException { line: *line }),
            "{text:?}"
        );
    });
    Ok(())
}

#[test]
fn parser_reports_duplicate_spelling_on_the_second_occurrence() -> TestResult {
    assert_eq!(
        Exceptions::parse("IoT\tiot\nIP\tip\nIoT\tother\n"),
        Err(NameError::DuplicateException { line: 3 })
    );
    Ok(())
}

#[test]
fn parser_accepts_empty_text_and_distinct_spellings_with_equal_replacements() -> TestResult {
    assert_eq!(Exceptions::parse("")?, Exceptions::none());
    let exceptions = Exceptions::parse("X\tsame_word2\nY\tsame_word2\n")?;
    assert_eq!(sql_name("A::X", &exceptions)?.as_str(), "a_same_word2");
    assert_eq!(sql_name("A::Y", &exceptions)?.as_str(), "a_same_word2");
    Ok(())
}

#[test]
fn parser_accepts_digit_spellings_and_matches_them_inside_parts() -> TestResult {
    let exceptions = Exceptions::parse("2X\ttwo_x\n")?;
    assert_eq!(sql_name("AWS::A2X", &exceptions)?.as_str(), "aws_a_two_x");
    Ok(())
}
