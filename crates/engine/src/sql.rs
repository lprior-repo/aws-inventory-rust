mod bounds;
mod policy;
mod reader;
mod run;
mod value;

#[cfg(test)]
mod test_peer;

pub use bounds::{Bounds, Budget};
pub use policy::validate;
pub use reader::ReaderUrl;
pub use run::{SqlResult, run};
pub use value::SqlValue;

pub const MAX_SQL_BYTES: usize = crate::store::MAX_SQL_BYTES;
pub(crate) use policy::{Dialect, validate_with};

#[derive(Debug, thiserror::Error)]
pub enum SqlError {
    #[error("SQL denied: {0}")]
    QueryDenied(&'static str),
    #[error("SQL result exceeds {bound} bound")]
    ResultTooLarge { bound: &'static str },
    #[error("column {column:?} has unsupported PostgreSQL type {type_name:?}; cast to text")]
    UnsupportedType { column: String, type_name: String },
    #[error("SQL requires a reader role with no inventory write or administrative privileges")]
    ReaderRequired,
    #[error("SQL requires PostgreSQL setting {name} = {required}")]
    SettingRequired {
        name: &'static str,
        required: &'static str,
    },
    #[error("invalid PostgreSQL reader URL")]
    InvalidUrl,
    #[error("SQL bounds must be positive and no greater than the defaults")]
    InvalidBounds,
    #[error("SQL connection or transaction step exceeded time bound")]
    Timeout,
    #[error("SQL query exceeded time bound")]
    StatementTimeout,
    #[error("invalid PostgreSQL value: {0}")]
    InvalidValue(&'static str),
    #[error("SQL result allocation failed")]
    Allocation,
    #[error("PostgreSQL query failed: {0}")]
    Database(#[from] sqlx::Error),
}
