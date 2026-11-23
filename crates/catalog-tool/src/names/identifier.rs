use super::{MAX_DERIVED_SQL_NAME_BYTES, NameError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SqlNameReason {
    #[error("expected [a-z][a-z0-9_]* with no empty words")]
    InvalidSyntax,

    #[error("expected at least two words")]
    TooFewWords,

    #[error("the sqlite_ prefix is reserved")]
    ReservedPrefix,

    #[error("reserved PostgreSQL or SQLite keyword")]
    ReservedKeyword,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SqlName(String);

impl SqlName {
    pub(super) fn new(name: String, type_name: &str) -> Result<Self, NameError> {
        if name.len() > MAX_DERIVED_SQL_NAME_BYTES {
            return Err(NameError::TooLong {
                type_name: type_name.to_owned(),
                limit: MAX_DERIVED_SQL_NAME_BYTES,
            });
        }
        if let Some(reason) = invalid_reason(&name) {
            return Err(NameError::InvalidSqlName { name, reason });
        }
        inventory_core::inventory::shorten_sql_name(name)
            .map(Self)
            .map_err(|_| NameError::InvalidSqlName {
                name: type_name.to_owned(),
                reason: SqlNameReason::InvalidSyntax,
            })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SqlName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

fn invalid_reason(name: &str) -> Option<SqlNameReason> {
    let valid_initial = name.starts_with(|ch: char| ch.is_ascii_lowercase());
    let valid_words = name.split('_').all(|word| {
        !word.is_empty()
            && word
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    });
    if !valid_initial || !valid_words {
        return Some(SqlNameReason::InvalidSyntax);
    }
    if super::keywords::is_reserved(name) {
        return Some(SqlNameReason::ReservedKeyword);
    }

    if name.split('_').count() < 2 {
        return Some(SqlNameReason::TooFewWords);
    }
    if name.starts_with("sqlite_") {
        return Some(SqlNameReason::ReservedPrefix);
    }
    None
}
