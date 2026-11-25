use super::*;

pub(super) fn request_record(
    plan: &PagePlan,
    response: &Record,
    headers: &http::HeaderMap,
) -> Result<Option<Record>, SweepError> {
    let truncated = truncated(plan, response)?;
    if truncated == Some(false) {
        return Ok(None);
    }
    let fields = plan
        .tokens
        .iter()
        .map(|token| response_field(token, response, headers))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, _>>()?;
    if fields.is_empty() && truncated != Some(true) {
        return Ok(None);
    }
    validate_tuple(plan, &fields)?;
    input::merge_input(&empty_record()?, &fields).map(Some)
}

fn response_field(
    token: &TokenBinding,
    response: &Record,
    headers: &http::HeaderMap,
) -> Result<Option<input::InputField>, SweepError> {
    response_value(token, response, headers).map(|value| {
        value.map(|value| input::InputField {
            path: token.request_path.clone(),
            value,
        })
    })
}

pub(super) fn validate_tuple(
    plan: &PagePlan,
    fields: &[input::InputField],
) -> Result<(), SweepError> {
    if plan.tokens.is_empty()
        || plan.tokens.len() > 8
        || plan.tokens.iter().all(|token| token.optional)
        || plan.tokens.iter().any(|token| {
            !token.optional && !fields.iter().any(|field| field.path == token.request_path)
        })
    {
        return Err(SweepError::Operation(
            "pagination response omitted required continuation",
        ));
    }
    Ok(())
}

pub(super) fn truncated(plan: &PagePlan, response: &Record) -> Result<Option<bool>, SweepError> {
    let Some(path) = plan.truncated_path.as_ref() else {
        return Ok(None);
    };
    match path_value(response, path)? {
        Some(ValueRef::Bool(value)) => Ok(Some(value)),
        _ => Err(SweepError::Operation(
            "pagination truncated flag is missing or not boolean",
        )),
    }
}

fn response_value(
    token: &TokenBinding,
    response: &Record,
    headers: &http::HeaderMap,
) -> Result<Option<Value>, SweepError> {
    match (&token.response_placement, token.selection) {
        (TokenPlacement::Body(path), selection) if *path == token.response_path => {
            let value = match selection {
                TokenSelection::Value => path_value(response, path)?,
                TokenSelection::LastListItem => last_item::select(response, path)?,
            };
            value
                .map(|value| typed(value, token.value_type))
                .transpose()
                .map(Option::flatten)
        }
        (TokenPlacement::Header(name), TokenSelection::Value) => {
            header_token(headers, name, token.value_type)
        }
        _ => Err(SweepError::Operation(
            "pagination response placement is unsupported",
        )),
    }
}

pub(super) fn typed(
    value: ValueRef<'_>,
    token_type: TokenType,
) -> Result<Option<Value>, SweepError> {
    match (token_type, value) {
        (_, ValueRef::Null) => Ok(None),
        (TokenType::String, ValueRef::Text("")) => Ok(None),
        (TokenType::String, ValueRef::Text(_))
        | (TokenType::Integer, ValueRef::Int(_))
        | (TokenType::Boolean, ValueRef::Bool(_)) => {
            value.to_value().map(Some).map_err(SweepError::from)
        }
        _ => Err(SweepError::Operation(
            "pagination token has the wrong native type",
        )),
    }
}

pub(super) fn path_value<'a>(
    response: &'a Record,
    path: &[FieldId],
) -> Result<Option<ValueRef<'a>>, SweepError> {
    let (leaf, parents) =
        path.split_last()
            .filter(|_| path.len() <= 8)
            .ok_or(SweepError::Operation(
                "pagination path exceeds compiled bounds",
            ))?;
    let parent = parents
        .iter()
        .try_fold(Some(response.root()), |current, id| {
            current.map_or(Ok(None), |map| match map.get_field(*id)? {
                None | Some(ValueRef::Null) => Ok(None),
                Some(ValueRef::Map(child)) => Ok(Some(child)),
                _ => Err(SweepError::Operation(
                    "pagination path traverses a non-map value",
                )),
            })
        })?;
    parent.map_or(Ok(None), |map| {
        map.get_field(*leaf).map_err(SweepError::from)
    })
}

pub(super) fn header_token(
    headers: &http::HeaderMap,
    name: &str,
    token_type: TokenType,
) -> Result<Option<Value>, SweepError> {
    if headers.get_all(name).iter().count() > 1 {
        return Err(SweepError::Operation(
            "pagination response header is repeated",
        ));
    }
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| SweepError::Operation("pagination response header is not text"))?;
    if value.is_empty() {
        return Ok(None);
    }
    parse_header(value, token_type).map(Some)
}

fn parse_header(value: &str, token_type: TokenType) -> Result<Value, SweepError> {
    Ok(match token_type {
        TokenType::String => Value::Text(value.to_owned()),
        TokenType::Integer => {
            Value::Int(value.parse().map_err(|_| {
                SweepError::Operation("pagination response header is not an integer")
            })?)
        }
        TokenType::Boolean => {
            Value::Bool(value.parse().map_err(|_| {
                SweepError::Operation("pagination response header is not a boolean")
            })?)
        }
    })
}
