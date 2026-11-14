use super::EncodeError;
use inventory_core::{catalog::TimestampFormat, record::Time};
use std::fmt::Write;

pub(super) fn format(time: Time, format: Option<TimestampFormat>) -> Result<String, EncodeError> {
    let mut output = String::new();
    output
        .try_reserve_exact(64)
        .map_err(EncodeError::Allocation)?;
    match format.unwrap_or(TimestampFormat::DateTime) {
        TimestampFormat::DateTime => date_time(&mut output, time)?,
        TimestampFormat::HttpDate => http_date(&mut output, time)?,
        TimestampFormat::EpochSeconds => epoch(&mut output, time)?,
    }
    Ok(output)
}

fn date_time(output: &mut String, time: Time) -> Result<(), EncodeError> {
    let (year, month, day, hour, minute, second) = parts(time)?;
    write!(
        output,
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"
    )
    .map_err(|_| invalid("timestamp formatting failed"))?;
    fraction(output, time.nanoseconds())?;
    output.push('Z');
    Ok(())
}

fn http_date(output: &mut String, time: Time) -> Result<(), EncodeError> {
    let days = time.seconds().div_euclid(86_400);
    let (year, month, day) = civil(days)?;
    let weekday = names(
        days.rem_euclid(7),
        &["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"],
    )?;
    let month_name = names(
        month
            .checked_sub(1)
            .ok_or_else(|| invalid("timestamp month is invalid"))?,
        &[
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
    )?;
    let seconds = time.seconds().rem_euclid(86_400);
    write!(
        output,
        "{weekday}, {day:02} {month_name} {year:04} {:02}:{:02}:{:02} GMT",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
    .map_err(|_| invalid("timestamp formatting failed"))
}

fn epoch(output: &mut String, time: Time) -> Result<(), EncodeError> {
    if time.seconds() < 0 && time.nanoseconds() > 0 {
        let magnitude = time
            .seconds()
            .checked_add(1)
            .and_then(i64::checked_neg)
            .ok_or_else(|| invalid("timestamp is out of range"))?;
        write!(output, "-{magnitude}").map_err(|_| invalid("timestamp formatting failed"))?;
        fraction(
            output,
            1_000_000_000_u32
                .checked_sub(time.nanoseconds())
                .ok_or_else(|| invalid("timestamp nanoseconds are invalid"))?,
        )?;
    } else {
        write!(output, "{}", time.seconds()).map_err(|_| invalid("timestamp formatting failed"))?;
        fraction(output, time.nanoseconds())?;
    }
    Ok(())
}

fn parts(time: Time) -> Result<(i64, i64, i64, i64, i64, i64), EncodeError> {
    let days = time.seconds().div_euclid(86_400);
    let (year, month, day) = civil(days)?;
    let seconds = time.seconds().rem_euclid(86_400);
    Ok((
        year,
        month,
        day,
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60,
    ))
}

fn civil(days: i64) -> Result<(i64, i64, i64), EncodeError> {
    let z = days
        .checked_add(719_468)
        .ok_or_else(|| invalid("timestamp is out of range"))?;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = doe
        .checked_sub(doe / 1_460)
        .and_then(|value| value.checked_add(doe / 36_524))
        .and_then(|value| value.checked_sub(doe / 146_096))
        .ok_or_else(|| invalid("timestamp year is out of range"))?
        / 365;
    let year = era
        .checked_mul(400)
        .and_then(|value| value.checked_add(yoe))
        .ok_or_else(|| invalid("timestamp year is out of range"))?;
    let doy = yoe
        .checked_mul(365)
        .and_then(|value| value.checked_add(yoe / 4))
        .and_then(|value| value.checked_sub(yoe / 100))
        .and_then(|value| doe.checked_sub(value))
        .ok_or_else(|| invalid("timestamp day is out of range"))?;
    civil_day(year, doy)
}

fn civil_day(year: i64, doy: i64) -> Result<(i64, i64, i64), EncodeError> {
    let mp = doy
        .checked_mul(5)
        .and_then(|value| value.checked_add(2))
        .ok_or_else(|| invalid("timestamp month is out of range"))?
        / 153;
    let day = mp
        .checked_mul(153)
        .and_then(|value| value.checked_add(2))
        .and_then(|value| doy.checked_sub(value / 5))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| invalid("timestamp day is out of range"))?;
    let month = mp
        .checked_add(if mp < 10 { 3 } else { -9 })
        .ok_or_else(|| invalid("timestamp month is out of range"))?;
    let year = year
        .checked_add(i64::from(month <= 2))
        .ok_or_else(|| invalid("timestamp year is out of range"))?;
    Ok((year, month, day))
}

fn names<'a>(index: i64, values: &'a [&'a str]) -> Result<&'a str, EncodeError> {
    let index =
        usize::try_from(index).map_err(|_| invalid("timestamp calendar value is invalid"))?;
    values
        .get(index)
        .copied()
        .ok_or_else(|| invalid("timestamp calendar value is invalid"))
}

fn fraction(output: &mut String, nanos: u32) -> Result<(), EncodeError> {
    if nanos == 0 {
        return Ok(());
    }
    write!(output, ".{nanos:09}").map_err(|_| invalid("timestamp formatting failed"))?;
    while output.ends_with('0') {
        output.pop();
    }
    Ok(())
}

fn invalid(reason: &'static str) -> EncodeError {
    EncodeError::InvalidValue {
        field: None,
        reason,
    }
}
