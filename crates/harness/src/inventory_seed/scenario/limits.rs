use super::*;

pub(super) fn validate(
    steps: &[SeedStep],
    relationships: &[SeedRelationship],
) -> Result<(), SeedError> {
    if steps.is_empty() || steps.len() > MAX_STEPS || relationships.len() > MAX_STEPS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its step or relationship limit",
        ));
    }
    let count = input_count(steps)?;
    if count > MAX_TOTAL_INPUTS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its input binding limit",
        ));
    }
    Ok(())
}

fn input_count(steps: &[SeedStep]) -> Result<usize, SeedError> {
    steps
        .iter()
        .enumerate()
        .try_fold(0_usize, |total, (position, step)| {
            step.inputs.iter().try_fold(total, |total, input| {
                total
                    .checked_add(values::validate(&input.value, position)?)
                    .ok_or(SeedError::Invalid("seed scenario input count overflow"))
            })
        })
}
