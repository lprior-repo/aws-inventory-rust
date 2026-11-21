use inventory_core::{catalog::TimestampFormat, record::Time};

pub(super) fn parse(value: &str, format: Option<TimestampFormat>) -> Option<Time> {
    match format {
        Some(TimestampFormat::DateTime) => datetime(value),
        Some(TimestampFormat::HttpDate) => http_date(value),
        Some(TimestampFormat::EpochSeconds) | None => epoch(value),
    }
}

fn epoch(value: &str) -> Option<Time> {
    let (mantissa, exponent) = match value.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i64>().ok()?),
        None => (value, 0),
    };
    let precision = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let coefficient = mantissa.replace('.', "").parse::<i128>().ok()?;
    let scale = exponent
        .checked_add(9)?
        .checked_sub(i64::try_from(precision).ok()?)?;
    let total = scale_nanos(coefficient, scale)?;
    Time::new(
        i64::try_from(total.div_euclid(1_000_000_000)).ok()?,
        u32::try_from(total.rem_euclid(1_000_000_000)).ok()?,
    )
    .ok()
}

fn scale_nanos(coefficient: i128, scale: i64) -> Option<i128> {
    if scale >= 0 {
        coefficient.checked_mul(10i128.checked_pow(u32::try_from(scale).ok()?)?)
    } else {
        let divisor = 10i128.checked_pow(u32::try_from(scale.checked_neg()?).ok()?)?;
        let rounded = coefficient
            .checked_abs()?
            .checked_add(divisor / 2)?
            .checked_div(divisor)?;
        if coefficient < 0 {
            rounded.checked_neg()
        } else {
            Some(rounded)
        }
    }
}

fn datetime(value: &str) -> Option<Time> {
    let (date, time_zone) = value.split_once('T')?;
    let (year, month, day) = date_parts(date)?;
    let (time, offset) = zone(time_zone)?;
    let (hour, minute, second, nanos) = time_parts(time)?;
    let seconds = days(year, month, day)?
        .checked_mul(86_400)?
        .checked_add(clock_seconds(hour, minute, second)?)?
        .checked_sub(offset)?;
    Time::new(seconds, nanos).ok()
}

fn clock_seconds(hour: i64, minute: i64, second: i64) -> Option<i64> {
    hour.checked_mul(3_600)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)
}

fn date_parts(value: &str) -> Option<(i64, i64, i64)> {
    let mut parts = value.split('-');
    let result = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(result)
}

fn zone(value: &str) -> Option<(&str, i64)> {
    if let Some(time) = value.strip_suffix('Z') {
        return Some((time, 0));
    }
    let index = value.rfind(['+', '-'])?;
    let sign = if value.as_bytes().get(index)? == &b'+' {
        1
    } else {
        -1
    };
    let time = value.get(..index)?;
    let zone = value.get(index.checked_add(1)?..)?;
    let (hours, minutes) = zone.split_once(':')?;
    let offset = hours
        .parse::<i64>()
        .ok()?
        .checked_mul(3_600)?
        .checked_add(minutes.parse::<i64>().ok()?.checked_mul(60)?)?
        .checked_mul(sign)?;
    Some((time, offset))
}

fn time_parts(value: &str) -> Option<(i64, i64, i64, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let seconds = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let (second, fraction) = seconds
        .split_once('.')
        .map_or((seconds, None), |(s, f)| (s, Some(f)));
    let second = second.parse().ok()?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second, fraction_nanos(fraction)?))
}

fn fraction_nanos(value: Option<&str>) -> Option<u32> {
    let Some(value) = value else {
        return Some(0);
    };
    if value.is_empty() || value.len() > 9 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value
        .parse::<u32>()
        .ok()?
        .checked_mul(10u32.checked_pow(u32::try_from(9usize.checked_sub(value.len())?).ok()?)?)
}

fn http_date(value: &str) -> Option<Time> {
    let mut parts = value.split_ascii_whitespace();
    parts.next()?;
    let day = parts.next()?.parse().ok()?;
    let month = match parts.next()? {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year = parts.next()?.parse().ok()?;
    let (hour, minute, second) = clock(parts.next()?)?;
    if parts.next() != Some("GMT") || parts.next().is_some() {
        return None;
    }
    let seconds = days(year, month, day)?
        .checked_mul(86_400)?
        .checked_add(clock_seconds(hour, minute, second)?)?;
    Time::new(seconds, 0).ok()
}

fn clock(value: &str) -> Option<(i64, i64, i64)> {
    let mut parts = value.split(':');
    let result = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    if parts.next().is_some() || result.0 > 23 || result.1 > 59 || result.2 > 59 {
        return None;
    }
    Some(result)
}

fn days(year: i64, month: i64, day: i64) -> Option<i64> {
    if !(1..=12).contains(&month) || day < 1 || day > month_days(year, month) {
        return None;
    }
    let adjusted = year.checked_sub(i64::from(month <= 2))?;
    let era = adjusted.div_euclid(400);
    let year_of_era = adjusted.checked_sub(era.checked_mul(400)?)?;
    let shifted_month = month.checked_add(if month > 2 { -3 } else { 9 })?;
    let day_of_year = 153i64
        .checked_mul(shifted_month)?
        .checked_add(2)?
        .div_euclid(5)
        .checked_add(day)?
        .checked_sub(1)?;
    let day_of_era = year_of_era
        .checked_mul(365)?
        .checked_add(year_of_era.div_euclid(4))?
        .checked_sub(year_of_era.div_euclid(100))?
        .checked_add(day_of_year)?;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
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
