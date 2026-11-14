use super::{DecodeError, Time, invalid};

struct Decimal<'a> {
    negative: bool,
    integer: &'a str,
    fraction: &'a str,
    exponent: i64,
}

pub(super) fn parse(value: &str) -> Result<Time, DecodeError> {
    let number = parts(value)?;
    let digits = number.integer.bytes().chain(number.fraction.bytes());
    let Some(leading) = digits.clone().position(|digit| digit != b'0') else {
        return Time::new(0, 0).map_err(|_| invalid());
    };
    let point = i64::try_from(number.integer.len())
        .map_err(|_| invalid())?
        .saturating_add(number.exponent)
        .saturating_sub(i64::try_from(leading).map_err(|_| invalid())?);
    if point > 19 {
        return Err(invalid());
    }
    if point < -9 {
        return Time::new(0, 0).map_err(|_| invalid());
    }
    let magnitude = magnitude(digits.skip(leading), point)?;
    let total = if number.negative {
        magnitude.checked_neg().ok_or_else(invalid)?
    } else {
        magnitude
    };
    instant(total)
}

fn instant(total: i128) -> Result<Time, DecodeError> {
    Time::new(
        i64::try_from(total.div_euclid(1_000_000_000)).map_err(|_| invalid())?,
        u32::try_from(total.rem_euclid(1_000_000_000)).map_err(|_| invalid())?,
    )
    .map_err(|_| invalid())
}

fn parts(value: &str) -> Result<Decimal<'_>, DecodeError> {
    let (negative, value) = match value.strip_prefix('-') {
        Some(value) => (true, value),
        None => (false, value.strip_prefix('+').unwrap_or(value)),
    };
    let (mantissa, power) = match value.split_once(['e', 'E']) {
        Some((mantissa, power)) => (mantissa, exponent(power)?),
        None => (value, 0),
    };
    let (integer, fraction) = components(mantissa)?;
    Ok(Decimal {
        negative,
        integer,
        fraction,
        exponent: power,
    })
}

fn components(value: &str) -> Result<(&str, &str), DecodeError> {
    let (integer, fraction) = match value.split_once('.') {
        Some((integer, fraction)) if valid_digits(fraction) => (integer, fraction),
        Some(_) => return Err(invalid()),
        None => (value, ""),
    };
    if !valid_digits(integer) {
        return Err(invalid());
    }
    Ok((integer, fraction))
}

fn valid_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn exponent(value: &str) -> Result<i64, DecodeError> {
    let digits = value
        .strip_prefix('-')
        .or_else(|| value.strip_prefix('+'))
        .unwrap_or(value);
    if !valid_digits(digits) {
        return Err(invalid());
    }
    Ok(value.parse().unwrap_or(if value.starts_with('-') {
        i64::MIN
    } else {
        i64::MAX
    }))
}

fn magnitude(mut digits: impl Iterator<Item = u8>, point: i64) -> Result<i128, DecodeError> {
    let whole = integer(&mut digits, point)?;
    let nanos = fraction(&mut digits, point)?;
    let mut total = whole
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(nanos))
        .ok_or_else(invalid)?;
    if point.saturating_add(9) >= 0 && digits.next().unwrap_or(b'0') >= b'5' {
        total = total.checked_add(1).ok_or_else(invalid)?;
    }
    Ok(total)
}

fn integer(digits: &mut impl Iterator<Item = u8>, point: i64) -> Result<i128, DecodeError> {
    let mut whole = 0;
    for _ in 0..point.max(0) {
        whole = shifted(whole, digits.next().unwrap_or(b'0'))?;
    }
    Ok(whole)
}

fn fraction(digits: &mut impl Iterator<Item = u8>, point: i64) -> Result<i128, DecodeError> {
    let mut nanos = 0;
    for place in 0..9i64 {
        let digit = if point.saturating_add(place) >= 0 {
            digits.next().unwrap_or(b'0')
        } else {
            b'0'
        };
        nanos = shifted(nanos, digit)?;
    }
    Ok(nanos)
}

fn shifted(value: i128, digit: u8) -> Result<i128, DecodeError> {
    let digit = digit.checked_sub(b'0').ok_or_else(invalid)?;
    value
        .checked_mul(10)
        .and_then(|value| value.checked_add(i128::from(digit)))
        .ok_or_else(invalid)
}
