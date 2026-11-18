use super::*;

pub(super) fn validate_step((position, step): (usize, &SeedStep)) -> Result<(), SeedError> {
    if usize::from(step.id.0) != position
        || !valid_service(&step.service)
        || !valid_name(&step.operation)
        || (step.identity.is_some() && !valid_kind(&step.kind))
        || step.inputs.len() > MAX_INPUTS_PER_STEP
        || step.response_checks.len() > MAX_INPUTS_PER_STEP
        || step
            .expected_identity
            .as_ref()
            .is_some_and(|identity| identity.is_empty() || identity.len() > 1024)
        || invalid_checks(step)
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
    validate_identity(step)?;
    step.inputs.iter().try_for_each(validate_input)
}

fn invalid_checks(step: &SeedStep) -> bool {
    step.response_checks.iter().any(|check| {
        !step
            .inputs
            .iter()
            .any(|input| input.field == check.input_field)
    })
}

fn valid_service(value: &str) -> bool {
    value.len() <= MAX_FIELD_NAME && value.split('-').all(valid_name)
}

fn validate_identity(step: &SeedStep) -> Result<(), SeedError> {
    match &step.identity {
        None if !step.kind.is_empty() || step.expected_identity.is_some() => Err(
            SeedError::Invalid("auxiliary step cannot declare a resource identity"),
        ),
        Some(SeedIdentity::Request {
            input_field,
            template,
        }) => {
            if !step
                .inputs
                .iter()
                .any(|input| input_field.parts().starts_with(input.field.parts()))
            {
                return Err(SeedError::Invalid(
                    "request identity must select a configured input field",
                ));
            }
            if let Some(template) = template {
                super::identity::validate_template(template)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
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
    if let SeedInputValue::Literal(value) = &input.value {
        super::super::literal::validate(value)?;
    }
    if let SeedInputValue::Template(value) = &input.value {
        value.validate()?;
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
        step.inputs.iter().try_for_each(|input| match &input.value {
            SeedInputValue::PreviousOutput { step, path } => validate_source(position, *step, path),
            SeedInputValue::Template(template) => template
                .bindings
                .iter()
                .try_for_each(|binding| validate_source(position, binding.step, &binding.path)),
            _ => Ok(()),
        })
    })
}

fn validate_source(position: usize, source: StepId, path: &FieldPath) -> Result<(), SeedError> {
    if usize::from(source.0) >= position || path.parts().is_empty() {
        return Err(SeedError::Invalid(
            "seed binding must refer to an earlier operation output",
        ));
    }
    Ok(())
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
            || steps
                .get(usize::from(relationship.source.0))
                .is_none_or(|step| step.identity.is_none())
            || steps
                .get(usize::from(relationship.target.0))
                .is_none_or(|step| step.identity.is_none())
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
