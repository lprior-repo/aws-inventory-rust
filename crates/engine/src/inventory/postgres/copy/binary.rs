use crate::inventory::StoreError;
use inventory_core::record::Time;

pub(super) const HEADER: &[u8; 19] = b"PGCOPY\n\xff\r\n\0\0\0\0\0\0\0\0\0";
pub(super) const TRAILER: [u8; 2] = (-1_i16).to_be_bytes();

#[derive(Clone, Copy)]
pub(in crate::inventory::postgres) enum Field<'a> {
    Null,
    Text(&'a str),
    Bigint(i64),
    Double(f64),
    Boolean(bool),
    Bytes(&'a [u8]),
    Timestamp(Time),
    Jsonb(Option<&'a str>),
}

pub(in crate::inventory::postgres) fn row(
    buffer: &mut Vec<u8>,
    fields: usize,
) -> Result<(), StoreError> {
    let fields = i16::try_from(fields)
        .map_err(|_| StoreError::Invalid("too many PostgreSQL COPY columns"))?;
    reserve(buffer, size_of::<i16>())?;
    buffer.extend_from_slice(&fields.to_be_bytes());
    Ok(())
}

pub(in crate::inventory::postgres) fn field(
    buffer: &mut Vec<u8>,
    value: Field<'_>,
) -> Result<(), StoreError> {
    match value {
        Field::Null | Field::Jsonb(None) => {
            reserve(buffer, size_of::<i32>())?;
            buffer.extend_from_slice(&(-1_i32).to_be_bytes());
            Ok(())
        }
        Field::Text(value) => payload(buffer, &[], value.as_bytes()),
        Field::Bigint(value) => payload(buffer, &[], &value.to_be_bytes()),
        Field::Double(value) => payload(buffer, &[], &value.to_be_bytes()),
        Field::Boolean(value) => payload(buffer, &[], &[u8::from(value)]),
        Field::Bytes(value) => payload(buffer, &[], value),
        Field::Timestamp(value) => payload(buffer, &[], &timestamp(value)?.to_be_bytes()),
        Field::Jsonb(Some(value)) => payload(buffer, &[1], value.as_bytes()),
    }
}

fn payload(buffer: &mut Vec<u8>, prefix: &[u8], bytes: &[u8]) -> Result<(), StoreError> {
    const TOO_LONG: StoreError = StoreError::Invalid("PostgreSQL COPY field exceeds length limit");
    let bytes_len = prefix.len().checked_add(bytes.len()).ok_or(TOO_LONG)?;
    let length = i32::try_from(bytes_len).map_err(|_| TOO_LONG)?;
    let additional = bytes_len.checked_add(size_of::<i32>()).ok_or(TOO_LONG)?;
    reserve(buffer, additional)?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(prefix);
    buffer.extend_from_slice(bytes);
    Ok(())
}

pub(super) fn reserve(buffer: &mut Vec<u8>, additional: usize) -> Result<(), StoreError> {
    buffer
        .try_reserve(additional)
        .map_err(|_| StoreError::Invalid("PostgreSQL COPY buffer exceeds capacity"))
}

fn timestamp(value: Time) -> Result<i64, StoreError> {
    const POSTGRES_EPOCH_UNIX_SECONDS: i64 = 946_684_800;
    i128::from(value.seconds())
        .checked_sub(i128::from(POSTGRES_EPOCH_UNIX_SECONDS))
        .and_then(|seconds| seconds.checked_mul(1_000_000))
        .and_then(|micros| micros.checked_add(i128::from(value.nanoseconds() / 1_000)))
        .and_then(|micros| i64::try_from(micros).ok())
        // PostgreSQL uses the two extremes as infinity sentinels, not finite times.
        .filter(|micros| !matches!(*micros, i64::MIN | i64::MAX))
        .ok_or(StoreError::Invalid(
            "timestamp exceeds PostgreSQL microsecond range",
        ))
}
