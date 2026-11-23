use super::*;
use std::collections::BTreeSet;

pub(super) fn validate(value: &SeedInputValue, position: usize) -> Result<usize, SeedError> {
    let mut pending = vec![(value, 0_usize)];
    let mut count = 0_usize;
    (0..257)
        .find_map(|_| {
            let Some((value, depth)) = pending.pop() else {
                return Some(Ok(count));
            };
            if depth > MAX_BINDING_DEPTH {
                return Some(Err(SeedError::Invalid(
                    "seed input exceeds nesting depth 8",
                )));
            }
            count = count.saturating_add(1);
            visit(value, position, depth.saturating_add(1), &mut pending)
                .err()
                .map(Err)
        })
        .ok_or(SeedError::Invalid("seed input exceeds node limit 256"))?
}

fn visit<'a>(
    value: &'a SeedInputValue,
    position: usize,
    next: usize,
    pending: &mut Vec<(&'a SeedInputValue, usize)>,
) -> Result<(), SeedError> {
    match value {
        SeedInputValue::List(items) if items.len() <= 256 => {
            pending.extend(items.iter().map(|value| (value, next)));
            Ok(())
        }
        SeedInputValue::Structure(fields) if fields.len() <= 256 => {
            names(fields)?;
            pending.extend(fields.iter().map(|(_, value)| (value, next)));
            Ok(())
        }
        SeedInputValue::List(_) | SeedInputValue::Structure(_) => {
            Err(SeedError::Invalid("seed container exceeds node limit"))
        }
        value => scalar(value, position),
    }
}

fn names(fields: &[(String, SeedInputValue)]) -> Result<(), SeedError> {
    fields
        .iter()
        .try_fold(BTreeSet::new(), |mut seen, (name, _)| {
            if name.is_empty() || name.len() > MAX_FIELD_NAME || !seen.insert(name) {
                return Err(SeedError::Invalid(
                    "seed structure fields must be unique bounded names",
                ));
            }
            Ok(seen)
        })
        .map(|_| ())
}

fn source(position: usize, source: StepId, path: &FieldPath) -> Result<(), SeedError> {
    if usize::from(source.0) >= position || path.parts().is_empty() {
        return Err(SeedError::Invalid(
            "seed binding must refer to an earlier operation output",
        ));
    }
    Ok(())
}

fn scalar(value: &SeedInputValue, position: usize) -> Result<(), SeedError> {
    match value {
        SeedInputValue::PreviousOutput { step, path } => source(position, *step, path),
        SeedInputValue::Template(template) => {
            template.validate()?;
            template
                .bindings
                .iter()
                .try_for_each(|binding| source(position, binding.step, &binding.path))
        }
        SeedInputValue::Literal(value) => super::super::literal::validate(value),
        SeedInputValue::Text(value) if value.is_empty() || value.len() > 1024 => {
            Err(SeedError::Invalid("seed literal exceeds its length bound"))
        }
        _ => Ok(()),
    }
}
