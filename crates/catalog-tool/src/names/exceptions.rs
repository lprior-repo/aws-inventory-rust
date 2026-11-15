use super::{MAX_EXCEPTIONS_BYTES, MAX_EXCEPTIONS_ROWS, NameError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exceptions {
    pub(super) entries: Vec<(String, String)>,
}

impl Exceptions {
    pub fn builtin() -> Self {
        const ENTRIES: &[(&str, &str)] = &[
            ("ACMPCA", "acm_pca"), ("B2BI", "b2bi"), ("FSx", "fsx"),
            ("IPv4", "ipv4"), ("IoT", "iot"), ("OAuth", "oauth"),
            ("OAuth2", "oauth2"), ("OTel", "otel"), ("SMSVOICE", "sms_voice"),
            ("VPCDHCP", "vpc_dhcp"), ("WAFv2", "wafv2"),
        ];
        Self { entries: ENTRIES.iter().map(|(spelling, replacement)| ((*spelling).to_owned(), (*replacement).to_owned())).collect() }
    }

    pub fn none() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn from_entries(pairs: &[(&str, &str)]) -> Result<Self, NameError> {
        validate_size(pairs)?;
        pairs.iter().zip(1..).try_fold(Vec::<(String, String)>::new(), |mut entries, (&pair, line)| {
            let pair = validate_pair(pair, line)?;
            if entries.iter().any(|(spelling, _)| spelling == &pair.0) {
                return Err(NameError::DuplicateException { line });
            }
            entries.push(pair);
            Ok(entries)
        }).map(|entries| Self { entries })
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

fn validate_size(pairs: &[(&str, &str)]) -> Result<(), NameError> {
    let length = pairs.iter().fold(0_usize, |total, (spelling, replacement)| total.saturating_add(spelling.len()).saturating_add(replacement.len()));
    if length > MAX_EXCEPTIONS_BYTES {
        return Err(NameError::ExceptionsTooLarge { length, limit: MAX_EXCEPTIONS_BYTES });
    }
    if pairs.len() > MAX_EXCEPTIONS_ROWS {
        return Err(NameError::TooManyExceptions { rows: pairs.len(), limit: MAX_EXCEPTIONS_ROWS });
    }
    Ok(())
}

fn validate_pair((spelling, replacement): (&str, &str), line: usize) -> Result<(String, String), NameError> {
    let invalid = || NameError::InvalidException { line };
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
