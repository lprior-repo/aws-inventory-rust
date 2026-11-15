use super::*;

pub(super) fn validate_step((position, step): (usize, &SeedStep)) -> Result<(), SeedError> {
    if usize::from(step.id.0) != position
        || !valid_name(&step.service)
        || !valid_name(&step.operation)
        || !valid_kind(&step.kind)
        || step.inputs.len() > MAX_INPUTS_PER_STEP
        || step.response_checks.len() > MAX_INPUTS_PER_STEP
    {
        return Err(SeedError::Invalid("seed operation descriptor is invalid"));
    }
    if step.inputs.iter().enumerate().any(|(index, input)| {
        step.inputs
            .iter()
            .take(index)
            .any(|previous| input_paths_conflict(&previous.field, &input.field))
    }) {
        return Err(SeedError::Invalid(
            "seed operation contains conflicting input fields",
        ));
    }
    step.inputs.iter().try_for_each(validate_input)
}
pub(super) fn input_paths_conflict(left: &FieldPath, right: &FieldPath) -> bool {
    let shorter = left.parts().len().min(right.parts().len());
    left.parts()
        .iter()
        .take(shorter)
        .eq(right.parts().iter().take(shorter))
}

pub(super) fn validate_input(input: &SeedInput) -> Result<(), SeedError> {
    if input.field.parts().is_empty() {
        return Err(SeedError::Invalid("seed input field path is empty"));
    }
    if let SeedInputValue::Text(value) = &input.value
        && (value.is_empty() || value.len() > 1024)
    {
        return Err(SeedError::Invalid("seed literal exceeds its length bound"));
    }
    Ok(())
}

pub(super) fn validate_bindings(steps: &[SeedStep]) -> Result<(), SeedError> {
    steps.iter().enumerate().try_for_each(|(position, step)| {
        step.inputs.iter().try_for_each(|input| {
            if let SeedInputValue::PreviousOutput { step: source, path } = &input.value
                && (usize::from(source.0) >= position || path.parts().is_empty())
            {
                return Err(SeedError::Invalid(
                    "seed binding must refer to an earlier operation output",
                ));
            }
            Ok(())
        })
    })
}

pub(super) fn validate_relationships(
    steps: &[SeedStep],
    relationships: &[SeedRelationship],
) -> Result<(), SeedError> {
    relationships.iter().try_for_each(|relationship| {
        if usize::from(relationship.source.0) >= steps.len()
            || usize::from(relationship.target.0) >= steps.len()
            || relationship.source == relationship.target
            || !valid_relation(&relationship.relation)
            || relationship.observed_path.parts().is_empty()
        {
            return Err(SeedError::Invalid(
                "seed relationship descriptor is invalid",
            ));
        }
        Ok(())
    })
}

pub(super) fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FIELD_NAME
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

pub(super) fn valid_kind(value: &str) -> bool {
    value.split("::").count() == 3 && value.split("::").all(valid_name)
}

pub(super) fn valid_relation(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FIELD_NAME
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub(super) fn validate_field(field: &&str) -> Result<String, SeedError> {
    if !valid_name(field) {
        return Err(SeedError::Invalid("seed field name is invalid"));
    }
    Ok((*field).to_owned())
}
