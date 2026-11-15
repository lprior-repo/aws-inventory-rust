#[path = "cursor/fields.rs"]
mod fields;
use fields::*;
use inventory_core::{
    catalog::runtime::{CallablePlan, TokenPlacement, TokenType},
    record::{Exclusions, FieldId, Key, Record, Value, ValueRef},
};

use crate::sweep_runtime::{SweepError, input, record};

const PAGES: u32 = 65_530;
const CURRENT: u32 = 65_532;
const PARTIAL: u32 = 65_533;
const MAX_TOKEN_RECORD: usize = 65_000;
const MAX_CURSOR: usize = 65_536;

#[derive(Default)]
pub(super) struct Cursor {
    pub pages: u32,
    pub current: Option<Record>,
    pub partial: bool,
}

pub(super) struct Advance {
    pub cursor: Record,
    pub continuation_digest: Option<[u8; 32]>,
}

impl Cursor {
    pub(super) fn decode(token: Option<&[u8]>) -> Result<Self, SweepError> {
        let Some(bytes) = token else {
            return Ok(Self::default());
        };
        if bytes.len() > MAX_CURSOR {
            return Err(SweepError::Operation(
                "persisted pagination cursor exceeds its bound",
            ));
        }
        let wrapped = Record::from_bytes(bytes.to_vec())?;
        let pages = integer(&wrapped, PAGES)?
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(SweepError::Operation(
                "persisted pagination cursor has invalid page count",
            ))?;
        let current = Some(Record::from_bytes(bytes_field(&wrapped, CURRENT)?)?);
        let partial = match field(&wrapped, PARTIAL)? {
            Some(ValueRef::Bool(value)) => value,
            _ => {
                return Err(SweepError::Operation(
                    "persisted pagination cursor has invalid partial marker",
                ));
            }
        };
        Ok(Self {
            pages,
            current,
            partial,
        })
    }

    pub(super) fn request_fields(
        &self,
        plan: &CallablePlan<'_>,
    ) -> Result<Vec<input::InputField>, SweepError> {
        let Some(current) = &self.current else {
            return Ok(Vec::new());
        };
        let tokens = &plan.pagination().tokens;
        if tokens.is_empty() {
            return Err(SweepError::Operation(
                "cursor has no reviewed pagination tokens",
            ));
        }
        let mut fields = Vec::new();
        fields.try_reserve(tokens.len())?;
        for token in tokens.iter() {
            let value = record::path_value(current.root(), &token.request_path)?
                .ok_or(SweepError::Operation(
                    "pagination checkpoint omitted a token",
                ))?
                .to_value()?;
            if value_is_empty(&value, token.value_type)? {
                return Err(SweepError::Operation(
                    "pagination checkpoint contains an empty token",
                ));
            }
            fields.push(input::InputField {
                path: token.request_path.clone(),
                value,
            });
        }
        Ok(fields)
    }

    pub(super) fn next(
        self,
        plan: &CallablePlan<'_>,
        response: &Record,
        headers: &http::HeaderMap,
        limit: u32,
    ) -> Result<Option<Advance>, SweepError> {
        let truncated = is_truncated(plan, response)?;
        let next_request = request_record(plan, response, headers)?;
        let has_next_request = next_request.is_some();
        let request = match next_request {
            Some(request) => request,
            None if truncated => empty_record()?,
            None => return Ok(None),
        };
        let pages = self
            .pages
            .checked_add(1)
            .ok_or(SweepError::Operation("pagination page counter overflow"))?;
        let partial = terminal_partial(
            truncated,
            has_next_request,
            pages,
            limit,
            request.as_bytes().len(),
        );
        let continuation_digest = Some(token_digest(&request)?);
        let current = if partial { empty_record()? } else { request };
        let cursor = encode(pages, current.as_bytes(), partial)?;
        if cursor.as_bytes().len() > MAX_CURSOR {
            return Err(SweepError::Operation(
                "pagination checkpoint exceeds durable cursor bound",
            ));
        }
        Ok(Some(Advance {
            cursor,
            continuation_digest,
        }))
    }
}

