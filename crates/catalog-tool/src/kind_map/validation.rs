use super::KindMapError;
pub(super) fn row_error(line: usize, reason: &'static str) -> KindMapError {
    KindMapError::InvalidRow { line, reason }
}
pub(super) fn valid_kind(value: &str) -> bool {
    valid_cfn_type(value)
}
pub(super) fn valid_reason(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}
pub(super) fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
pub(super) fn valid_path(value: &str) -> bool {
    value.len() <= 256 && value.split('.').count() <= 8 && value.split('.').all(valid_ident)
}
pub(super) fn valid_cfn_type(value: &str) -> bool {
    let mut parts = value.split("::");
    parts.next().is_some_and(valid_ident)
        && parts.next().is_some_and(valid_ident)
        && parts.next().is_some_and(valid_ident)
        && parts.all(valid_ident)
}
fn valid_ident(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
pub(super) fn valid_date(value: &str) -> bool {
    let shape = value.bytes().enumerate().all(|(index, byte)| match index {
        4 | 7 => byte == b'-',
        _ => byte.is_ascii_digit(),
    });
    if value.len() != 10 || !shape {
        return false;
    }
    let parts: Vec<&str> = value.split('-').collect();
    let [year, month, day] = parts.as_slice() else {
        return false;
    };
    match (year.parse::<u16>(), month.parse::<u8>(), day.parse::<u8>()) {
        (Ok(year), Ok(month), Ok(day)) => valid_calendar_day(year, month, day),
        _ => false,
    }
}

fn valid_calendar_day(year: u16, month: u8, day: u8) -> bool {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year > 0 && day > 0 && day <= max_day
}
pub(super) fn valid_service(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
pub(super) fn valid_operation(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}
