use super::{normalized_code, own_text};
use crate::response::DecodeError;

pub(super) const MAX_FIELD_BYTES: usize = 256;

#[derive(Clone, Copy)]
pub(super) enum Field {
    Code,
    Message,
}

impl Field {
    pub(super) fn matches(self, key: &str) -> bool {
        match self {
            Self::Code => matches!(key, "__type" | "code" | "Code"),
            Self::Message => matches!(key, "message" | "Message"),
        }
    }

    pub(super) fn invalid(self) -> DecodeError {
        DecodeError::Invalid(match self {
            Self::Code => "invalid service error code",
            Self::Message => "invalid service error message",
        })
    }

    pub(super) fn record(self, found: &mut Option<String>, value: &str) -> Result<(), DecodeError> {
        let value = match self {
            Self::Code => normalized_code(value)?,
            Self::Message if value.len() <= MAX_FIELD_BYTES => value,
            Self::Message => return Err(self.invalid()),
        };
        match found {
            Some(previous) if previous.as_str() != value => Err(DecodeError::Invalid(match self {
                Self::Code => "conflicting service error codes",
                Self::Message => "conflicting service error messages",
            })),
            Some(_) => Ok(()),
            None => {
                *found = Some(own_text(value)?);
                Ok(())
            }
        }
    }
}

pub(super) fn error_header(headers: &http::HeaderMap) -> Result<Option<&str>, DecodeError> {
    error_headers(headers).next().transpose()
}

/// Every recognized error-code header, highest priority first.
pub(super) fn error_headers(
    headers: &http::HeaderMap,
) -> impl Iterator<Item = Result<&str, DecodeError>> {
    [
        "x-amzn-errortype",
        "x-amzn-query-error",
        "smithy-error-code",
        "x-amz-error-code",
    ]
    .into_iter()
    .filter_map(move |name| {
        let mut values = headers.get_all(name).into_iter();
        let first = values.next()?;
        Some(if values.any(|value| value != first) {
            Err(Field::Code.invalid())
        } else {
            first
                .to_str()
                .map_err(|_| Field::Code.invalid())
                .and_then(normalized_code)
        })
    })
}
