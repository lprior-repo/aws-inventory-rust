//! Python-compatible timestamps and grouped counts for evidence receipts.

use crate::HarnessError;

/// Current UTC timestamp rendered like `datetime.now(timezone.utc).isoformat()`.
pub fn iso8601_now() -> Result<String, HarnessError> {
    let (days, hour, minute, second, micros) = utc_parts()?;
    let (year, month, day) = civil_from_days(days);
    let base = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    if micros == 0 {
        Ok(format!("{base}+00:00"))
    } else {
        Ok(format!("{base}.{micros:06}+00:00"))
    }
}

fn clock_error() -> HarnessError {
    HarnessError::invalid("system clock is unusable")
}

fn clock_part(value: u64, divisor: u64) -> Result<u64, HarnessError> {
    value.checked_div(divisor).ok_or_else(clock_error)
}

fn clock_rem(value: u64, divisor: u64) -> Result<u64, HarnessError> {
    value.checked_rem(divisor).ok_or_else(clock_error)
}

/// Split the Unix epoch clock into days and time of day.
fn utc_parts() -> Result<(i64, u64, u64, u64, u32), HarnessError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| HarnessError::invalid("system clock precedes the Unix epoch"))?;
    let seconds = elapsed.as_secs();
    let micros = elapsed.subsec_micros();
    let days = clock_part(seconds, 86_400)?;
    let remainder = clock_rem(seconds, 86_400)?;
    let hour = clock_part(remainder, 3_600)?;
    let minute = clock_part(clock_rem(remainder, 3_600)?, 60)?;
    let second = clock_rem(remainder, 60)?;
    let days = i64::try_from(days).map_err(|_| clock_error())?;
    Ok((days, hour, minute, second, micros))
}

/// Convert days since 1970-01-01 into a civil calendar date.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days.saturating_add(719_468);
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = era_year(day_of_era);
    let year = year_of_era.saturating_add(era.saturating_mul(400));
    let day_of_year = year_day_offset(year_of_era, day_of_era);
    let month_position = day_of_year
        .saturating_mul(5)
        .saturating_add(2)
        .div_euclid(153);
    let day = month_day(day_of_year, month_position);
    let month = month_number(month_position);
    let year = if month <= 2 {
        year.saturating_add(1)
    } else {
        year
    };
    (year, month, day)
}

fn era_year(day_of_era: i64) -> i64 {
    day_of_era
        .saturating_sub(day_of_era.div_euclid(1_460))
        .saturating_add(day_of_era.div_euclid(36_524))
        .saturating_sub(day_of_era.div_euclid(146_096))
        .div_euclid(365)
}

fn year_day_offset(year_of_era: i64, day_of_era: i64) -> i64 {
    day_of_era.saturating_sub(
        year_of_era
            .saturating_mul(365)
            .saturating_add(year_of_era.div_euclid(4))
            .saturating_sub(year_of_era.div_euclid(100)),
    )
}

fn month_day(day_of_year: i64, month_position: i64) -> i64 {
    day_of_year
        .saturating_sub(
            month_position
                .saturating_mul(153)
                .saturating_add(2)
                .div_euclid(5),
        )
        .saturating_add(1)
}

fn month_number(month_position: i64) -> i64 {
    if month_position < 10 {
        month_position.saturating_add(3)
    } else {
        month_position.saturating_sub(9)
    }
}

/// Format an integer with Python's `,` thousands grouping.
pub fn group_thousands(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let total = digits.chars().count();
    let mut grouped = String::new();
    for (index, character) in digits.chars().enumerate() {
        let position = total.saturating_sub(index);
        if index > 0 && position.rem_euclid(3) == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}
