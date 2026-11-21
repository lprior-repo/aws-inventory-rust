use crate::request::{EncodeError, common::BoundedBody};
use inventory_core::record::{Time, ValueRef};
use std::fmt::{self, Write};

use super::form;

pub(super) fn emit(
    body: &mut BoundedBody,
    first: &mut bool,
    name: &str,
    kind: &str,
    value: ValueRef<'_>,
    timestamp_format: Option<&str>,
    body_limit: usize,
) -> Result<(), EncodeError> {
    match (kind, value) {
        ("float" | "double", ValueRef::Text(text))
            if matches!(text, "NaN" | "Infinity" | "-Infinity") =>
        {
            form::pair(body, first, name, text)
        }
        ("string" | "enum" | "bigDecimal", ValueRef::Text(text)) => {
            form::pair(body, first, name, text)
        }
        ("boolean", ValueRef::Bool(value)) => {
            form::pair(body, first, name, if value { "true" } else { "false" })
        }
        (
            "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum",
            ValueRef::Int(value),
        ) => {
            let mut text = Text::new();
            write!(&mut text, "{value}").map_err(|_| invalid("numeric formatting failed"))?;
            form::pair(body, first, name, text.as_str()?)
        }
        ("float" | "double", ValueRef::Float(value)) => {
            let mut text = Text::new();
            write_float(&mut text, value.to_f64())?;
            form::pair(body, first, name, text.as_str()?)
        }
        ("blob", ValueRef::Bytes(bytes)) => form::pair_base64(body, first, name, bytes, body_limit),
        ("timestamp", ValueRef::Time(time)) => {
            let formatted = timestamp(time, timestamp_format)?;
            form::pair(body, first, name, formatted.as_str()?)
        }
        ("document", _) => Err(EncodeError::Unsupported(
            "document values are not supported by AWS Query protocols",
        )),
        (_, ValueRef::Null) => Ok(()),
        _ => Err(EncodeError::InvalidValue {
            field: None,
            reason: "value kind does not match modeled scalar",
        }),
    }
}

fn write_float(text: &mut Text, value: f64) -> Result<(), EncodeError> {
    if value.is_nan() {
        return text
            .write_str("NaN")
            .map_err(|_| invalid("float formatting failed"));
    }
    if value == f64::INFINITY {
        return text
            .write_str("Infinity")
            .map_err(|_| invalid("float formatting failed"));
    }
    if value == f64::NEG_INFINITY {
        return text
            .write_str("-Infinity")
            .map_err(|_| invalid("float formatting failed"));
    }
    write!(text, "{value}").map_err(|_| invalid("float formatting failed"))?;
    let formatted = text.as_str()?;
    if !formatted.contains('.') && !formatted.contains('e') && !formatted.contains('E') {
        text.write_str(".0")
            .map_err(|_| invalid("float formatting failed"))?;
    }
    Ok(())
}

fn timestamp(time: Time, format: Option<&str>) -> Result<Text, EncodeError> {
    match format.unwrap_or("date-time") {
        "date-time" => date_time(time),
        "epoch-seconds" => epoch(time),
        "http-date" => http_date(time),
        _ => Err(EncodeError::Unsupported("unknown timestamp format")),
    }
}

fn date_time(time: Time) -> Result<Text, EncodeError> {
    let days = time.seconds().div_euclid(86_400);
    let seconds = time.seconds().rem_euclid(86_400);
    let (year, month, day) = civil(days)?;
    let mut text = Text::new();
    write!(
        &mut text,
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
        seconds / 3_600,
        seconds % 3_600 / 60,
        seconds % 60
    )
    .map_err(|_| invalid("timestamp formatting failed"))?;
    fractional(&mut text, time.nanoseconds())?;
    text.write_str("Z")
        .map_err(|_| invalid("timestamp formatting failed"))?;
    Ok(text)
}

fn epoch(time: Time) -> Result<Text, EncodeError> {
    let mut text = Text::new();
    if time.seconds() < 0 && time.nanoseconds() > 0 {
        write!(
            &mut text,
            "-{}",
            time.seconds()
                .checked_add(1)
                .and_then(i64::checked_neg)
                .ok_or_else(|| invalid("timestamp is out of range"))?
        )
        .map_err(|_| invalid("timestamp formatting failed"))?;
        let fractional_nanoseconds = 1_000_000_000u32
            .checked_sub(time.nanoseconds())
            .ok_or_else(|| invalid("timestamp is out of range"))?;
        fractional(&mut text, fractional_nanoseconds)?;
    } else {
        write!(&mut text, "{}", time.seconds())
            .map_err(|_| invalid("timestamp formatting failed"))?;
        fractional(&mut text, time.nanoseconds())?;
    }
    Ok(text)
}

