use super::DecodeError;
use inventory_core::{catalog::TimestampFormat, record::Time};
mod epoch;
use std::io::Write as _;
#[cfg(test)]
mod tests;

pub(super) fn parse(value: &str, format: Option<TimestampFormat>) -> Result<Time, DecodeError> {
    match format {
        Some(TimestampFormat::DateTime) => datetime(value),
        Some(TimestampFormat::HttpDate) => http_date(value),
        None | Some(TimestampFormat::EpochSeconds) => epoch::parse(value),
    }
}

fn datetime(value: &str) -> Result<Time, DecodeError> {
    let (date, time_zone) = value.split_once('T').ok_or_else(invalid)?;
    let (year, month, day) = date_parts(date)?;
    let (time, offset) = split_zone(time_zone)?;
    let (hour, minute, second, nanos) = time_parts(time)?;
    let days = days_from_civil(year, month, day)?;
    let time_seconds = hour
        .checked_mul(3_600)
        .and_then(|value| {
            minute
                .checked_mul(60)
                .and_then(|minutes| value.checked_add(minutes))
        })
        .and_then(|value| value.checked_add(second))
        .ok_or_else(invalid)?;
    let seconds = days
        .checked_mul(86_400)
        .and_then(|n| n.checked_add(time_seconds))
        .and_then(|n| n.checked_sub(offset))
        .ok_or_else(invalid)?;
    Time::new(seconds, nanos).map_err(|_| invalid())
}

fn date_parts(value: &str) -> Result<(i64, i64, i64), DecodeError> {
    let mut parts = value.split('-');
    let year = parse_part(parts.next())?;
    let month = parse_part(parts.next())?;
    let day = parse_part(parts.next())?;
    if parts.next().is_some() {
        return Err(invalid());
    }
    Ok((year, month, day))
}

fn split_zone(value: &str) -> Result<(&str, i64), DecodeError> {
    if let Some(time) = value.strip_suffix('Z') {
        return Ok((time, 0));
    }
    let index = value.rfind(['+', '-']).ok_or_else(invalid)?;
    let sign = zone_sign(value, index);
    let time = value.get(..index).ok_or_else(invalid)?;
    let zone = value
        .get(index.checked_add(1).ok_or_else(invalid)?..)
        .ok_or_else(invalid)?;
    let (hours, minutes) = zone.split_once(':').ok_or_else(invalid)?;
    let hours = parse_part(Some(hours))?;
    let minutes = parse_part(Some(minutes))?;
    if hours > 23 || minutes > 59 {
        return Err(invalid());
    }
    let offset = hours
        .checked_mul(3_600)
        .and_then(|n| minutes.checked_mul(60).and_then(|m| n.checked_add(m)))
        .ok_or_else(invalid)?;
    Ok((time, offset.checked_mul(sign).ok_or_else(invalid)?))
}

fn zone_sign(value: &str, index: usize) -> i64 {
    if value.as_bytes().get(index) == Some(&b'+') {
        1
    } else {
        -1
    }
}

fn time_parts(value: &str) -> Result<(i64, i64, i64, u32), DecodeError> {
    let mut parts = value.split(':');
    let hour = parse_part(parts.next())?;
    let minute = parse_part(parts.next())?;
    let seconds = parts.next().ok_or_else(invalid)?;
    if parts.next().is_some() {
        return Err(invalid());
    }
    let (second_text, fraction) = seconds
        .split_once('.')
        .map_or((seconds, None), |(s, f)| (s, Some(f)));
    let second = parse_part(Some(second_text))?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(invalid());
    }
    Ok((hour, minute, second, fraction_nanos(fraction)?))
}

fn fraction_nanos(fraction: Option<&str>) -> Result<u32, DecodeError> {
    let Some(fraction) = fraction else {
        return Ok(0);
    };
    if fraction.is_empty()
        || fraction.len() > 9
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid());
    }
    let digits = fraction.parse::<u32>().map_err(|_| invalid())?;
    let exponent = u32::try_from(9usize.checked_sub(fraction.len()).ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    digits
        .checked_mul(10u32.checked_pow(exponent).ok_or_else(invalid)?)
        .ok_or_else(invalid)
}

