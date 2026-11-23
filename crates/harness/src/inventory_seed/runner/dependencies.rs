use super::*;

pub(super) fn dependencies(step: &PreparedStep<'_>) -> Result<Vec<super::super::StepId>, SeedError> {
    let mut pending: Vec<_> = step.inputs.iter().map(|input| &input.value).collect();
    let mut dependencies = Vec::new();
    (0..16_384).try_for_each(|_| {
        if let Some(value) = pending.pop() {
            append(value, &mut pending, &mut dependencies);
        }
        Ok::<_, SeedError>(())
    })?;
    if !pending.is_empty() {
        return Err(SeedError::Invalid("seed dependency traversal exceeds node bound"));
    }
    dependencies.sort();
    dependencies.dedup();
    Ok(dependencies)
}

fn append<'a>(value: &'a PreparedInputValue, pending: &mut Vec<&'a PreparedInputValue>, ids: &mut Vec<super::super::StepId>) {
    match value {
        PreparedInputValue::PreviousOutput { step, .. } => ids.push(*step),
        PreparedInputValue::Template(template) => ids.extend(template.bindings.iter().map(|binding| binding.step)),
        PreparedInputValue::List(items) => pending.extend(items),
        PreparedInputValue::Structure(fields) => pending.extend(fields.iter().map(|(_, value)| value)),
        _ => {}
    }
}
