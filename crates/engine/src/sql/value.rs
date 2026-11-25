use super::{Budget, SqlError};
use sqlx::{
    Column, Row, TypeInfo, ValueRef,
    postgres::{PgRow, PgValueFormat},
};
use std::io::{Cursor, Write};

#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    Null,
    Bool(bool),
    Int(i64),
    Real(f64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp { seconds: i64, micros: u32 },
    Json(String),
}

pub(super) fn decode(row: &PgRow, index: usize, budget: &mut Budget) -> Result<SqlValue, SqlError> {
    let column = row
        .columns()
        .get(index)
        .ok_or(SqlError::InvalidValue("missing column"))?;
    let raw = row.try_get_raw(index)?;
    let name = column.type_info().name();
    if !matches!(
        name,
        "BOOL"
            | "INT2"
            | "INT4"
            | "INT8"
            | "FLOAT4"
            | "FLOAT8"
            | "TEXT"
            | "VARCHAR"
            | "NAME"
            | "BYTEA"
            | "TIMESTAMPTZ"
            | "JSONB"
    ) {
        return Err(SqlError::UnsupportedType {
            column: column.name().to_owned(),
            type_name: name.to_owned(),
        });
    }
    if raw.is_null() {
        return Ok(SqlValue::Null);
    }
    if raw.format() != PgValueFormat::Binary {
        return Err(SqlError::InvalidValue("expected binary protocol"));
    }
    let bytes = raw
        .as_bytes()
        .map_err(|_| SqlError::InvalidValue("missing binary payload"))?;
    budget.charge_bytes(bytes.len())?;
    decode_binary(name, bytes)
}

fn decode_binary(name: &str, bytes: &[u8]) -> Result<SqlValue, SqlError> {
    match name {
        "BOOL" => match bytes {
            [0] => Ok(SqlValue::Bool(false)),
            [1] => Ok(SqlValue::Bool(true)),
            _ => Err(SqlError::InvalidValue("bool")),
        },
        "INT2" => Ok(SqlValue::Int(i64::from(i16::from_be_bytes(array(bytes)?)))),
        "INT4" => Ok(SqlValue::Int(i64::from(i32::from_be_bytes(array(bytes)?)))),
        "INT8" => Ok(SqlValue::Int(i64::from_be_bytes(array(bytes)?))),
        "FLOAT4" => Ok(SqlValue::Real(float4(f32::from_be_bytes(array(bytes)?))?)),
        "FLOAT8" => Ok(SqlValue::Real(f64::from_be_bytes(array(bytes)?))),
        "TEXT" | "VARCHAR" | "NAME" => Ok(SqlValue::Text(text(bytes)?)),
        "BYTEA" => {
            let mut owned = Vec::new();
            owned
                .try_reserve_exact(bytes.len())
                .map_err(|_| SqlError::Allocation)?;
            owned.extend_from_slice(bytes);
            Ok(SqlValue::Bytes(owned))
        }
        "TIMESTAMPTZ" => timestamp(i64::from_be_bytes(array(bytes)?)),
        "JSONB" => match bytes.split_first() {
            Some((1, json)) => Ok(SqlValue::Json(text(json)?)),
            _ => Err(SqlError::InvalidValue("jsonb version")),
        },
        _ => Err(SqlError::InvalidValue("unexpected type")),
    }
}

fn float4(value: f32) -> Result<f64, SqlError> {
    // f32's shortest decimal fits in 64 bytes, including sign and subnormal zeros.
    let mut decimal = Cursor::new([0_u8; 64]);
    write!(&mut decimal, "{value}").map_err(|_| SqlError::InvalidValue("float4 decimal"))?;
    let length = usize::try_from(decimal.position())
        .map_err(|_| SqlError::InvalidValue("float4 decimal length"))?;
    let bytes = decimal
        .get_ref()
        .get(..length)
        .ok_or(SqlError::InvalidValue("float4 decimal length"))?;
    std::str::from_utf8(bytes)
        .map_err(|_| SqlError::InvalidValue("float4 decimal UTF-8"))?
        .parse()
        .map_err(|_| SqlError::InvalidValue("float4 decimal"))
}