fn http_date(value: &str) -> Result<Time, DecodeError> {
    let mut parts = value.split_ascii_whitespace();
    let _weekday = parts.next().ok_or_else(invalid)?;
    let day = parse_part(parts.next())?;
    let month = month_number(parts.next().ok_or_else(invalid)?)?;
    let year = parse_part(parts.next())?;
    let (hour, minute, second) = clock(parts.next().ok_or_else(invalid)?)?;
    if parts.next() != Some("GMT") || parts.next().is_some() {
        return Err(invalid());
    }
    let time_seconds = hour
        .checked_mul(3_600)
        .and_then(|value| {
            minute
                .checked_mul(60)
                .and_then(|minutes| value.checked_add(minutes))
        })
        .and_then(|value| value.checked_add(second))
        .ok_or_else(invalid)?;
    let seconds = days_from_civil(year, month, day)?
        .checked_mul(86_400)
        .and_then(|n| n.checked_add(time_seconds))
        .ok_or_else(invalid)?;
    Time::new(seconds, 0).map_err(|_| invalid())
}

fn clock(value: &str) -> Result<(i64, i64, i64), DecodeError> {
    let mut parts = value.split(':');
    let hour = parse_part(parts.next())?;
    let minute = parse_part(parts.next())?;
    let second = parse_part(parts.next())?;
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return Err(invalid());
    }
    Ok((hour, minute, second))
}

fn month_number(value: &str) -> Result<i64, DecodeError> {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|month| *month == value)
    .and_then(|index| index.checked_add(1))
    .and_then(|month| i64::try_from(month).ok())
    .ok_or_else(invalid)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> Result<i64, DecodeError> {
    if !(1..=12).contains(&month) || day < 1 || day > month_days(year, month) {
        return Err(invalid());
    }
    let adjusted = year
        .checked_sub(i64::from(month <= 2))
        .ok_or_else(invalid)?;
    let era = adjusted.div_euclid(400);
    let year_of_era = adjusted
        .checked_sub(era.checked_mul(400).ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    let day_of_year = day_of_year(month, day)?;
    let day_of_era = year_of_era
        .checked_mul(365)
        .and_then(|value| value.checked_add(year_of_era / 4))
        .and_then(|value| value.checked_sub(year_of_era / 100))
        .and_then(|value| value.checked_add(day_of_year))
        .ok_or_else(invalid)?;
    era.checked_mul(146_097)
        .and_then(|n| n.checked_add(day_of_era))
        .and_then(|n| n.checked_sub(719_468))
        .ok_or_else(invalid)
}

fn day_of_year(month: i64, day: i64) -> Result<i64, DecodeError> {
    let shifted = month
        .checked_add(if month > 2 { -3 } else { 9 })
        .ok_or_else(invalid)?;
    153_i64
        .checked_mul(shifted)
        .and_then(|value| value.checked_add(2))
        .map(|value| value / 5)
        .and_then(|value| value.checked_add(day))
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(invalid)
}

fn month_days(year: i64, month: i64) -> i64 {
    match month {
        2 if year.rem_euclid(4) == 0
            && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0) =>
        {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn parse_part(value: Option<&str>) -> Result<i64, DecodeError> {
    let value = value.ok_or_else(invalid)?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    value.parse().map_err(|_| invalid())
}
fn invalid() -> DecodeError {
    DecodeError::Invalid("invalid timestamp")
}

pub(super) fn from_float(value: f64) -> Result<Time, DecodeError> {
    if !value.is_finite() {
        return Err(invalid());
    }
    let mut bytes = [0; 512];
    let mut writer = std::io::Cursor::new(bytes.as_mut_slice());
    write!(&mut writer, "{value}").map_err(|_| invalid())?;
    let length = usize::try_from(writer.position()).map_err(|_| invalid())?;
    let value =
        std::str::from_utf8(bytes.get(..length).ok_or_else(invalid)?).map_err(|_| invalid())?;
    epoch::parse(value)
}
