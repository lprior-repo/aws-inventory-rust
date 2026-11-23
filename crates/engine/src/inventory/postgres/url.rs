use crate::inventory::StoreError;
use sqlx::postgres::PgConnectOptions;
use std::fmt;

#[derive(Clone)]
pub struct DatabaseUrl(String);

impl DatabaseUrl {
    pub fn parse(value: &str) -> Result<Self, StoreError> {
        if value.len() > 8_192
            || !(value.starts_with("postgres://") || value.starts_with("postgresql://"))
        {
            return Err(StoreError::Invalid("invalid PostgreSQL database URL"));
        }
        parse_options(value)?;
        Ok(Self(value.to_owned()))
    }

    pub(super) fn options(&self) -> Result<PgConnectOptions, StoreError> {
        parse_options(&self.0)
    }
}

impl fmt::Display for DatabaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("postgresql://***")
    }
}

impl fmt::Debug for DatabaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DatabaseUrl(postgresql://***)")
    }
}

fn parse_options(value: &str) -> Result<PgConnectOptions, StoreError> {
    value
        .parse()
        .map_err(|_| StoreError::Invalid("invalid PostgreSQL database URL"))
}
