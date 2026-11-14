use super::wire_time::{fraction_digits, fraction_length};
use crate::request::{EncodeError, common::BoundedBody};
use inventory_core::record::FieldId;
pub(super) fn write_datetime(
    value: inventory_core::record::Time,
    field: Option<FieldId>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    let (year, month, day, hour, minute, second) = calendar(value.seconds(), field)?;
    let mut text = TextBuffer {
        bytes: [0; 40],
        length: 0,
    };
    use std::fmt::Write;
    write!(
        &mut text,
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"
    )
    .map_err(|_| EncodeError::Limit("timestamp"))?;
    append_datetime_fraction(&mut text, value.nanoseconds())?;
    text.write_str("Z")
        .map_err(|_| EncodeError::Limit("timestamp"))?;
    write_buffer(body, &text)
}

fn append_datetime_fraction(text: &mut TextBuffer, nanos: u32) -> Result<(), EncodeError> {
    if nanos == 0 {
        return Ok(());
    }
    let digits = fraction_digits(nanos)?;
    let end = fraction_length(&digits)?;
    use std::fmt::Write;
    text.write_str(".")
        .map_err(|_| EncodeError::Limit("timestamp"))?;
    let fraction = std::str::from_utf8(digits.get(..end).ok_or(EncodeError::Limit("timestamp"))?)
        .map_err(|_| EncodeError::Limit("timestamp"))?;
    text.write_str(fraction)
        .map_err(|_| EncodeError::Limit("timestamp"))
}

pub(super) fn write_http_date(
    value: inventory_core::record::Time,
    field: Option<FieldId>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    let (year, month, day, hour, minute, second) = calendar(value.seconds(), field)?;
    let weekday = weekday_name(value.seconds(), field)?;
    let month = month_name(month, field)?;
    let mut text = TextBuffer {
        bytes: [0; 40],
        length: 0,
    };
    use std::fmt::Write;
    write!(
        &mut text,
        "{weekday}, {day:02} {month} {year:04} {hour:02}:{minute:02}:{second:02} GMT"
    )
    .map_err(|_| EncodeError::Limit("timestamp"))?;
    write_buffer(body, &text)
}

fn weekday_name(seconds: i64, field: Option<FieldId>) -> Result<&'static str, EncodeError> {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let day = seconds
        .div_euclid(86_400)
        .checked_add(4)
        .ok_or(EncodeError::InvalidValue {
            field,
            reason: "timestamp weekday is out of range",
        })?;
    let index = usize::try_from(day.rem_euclid(7)).map_err(|_| EncodeError::InvalidValue {
        field,
        reason: "timestamp weekday is out of range",
    })?;
    WEEKDAYS
        .get(index)
        .copied()
        .ok_or(EncodeError::InvalidValue {
            field,
            reason: "timestamp weekday is out of range",
        })
}

fn month_name(month: i64, field: Option<FieldId>) -> Result<&'static str, EncodeError> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let index = usize::try_from(month.checked_sub(1).ok_or(EncodeError::InvalidValue {
        field,
        reason: "timestamp month is out of range",
    })?)
    .map_err(|_| EncodeError::InvalidValue {
        field,
        reason: "timestamp month is out of range",
    })?;
    MONTHS.get(index).copied().ok_or(EncodeError::InvalidValue {
        field,
        reason: "timestamp month is out of range",
    })
}

fn write_buffer(body: &mut BoundedBody, text: &TextBuffer) -> Result<(), EncodeError> {
    let bytes = text
        .bytes
        .get(..text.length)
        .ok_or(EncodeError::Limit("timestamp"))?;
    let value = std::str::from_utf8(bytes).map_err(|_| EncodeError::Limit("timestamp"))?;
    super::json::string(body, value)
}

fn calendar(
    seconds: i64,
    field: Option<FieldId>,
) -> Result<(i64, i64, i64, i64, i64, i64), EncodeError> {
    let (year, month, day) = calendar_date(seconds.div_euclid(86_400), field)?;
    let day_seconds = seconds.rem_euclid(86_400);
    Ok((
        year,
        month,
        day,
        day_seconds / 3_600,
        day_seconds / 60 % 60,
        day_seconds % 60,
    ))
}

fn calendar_date(days: i64, field: Option<FieldId>) -> Result<(i64, i64, i64), EncodeError> {
    let (era, day_of_era, year_of_era) = calendar_year(days, field)?;
    calendar_month(era, day_of_era, year_of_era, field)
}

fn calendar_year(days: i64, field: Option<FieldId>) -> Result<(i64, i64, i64), EncodeError> {
    let checked = |value: Option<i64>| {
        value.ok_or(EncodeError::InvalidValue {
            field,
            reason: "timestamp date is out of range",
        })
    };
    let shifted = checked(days.checked_add(719_468))?;
    let era = shifted.div_euclid(146_097);
    let era_days = checked(era.checked_mul(146_097))?;
    let day_of_era = checked(shifted.checked_sub(era_days))?;
    let year_of_era = checked(day_of_era.checked_sub(day_of_era / 1_460))?;
    let year_of_era = checked(year_of_era.checked_add(day_of_era / 36_524))?;
    let year_of_era = checked(year_of_era.checked_sub(day_of_era / 146_096))? / 365;
    Ok((era, day_of_era, year_of_era))
}

fn calendar_month(
    era: i64,
    day_of_era: i64,
    year_of_era: i64,
    field: Option<FieldId>,
) -> Result<(i64, i64, i64), EncodeError> {
    let checked = |value: Option<i64>| {
        value.ok_or(EncodeError::InvalidValue {
            field,
            reason: "timestamp date is out of range",
        })
    };
    let mut year = checked(year_of_era.checked_add(checked(era.checked_mul(400))?))?;
    let year_days = checked(year_of_era.checked_mul(365))?;
    let year_days = checked(year_days.checked_add(year_of_era / 4))?;
    let year_days = checked(year_days.checked_sub(year_of_era / 100))?;
    let day_of_year = checked(day_of_era.checked_sub(year_days))?;
    let month_prime = checked(day_of_year.checked_mul(5))?;
    let month_prime = checked(month_prime.checked_add(2))? / 153;
    let month_days = checked(month_prime.checked_mul(153))?;
    let month_days = checked(month_days.checked_add(2))? / 5;
    let day = checked(day_of_year.checked_sub(month_days))?;
    let day = checked(day.checked_add(1))?;
    let month = checked(month_prime.checked_add(if month_prime < 10 { 3 } else { -9 }))?;
    if month <= 2 {
        year = checked(year.checked_add(1))?;
    }
    if !(0..=9999).contains(&year) {
        return Err(EncodeError::InvalidValue {
            field,
            reason: "timestamp date is out of range",
        });
    }
    Ok((year, month, day))
}
struct TextBuffer {
    bytes: [u8; 40],
    length: usize,
}

impl std::fmt::Write for TextBuffer {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let end = self
            .length
            .checked_add(value.len())
            .ok_or(std::fmt::Error)?;
        let destination = self
            .bytes
            .get_mut(self.length..end)
            .ok_or(std::fmt::Error)?;
        destination.copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}
