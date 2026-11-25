use super::*;

pub(super) fn decode(bytes: &[u8]) -> Result<Cursor, SweepError> {
    if bytes.len() > MAX_CURSOR {
        return Err(SweepError::Operation(
            "persisted pagination cursor exceeds its bound",
        ));
    }
    let wrapped = Record::from_bytes(bytes.to_vec())?;
    let pages = integer(&wrapped, PAGES)?
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or(SweepError::Operation(
            "persisted pagination cursor has invalid page count",
        ))?;
    let current = Some(Record::from_bytes(bytes_field(&wrapped, CURRENT)?)?);
    let partial = partial_marker(&wrapped)?;
    Ok(Cursor {
        pages,
        current,
        partial,
    })
}

fn partial_marker(wrapped: &Record) -> Result<bool, SweepError> {
    match field(wrapped, PARTIAL)? {
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(SweepError::Operation(
            "persisted pagination cursor has invalid partial marker",
        )),
    }
}

pub(super) fn request_fields(
    plan: &PagePlan,
    current: &Record,
) -> Result<Vec<input::InputField>, SweepError> {
    let fields = plan
        .tokens
        .iter()
        .map(|token| request_field(token, current))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, _>>()?;
    tokens::validate_tuple(plan, &fields)?;
    let canonical = input::merge_input(&empty_record()?, &fields)?;
    if canonical.as_bytes() != current.as_bytes() {
        return Err(SweepError::Operation(
            "pagination checkpoint contains unreviewed or empty fields",
        ));
    }
    Ok(fields)
}

fn request_field(
    token: &TokenBinding,
    current: &Record,
) -> Result<Option<input::InputField>, SweepError> {
    let value = tokens::path_value(current, &token.request_path)?;
    value
        .map(|value| tokens::typed(value, token.value_type))
        .transpose()
        .map(Option::flatten)
        .map(|value| {
            value.map(|value| input::InputField {
                path: token.request_path.clone(),
                value,
            })
        })
}
