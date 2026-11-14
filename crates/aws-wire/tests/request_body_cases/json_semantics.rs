use serde_json::{Number, Value};

const MAX_ITEMS: usize = 100_000;

pub(super) fn equivalent(actual: &Value, expected: &Value) -> bool {
    let mut pending = vec![(actual, expected)];
    let mut remaining = MAX_ITEMS;
    while let Some((left, right)) = pending.pop() {
        let Some(next) = remaining.checked_sub(1) else {
            return false;
        };
        remaining = next;
        if !compare(left, right, &mut pending) || pending.len() > MAX_ITEMS {
            return false;
        }
    }
    true
}

fn compare<'a>(
    left: &'a Value,
    right: &'a Value,
    pending: &mut Vec<(&'a Value, &'a Value)>,
) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => numbers_equal(left, right),
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            pending.extend(left.iter().zip(right));
            true
        }
        (Value::Object(left), Value::Object(right)) if left.len() == right.len() => {
            for (key, value) in left {
                let Some(other) = right.get(key) else {
                    return false;
                };
                pending.push((value, other));
            }
            true
        }
        (Value::Null, Value::Null)
        | (Value::Bool(_), Value::Bool(_))
        | (Value::String(_), Value::String(_)) => left == right,
        _ => false,
    }
}

struct Decimal<'a> {
    negative: bool,
    digits: &'a str,
    exponent: i128,
}

fn numbers_equal(left: &Number, right: &Number) -> bool {
    if left == right {
        return true;
    }
    let left = left.to_string();
    let right = right.to_string();
    let (Some(left), Some(right)) = (decimal(&left), decimal(&right)) else {
        return false;
    };
    left.negative == right.negative
        && left.exponent == right.exponent
        && left
            .digits
            .bytes()
            .filter(|byte| *byte != b'.')
            .eq(right.digits.bytes().filter(|byte| *byte != b'.'))
}

fn decimal(text: &str) -> Option<Decimal<'_>> {
    let negative = text.starts_with('-');
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
    let digits = mantissa
        .trim_start_matches(['0', '.'])
        .trim_end_matches(['0', '.']);
    if digits.is_empty() {
        return Some(Decimal {
            negative: false,
            digits,
            exponent: 0,
        });
    }
    let fraction = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let zeros = mantissa
        .bytes()
        .rev()
        .filter(|byte| *byte != b'.')
        .take_while(|byte| *byte == b'0')
        .count();
    let exponent = exponent
        .parse::<i128>()
        .ok()?
        .checked_sub(i128::try_from(fraction).ok()?)?
        .checked_add(i128::try_from(zeros).ok()?)?;
    Some(Decimal {
        negative,
        digits,
        exponent,
    })
}

#[cfg(test)]
mod tests {
    use super::equivalent;

    #[test]
    fn accepts_equivalent_decimals_without_rounding_large_integers() -> Result<(), serde_json::Error>
    {
        for (left, right) in [
            ("1", "1.0"),
            ("1e3", "1000.00"),
            ("0.0012300", "123e-5"),
            ("-0.0", "0"),
            ("-100.10", "-1001e-1"),
            ("9007199254740993", "9007199254740993.0"),
            ("18446744073709551616", "184467440737095516160e-1"),
        ] {
            assert!(
                equivalent(&serde_json::from_str(left)?, &serde_json::from_str(right)?),
                "{left} != {right}"
            );
        }
        Ok(())
    }

    #[test]
    fn rejects_precision_loss_and_number_kind_or_sign_changes() -> Result<(), serde_json::Error> {
        for (left, right) in [
            ("9007199254740993", "9007199254740992.0"),
            ("18446744073709551616", "18446744073709551617"),
            ("1", "\"1\""),
            ("1", "-1"),
            ("1.01", "1.001"),
            ("1e30", "1e31"),
        ] {
            assert!(
                !equivalent(&serde_json::from_str(left)?, &serde_json::from_str(right)?),
                "{left} == {right}"
            );
        }
        Ok(())
    }

    #[test]
    fn preserves_nested_keys_array_order_and_value_kinds() -> Result<(), serde_json::Error> {
        let expected = serde_json::from_str(r#"{"values":[1,2],"object":{"x":true}}"#)?;
        let same = serde_json::from_str(r#"{"object":{"x":true},"values":[1.0,20e-1]}"#)?;
        assert!(equivalent(&expected, &same));
        for changed in [
            r#"{"values":[2,1],"object":{"x":true}}"#,
            r#"{"values":[1,2],"object":{"y":true}}"#,
            r#"{"values":[1,2],"object":{"x":"true"}}"#,
            r#"{"values":[1,2,3],"object":{"x":true}}"#,
        ] {
            assert!(!equivalent(&expected, &serde_json::from_str(changed)?));
        }
        Ok(())
    }
}
