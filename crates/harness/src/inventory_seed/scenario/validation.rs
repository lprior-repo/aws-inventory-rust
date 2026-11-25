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
    if conflicting_inputs(step) {
        return Err(SeedError::Invalid(
            "seed operation contains conflicting input fields",
        ));
    }
    validate_identity(step)
}

fn conflicting_inputs(step: &SeedStep) -> bool {
    step.inputs.iter().enumerate().any(|(index, input)| {
        step.inputs
            .iter()
            .take(index)
            .any(|previous| input_paths_conflict(&previous.field, &input.field))
    })
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
            validate_request_field(step, input_field)?;
            if let Some(template) = template {
                super::identity::validate_template(template)?;
            }
            Ok(())
        }
        Some(SeedIdentity::ResponseList { count, .. })
            if !(1..=256).contains(count) || step.expected_identity.is_some() =>
        {
            Err(SeedError::Invalid(
                "seed response identity list needs a count from 1 through 256 and no scalar identity",
            ))
        }
        _ => Ok(()),
    }
}

fn validate_request_field(step: &SeedStep, field: &FieldPath) -> Result<(), SeedError> {
    if !step
        .inputs
        .iter()
        .any(|input| field.parts().starts_with(input.field.parts()))
    {
        return Err(SeedError::Invalid(
            "request identity must select a configured input field",
        ));
    }
    Ok(())
}
pub(super) fn input_paths_conflict(left: &FieldPath, right: &FieldPath) -> bool {
    let shorter = left.parts().len().min(right.parts().len());
    left.parts()
        .iter()
        .take(shorter)
        .eq(right.parts().iter().take(shorter))
}

pub(super) fn validate_relationships(
    steps: &[SeedStep],
    relationships: &[SeedRelationship],
) -> Result<(), SeedError> {
    relationships.iter().try_for_each(|relationship| {
        if !resource(steps, relationship.source)
            || !resource(steps, relationship.target)
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

fn resource(steps: &[SeedStep], id: StepId) -> bool {
    steps.get(usize::from(id.0)).is_some_and(|step| {
        matches!(
            step.identity,
            Some(SeedIdentity::Response(_) | SeedIdentity::Request { .. })
        )
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