fn http_date(time: Time) -> Result<Text, EncodeError> {
    let days = time.seconds().div_euclid(86_400);
    let seconds = time.seconds().rem_euclid(86_400);
    let (year, month, day) = civil(days)?;
    let weekday =
        usize::try_from(days.rem_euclid(7)).map_err(|_| invalid("timestamp weekday is invalid"))?;
    let weekday = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"]
        .get(weekday)
        .ok_or_else(|| invalid("timestamp weekday is invalid"))?;
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .get(
        usize::try_from(
            month
                .checked_sub(1)
                .ok_or_else(|| invalid("timestamp month is invalid"))?,
        )
        .map_err(|_| invalid("timestamp month is invalid"))?,
    )
    .ok_or_else(|| invalid("timestamp month is invalid"))?;
    let mut text = Text::new();
    write!(
        &mut text,
        "{weekday}, {day:02} {month} {year:04} {:02}:{:02}:{:02} GMT",
        seconds / 3_600,
        seconds % 3_600 / 60,
        seconds % 60
    )
    .map_err(|_| invalid("timestamp formatting failed"))?;
    Ok(text)
}

fn fractional(text: &mut Text, nanoseconds: u32) -> Result<(), EncodeError> {
    if nanoseconds == 0 {
        return Ok(());
    }
    let mut fraction = Text::new();
    write!(&mut fraction, "{nanoseconds:09}")
        .map_err(|_| invalid("timestamp formatting failed"))?;
    let fraction = fraction.as_str()?;
    let trimmed = fraction.trim_end_matches('0');
    text.write_char('.')
        .map_err(|_| invalid("timestamp formatting failed"))?;
    text.write_str(trimmed)
        .map_err(|_| invalid("timestamp formatting failed"))
}

fn checked_add(left: i64, right: i64) -> Result<i64, EncodeError> {
    left.checked_add(right)
        .ok_or_else(|| invalid("timestamp is out of range"))
}

fn checked_sub(left: i64, right: i64) -> Result<i64, EncodeError> {
    left.checked_sub(right)
        .ok_or_else(|| invalid("timestamp is out of range"))
}

fn checked_mul(left: i64, right: i64) -> Result<i64, EncodeError> {
    left.checked_mul(right)
        .ok_or_else(|| invalid("timestamp is out of range"))
}

fn civil(days: i64) -> Result<(i64, i64, i64), EncodeError> {
    let z = checked_add(days, 719_468)?;
    let era = z.div_euclid(146_097);
    let doe = checked_sub(z, checked_mul(era, 146_097)?)?;
    let yoe = checked_sub(
        checked_add(checked_sub(doe, doe / 1_460)?, doe / 36_524)?,
        doe / 146_096,
    )? / 365;
    let year = checked_add(yoe, checked_mul(era, 400)?)?;
    let year_days = checked_sub(checked_add(checked_mul(365, yoe)?, yoe / 4)?, yoe / 100)?;
    let doy = checked_sub(doe, year_days)?;
    let mp = checked_add(checked_mul(5, doy)?, 2)? / 153;
    let month_days = checked_add(checked_mul(153, mp)?, 2)? / 5;
    let day = checked_add(checked_sub(doy, month_days)?, 1)?;
    let month = if mp < 10 {
        checked_add(mp, 3)?
    } else {
        checked_sub(mp, 9)?
    };
    let year = checked_add(year, i64::from(month <= 2))?;
    Ok((year, month, day))
}

fn invalid(reason: &'static str) -> EncodeError {
    EncodeError::InvalidValue {
        field: None,
        reason,
    }
}

struct Text {
    bytes: [u8; 128],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> Result<&str, EncodeError> {
        let bytes = self
            .bytes
            .get(..self.len)
            .ok_or_else(|| invalid("formatted scalar is not UTF-8"))?;
        std::str::from_utf8(bytes).map_err(|_| invalid("formatted scalar is not UTF-8"))
    }
}

impl Write for Text {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::timestamp;
    use inventory_core::record::Time;

    #[test]
    fn timestamps_follow_query_wire_formats_including_negative_fractional_epochs()
    -> Result<(), Box<dyn std::error::Error>> {
        let time = Time::new(1_422_172_800, 0)?;
        let date_time = timestamp(time, None)?;
        assert_eq!(date_time.as_str().ok(), Some("2015-01-25T08:00:00Z"));

        let http_date = timestamp(time, Some("http-date"))?;
        assert_eq!(
            http_date.as_str().ok(),
            Some("Sun, 25 Jan 2015 08:00:00 GMT")
        );

        let negative = Time::new(-1, 500_000_000)?;
        let epoch = timestamp(negative, Some("epoch-seconds"))?;
        assert_eq!(epoch.as_str().ok(), Some("-0.5"));
        Ok(())
    }
}
