use super::resources::Prepared;
use crate::inventory::StoreError;
use inventory_core::inventory::{Column, Observation, ScalarType};
use inventory_core::record::ValueRef;
use sqlx::{Postgres, QueryBuilder, postgres::types::PgInterval};

pub(super) enum Array<'a> {
    Bool(Vec<Option<bool>>),
    Integer(Vec<Option<i64>>),
    Real(Vec<Option<f64>>),
    Text(Vec<Option<&'a str>>),
    Blob(Vec<Option<&'a [u8]>>),
    Timestamp(Vec<Option<PgInterval>>),
}

impl<'a> Array<'a> {
    pub(super) fn new(rows: &[&'a Prepared], column: &Column) -> Result<Self, StoreError> {
        match column.scalar() {
            ScalarType::Bool => collect(rows, column, boolean).map(Self::Bool),
            ScalarType::Integer => collect(rows, column, integer).map(Self::Integer),
            ScalarType::Real => collect(rows, column, real).map(Self::Real),
            ScalarType::Text => collect(rows, column, text).map(Self::Text),
            ScalarType::Blob => collect(rows, column, blob).map(Self::Blob),
            ScalarType::Timestamp => collect(rows, column, timestamp).map(Self::Timestamp),
        }
    }

    pub(super) fn bind(self, query: &mut QueryBuilder<Postgres>) {
        match self {
            Self::Bool(values) => query.push_bind(values).push("::boolean[]"),
            Self::Integer(values) => query.push_bind(values).push("::bigint[]"),
            Self::Real(values) => query.push_bind(values).push("::double precision[]"),
            Self::Text(values) => query.push_bind(values).push("::text[]"),
            Self::Blob(values) => query.push_bind(values).push("::bytea[]"),
            Self::Timestamp(values) => query.push_bind(values).push("::interval[]"),
        };
    }
}

fn collect<'a, T>(
    rows: &[&'a Prepared],
    column: &Column,
    parse: impl Fn(ValueRef<'a>) -> Result<T, StoreError>,
) -> Result<Vec<Option<T>>, StoreError> {
    rows.iter()
        .map(|row| match value(&row.aggregate.observation, column)? {
            None | Some(ValueRef::Null) => Ok(None),
            Some(value) => parse(value).map(Some),
        })
        .collect()
}

fn value<'a>(row: &'a Observation, column: &Column) -> Result<Option<ValueRef<'a>>, StoreError> {
    if column.path().is_empty() {
        return Ok(inventory_core::record::scalar_item(row.record().root())?);
    }
    column.path().iter().try_fold(
        Some(ValueRef::Map(row.record().root())),
        |current, field| match current {
            Some(ValueRef::Map(map)) => Ok(map.get_field(*field)?),
            _ => Ok(None),
        },
    )
}

fn mismatch() -> StoreError {
    StoreError::Invalid("record scalar does not match column schema")
}

fn boolean(value: ValueRef<'_>) -> Result<bool, StoreError> {
    match value {
        ValueRef::Bool(value) => Ok(value),
        _ => Err(mismatch()),
    }
}

fn integer(value: ValueRef<'_>) -> Result<i64, StoreError> {
    match value {
        ValueRef::Int(value) => Ok(value),
        _ => Err(mismatch()),
    }
}

fn real(value: ValueRef<'_>) -> Result<f64, StoreError> {
    match value {
        ValueRef::Float(value) => Ok(value.to_f64()),
        _ => Err(mismatch()),
    }
}

fn text(value: ValueRef<'_>) -> Result<&str, StoreError> {
    match value {
        ValueRef::Text(value) => Ok(value),
        _ => Err(mismatch()),
    }
}

fn blob(value: ValueRef<'_>) -> Result<&[u8], StoreError> {
    match value {
        ValueRef::Bytes(value) => Ok(value),
        _ => Err(mismatch()),
    }
}

fn timestamp(value: ValueRef<'_>) -> Result<PgInterval, StoreError> {
    let ValueRef::Time(value) = value else {
        return Err(mismatch());
    };
    let microseconds = value
        .seconds()
        .checked_mul(1_000_000)
        .and_then(|seconds| seconds.checked_add(i64::from(value.nanoseconds() / 1_000)))
        .ok_or(StoreError::Invalid(
            "timestamp exceeds PostgreSQL microsecond range",
        ))?;
    Ok(PgInterval {
        months: 0,
        days: 0,
        microseconds,
    })
}