fn is_truncated(plan: &CallablePlan<'_>, response: &Record) -> Result<bool, SweepError> {
    let Some(path) = plan.pagination().truncated_path.as_ref() else {
        return Ok(false);
    };
    match record::path_value(response.root(), path)? {
        None | Some(ValueRef::Null) => Ok(false),
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(SweepError::Operation(
            "pagination truncated flag has the wrong native type",
        )),
    }
}
fn request_record(
    plan: &CallablePlan<'_>,
    response: &Record,
    headers: &http::HeaderMap,
) -> Result<Option<Record>, SweepError> {
    if plan.pagination().tokens.is_empty() {
        return Ok(None);
    }
    let mut fields = Vec::new();
    let mut incomplete = false;
    for token in plan.pagination().tokens.iter() {
        let value = match &token.response_placement {
            TokenPlacement::Body(_) => record::path_value(response.root(), &token.response_path)?
                .map(|value| value.to_value())
                .transpose()?,
            TokenPlacement::Header(name) => header_token(headers, name, token.value_type)?,
            TokenPlacement::Path(_) | TokenPlacement::Query(_) => {
                return Err(SweepError::Operation(
                    "pagination response placement is unsupported",
                ));
            }
        };
        let Some(value) = value else {
            incomplete = true;
            continue;
        };
        if value_is_empty(&value, token.value_type)? {
            incomplete = true;
            continue;
        }
        fields.try_reserve(1)?;
        fields.push(input::InputField {
            path: token.request_path.clone(),
            value,
        });
    }
    if fields.is_empty() {
        return Ok(None);
    }
    if incomplete {
        return Err(SweepError::Operation(
            "pagination response omitted part of its token tuple",
        ));
    }
    input::merge_input(&empty_record()?, &fields).map(Some)
}

fn header_token(
    headers: &http::HeaderMap,
    name: &str,
    token_type: TokenType,
) -> Result<Option<Value>, SweepError> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| SweepError::Operation("pagination response header is not text"))?;
    if value.is_empty() {
        return Ok(None);
    }
    let token =
        match token_type {
            TokenType::String => Value::Text(value.to_owned()),
            TokenType::Integer => Value::Int(value.parse().map_err(|_| {
                SweepError::Operation("pagination response header is not an integer")
            })?),
            TokenType::Boolean => match value {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => {
                    return Err(SweepError::Operation(
                        "pagination response header is not a boolean",
                    ));
                }
            },
        };
    Ok(Some(token))
}

fn value_is_empty(value: &Value, token_type: TokenType) -> Result<bool, SweepError> {
    match (token_type, value) {
        (TokenType::String, Value::Text(value)) => Ok(value.is_empty()),
        (TokenType::Integer, Value::Int(_)) | (TokenType::Boolean, Value::Bool(_)) => Ok(false),
        (_, Value::Null) => Ok(true),
        _ => Err(SweepError::Operation(
            "pagination response token has the wrong native type",
        )),
    }
}

fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(value: &str) -> Result<Record, Box<dyn std::error::Error>> {
        let key = Key::Field(FieldId::new(1)?);
        let map = std::collections::BTreeMap::from([(key, Value::Text(value.to_owned()))]);
        Ok(Record::encode(&Value::Map(map))?)
    }

    #[test]
    fn repeated_native_next_request_has_same_digest() -> Result<(), Box<dyn std::error::Error>> {
        let first = token("next")?;
        let second = token("next")?;
        let changed = token("other")?;
        assert_eq!(token_digest(&first)?, token_digest(&second)?);
        assert_ne!(token_digest(&first)?, token_digest(&changed)?);
        Ok(())
    }

    #[test]
    fn modeled_header_token_is_native_and_typed() -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = http::HeaderMap::new();
        headers.insert("x-next-token", http::HeaderValue::from_static("42"));
        assert_eq!(
            header_token(&headers, "x-next-token", TokenType::Integer)?,
            Some(Value::Int(42))
        );
        Ok(())
    }
    #[test]
    fn truncated_page_with_token_continues_until_bound() {
        assert!(!terminal_partial(true, true, 2, 10, 16));
        assert!(terminal_partial(true, false, 2, 10, 0));
    }
}