fn array<const N: usize>(bytes: &[u8]) -> Result<[u8; N], SqlError> {
    bytes
        .try_into()
        .map_err(|_| SqlError::InvalidValue("binary length"))
}

fn text(bytes: &[u8]) -> Result<String, SqlError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SqlError::InvalidValue("UTF-8"))?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(text.len())
        .map_err(|_| SqlError::Allocation)?;
    owned.push_str(text);
    Ok(owned)
}

fn timestamp(micros: i64) -> Result<SqlValue, SqlError> {
    if matches!(micros, i64::MIN | i64::MAX) {
        return Err(SqlError::InvalidValue("infinite timestamp; cast to text"));
    }
    let seconds = micros
        .div_euclid(1_000_000)
        .checked_add(946_684_800)
        .ok_or(SqlError::InvalidValue("timestamp overflow"))?;
    let micros = u32::try_from(micros.rem_euclid(1_000_000))
        .map_err(|_| SqlError::InvalidValue("timestamp fraction"))?;
    Ok(SqlValue::Timestamp { seconds, micros })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_float4_when_decoding_then_shortest_decimal_is_preserved()
    -> Result<(), Box<dyn std::error::Error>> {
        for (single, expected) in [
            (1.1_f32, 1.1_f64),
            (-1.1, -1.1),
            (0.0, 0.0),
            (-0.0, -0.0),
            (f32::MAX, 3.402_823_5e38),
            (f32::MIN, -3.402_823_5e38),
            (f32::MIN_POSITIVE, 1.175_494_4e-38),
            (f32::from_bits(1), 1e-45),
            (f32::INFINITY, f64::INFINITY),
            (f32::NEG_INFINITY, f64::NEG_INFINITY),
        ] {
            assert_eq!(float4(single)?.to_bits(), expected.to_bits());
        }
        assert!(float4(f32::NAN)?.is_nan());
        Ok(())
    }

    #[test]
    fn given_infinite_timestamps_when_decoding_then_cast_guidance_is_explicit()
    -> Result<(), Box<dyn std::error::Error>> {
        for micros in [i64::MIN, i64::MAX] {
            assert_eq!(
                timestamp(micros)
                    .err()
                    .ok_or("expected an error")?
                    .to_string(),
                "invalid PostgreSQL value: infinite timestamp; cast to text"
            );
        }
        Ok(())
    }

    #[test]
    fn given_malformed_binary_values_when_decoding_then_exact_errors_are_returned()
    -> Result<(), Box<dyn std::error::Error>> {
        let cases: &[(&str, &[u8], &str)] = &[
            ("BOOL", &[2], "bool"),
            ("BOOL", &[255], "bool"),
            ("BOOL", &[], "bool"),
            ("JSONB", &[0, b'{', b'}'], "jsonb version"),
            ("JSONB", &[2, b'{', b'}'], "jsonb version"),
            ("JSONB", &[], "jsonb version"),
            ("INT2", &[0], "binary length"),
            ("INT2", &[0; 3], "binary length"),
            ("INT4", &[0; 3], "binary length"),
            ("INT4", &[0; 5], "binary length"),
            ("INT8", &[0; 7], "binary length"),
            ("INT8", &[0; 9], "binary length"),
        ];
        for (name, bytes, reason) in cases {
            let error = decode_binary(name, bytes)
                .err()
                .ok_or("expected an error")?;
            assert_eq!(
                error.to_string(),
                format!("invalid PostgreSQL value: {reason}"),
                "{name}"
            );
            assert!(matches!(error, SqlError::InvalidValue(actual) if actual == *reason));
        }
        Ok(())
    }
}
