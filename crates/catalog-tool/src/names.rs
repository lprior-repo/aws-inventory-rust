use thiserror::Error;

mod exceptions;
mod identifier;
mod output;
mod render;
mod words;
pub use exceptions::Exceptions;
pub use identifier::{SqlName, SqlNameReason};
use output::Output;
pub use render::render_names;
use words::append_plain;

pub const MAX_SQL_NAME_BYTES: usize = 128;

pub const MAX_TYPE_NAME_BYTES: usize = 256;

pub const MAX_EXCEPTIONS_BYTES: usize = 65_536;

pub const MAX_EXCEPTIONS_ROWS: usize = 1_024;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum NameError {
    #[error("invalid type name: {name}")]
    InvalidType { name: String },
    #[error("SQL name for {type_name} exceeds {limit} bytes")]
    TooLong { type_name: String, limit: usize },
    #[error("invalid SQL name {name}: {reason}")]
    InvalidSqlName { name: String, reason: SqlNameReason },
    #[error("type name is {length} bytes; limit is {limit}")]
    TypeTooLong { length: usize, limit: usize },
    #[error("exceptions file is {length} bytes; limit is {limit}")]
    ExceptionsTooLarge { length: usize, limit: usize },
    #[error("exceptions file has {rows} rows; limit is {limit}")]
    TooManyExceptions { rows: usize, limit: usize },
    #[error("invalid exception at line {line}")]
    InvalidException { line: usize },
    #[error("duplicate exception at line {line}")]
    DuplicateException { line: usize },
    #[error("SQL name {name} is shared by {first_type} and {second_type}")]
    DuplicateSqlName {
        name: SqlName,
        first_type: String,
        second_type: String,
    },
    #[error("unused exception {spelling} at line {line}")]
    UnusedException { line: usize, spelling: String },
}

fn is_valid_part(part: &str) -> bool {
    part.starts_with(|ch: char| ch.is_ascii_alphabetic())
        && part.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn validate_type_name(type_name: &str) -> Result<(), NameError> {
    if type_name.len() > MAX_TYPE_NAME_BYTES {
        return Err(NameError::TypeTooLong {
            length: type_name.len(),
            limit: MAX_TYPE_NAME_BYTES,
        });
    }
    if !type_name.contains("::") || !type_name.split("::").all(is_valid_part) {
        return Err(NameError::InvalidType {
            name: type_name.to_owned(),
        });
    }
    Ok(())
}

fn next_piece<'a>(
    part: &'a str,
    exceptions: &'a Exceptions,
) -> Option<(&'a str, &'a str, Option<usize>)> {
    if part.is_empty() {
        return None;
    }
    part.char_indices()
        .find_map(|(position, _)| {
            let (prefix, suffix) = part.split_at_checked(position)?;
            let (index, spelling, replacement) = exceptions.longest_match(suffix)?;
            if prefix.is_empty() {
                suffix
                    .strip_prefix(spelling)
                    .map(|tail| (replacement, tail, Some(index)))
            } else {
                Some((prefix, suffix, None))
            }
        })
        .or(Some((part, "", None)))
}

fn append_part(
    part: &str,
    exceptions: &Exceptions,
    used: &mut [bool],
    output: &mut Output<'_>,
) -> Result<(), NameError> {
    let mut remaining = part;
    std::iter::from_fn(|| {
        let (piece, tail, index) = next_piece(remaining, exceptions)?;
        remaining = tail;
        Some((piece, index))
    })
    .try_for_each(|(piece, index)| match index {
        Some(index) => {
            if let Some(flag) = used.get_mut(index) {
                *flag = true;
            }
            output.separator()?;
            output.extend(piece)
        }
        None => append_plain(piece, output),
    })
}

fn derive_name(
    type_name: &str,
    exceptions: &Exceptions,
    used: &mut [bool],
) -> Result<SqlName, NameError> {
    validate_type_name(type_name)?;
    let mut output = Output::new(type_name);
    type_name
        .split("::")
        .try_for_each(|part| append_part(part, exceptions, used, &mut output))?;
    output.finish()
}

pub fn sql_name(type_name: &str, exceptions: &Exceptions) -> Result<SqlName, NameError> {
    derive_name(type_name, exceptions, &mut [])
}

pub fn sql_name_used(
    type_name: &str,
    exceptions: &Exceptions,
    used: &mut Vec<bool>,
) -> Result<SqlName, NameError> {
    used.clear();
    used.resize(exceptions.entries.len(), false);
    derive_name(type_name, exceptions, used)
}

#[cfg(test)]
mod tests;
