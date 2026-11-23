use super::*;
pub(super) fn key_text<'a>(
    record: &'a Record,
    paths: &[Box<[FieldId]>],
) -> Result<Cow<'a, str>, SweepError> {
    match paths {
        [] => {
            return Err(SweepError::Operation(
                "modeled composite identity has no fields",
            ));
        }
        [single] => return required_text(record, single),
        _ => {}
    }
    paths
        .iter()
        .enumerate()
        .try_fold(String::new(), |text, (position, path)| {
            append(text, position, &required_text(record, path)?)
        })
        .map(Cow::Owned)
}

pub(super) fn append(
    mut text: String,
    position: usize,
    component: &str,
) -> Result<String, SweepError> {
    let additional = component_length(position, component)?;
    check_length(text.len(), additional)?;
    text.try_reserve(additional)?;
    if position > 0 {
        text.push('|');
    }
    write!(text, "{}:{component}", component.len())
        .map_err(|_| SweepError::Operation("identity formatting failed"))?;
    Ok(text)
}

fn component_length(position: usize, component: &str) -> Result<usize, SweepError> {
    let digits = if component.is_empty() {
        1
    } else {
        component.len().ilog10().saturating_add(1)
    };
    let digits =
        usize::try_from(digits).map_err(|_| SweepError::Operation("identity length overflow"))?;
    component
        .len()
        .checked_add(digits)
        .and_then(|length| length.checked_add(1))
        .and_then(|length| length.checked_add(usize::from(position > 0)))
        .ok_or(SweepError::Operation("identity length overflow"))
}

fn check_length(length: usize, additional: usize) -> Result<(), SweepError> {
    if length
        .checked_add(additional)
        .is_none_or(|length| length > 4096)
    {
        return Err(SweepError::Operation(
            "modeled resource identity is too long",
        ));
    }
    Ok(())
}
