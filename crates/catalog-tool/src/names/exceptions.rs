use super::{MAX_EXCEPTIONS_BYTES, MAX_EXCEPTIONS_ROWS, NameError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exceptions {
    pub(super) entries: Vec<(String, String)>,
}

impl Exceptions {
    pub fn none() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn parse(text: &str) -> Result<Self, NameError> {
        validate_size(text)?;
        text.split_inclusive('\n')
            .zip(1..)
            .try_fold(
                Vec::<(String, String)>::new(),
                |mut entries, (text, line)| {
                    let pair = parse_exception_line(text, line)?;
                    if entries.iter().any(|(spelling, _)| spelling == &pair.0) {
                        return Err(NameError::DuplicateException { line });
                    }
                    entries.push(pair);
                    Ok(entries)
                },
            )
            .map(|entries| Self { entries })
    }

    pub(super) fn longest_match(&self, text: &str) -> Option<(usize, &str, &str)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, (spelling, _))| text.starts_with(spelling.as_str()))
            .max_by_key(|(_, (spelling, _))| spelling.len())
            .map(|(index, (spelling, replacement))| {
                (index, spelling.as_str(), replacement.as_str())
            })
    }
}

fn validate_size(text: &str) -> Result<(), NameError> {
    if text.len() > MAX_EXCEPTIONS_BYTES {
        return Err(NameError::ExceptionsTooLarge {
            length: text.len(),
            limit: MAX_EXCEPTIONS_BYTES,
        });
    }
    let rows = text.split_inclusive('\n').count();
    if rows > MAX_EXCEPTIONS_ROWS {
        return Err(NameError::TooManyExceptions {
            rows,
            limit: MAX_EXCEPTIONS_ROWS,
        });
    }
    Ok(())
}

fn parse_exception_line(text: &str, line: usize) -> Result<(String, String), NameError> {
    let invalid = || NameError::InvalidException { line };
    let text = text.strip_suffix('\n').ok_or_else(invalid)?;
    let (spelling, replacement) = text.split_once('\t').ok_or_else(invalid)?;
    let valid_spelling =
        !spelling.is_empty() && spelling.chars().all(|ch| ch.is_ascii_alphanumeric());
    let valid_replacement = replacement.split('_').all(|word| {
        !word.is_empty()
            && word
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    });
    if !valid_spelling || !valid_replacement {
        return Err(invalid());
    }
    Ok((spelling.to_owned(), replacement.to_owned()))
}
