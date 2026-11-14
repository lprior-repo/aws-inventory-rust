use super::{MAX_SQL_NAME_BYTES, NameError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SqlNameReason {
    #[error("expected [a-z][a-z0-9_]* with no empty words")]
    InvalidSyntax,

    #[error("expected at least two words")]
    TooFewWords,

    #[error("the sqlite_ prefix is reserved")]
    ReservedPrefix,

    #[error("reserved SQLite keyword")]
    ReservedKeyword,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SqlName(String);

impl SqlName {
    pub(super) fn new(name: String, type_name: &str) -> Result<Self, NameError> {
        if name.len() > MAX_SQL_NAME_BYTES {
            return Err(NameError::TooLong {
                type_name: type_name.to_owned(),
                limit: MAX_SQL_NAME_BYTES,
            });
        }
        if let Some(reason) = invalid_reason(&name) {
            return Err(NameError::InvalidSqlName { name, reason });
        }
        Ok(Self(name))
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

    if name.split('_').count() < 2 {
        return Some(SqlNameReason::TooFewWords);
    }
    if name.starts_with("sqlite_") {
        return Some(SqlNameReason::ReservedPrefix);
    }
    if matches!(name, "current_date" | "current_time" | "current_timestamp") {
        return Some(SqlNameReason::ReservedKeyword);
    }
    None
}
