use super::*;
use crate::inventory_seed::{SeedIdentityEvidence, SeedResourceIdentity};
use crate::inventory_seed::model::{PreparedIdentity, PreparedResource};

pub(super) fn resource(state: &SeedState, step: super::super::StepId, spec: &PreparedResource,
    response: &Record, inputs: &[(Vec<FieldId>, Value)]) -> Result<SeedResourceIdentity, SeedError> {
    let (id, evidence) = match &spec.identity {
        PreparedIdentity::Response(path) => (ResourceId::parse(&text_path(response.root(), path)?)?, SeedIdentityEvidence::Response),
        PreparedIdentity::Request { input_field, template } => {
            let value = request_value(inputs, input_field)?;
            let text = match template {
                Some(template) => render(template, value, state)?, None => value.to_owned(),
            };
            (ResourceId::parse(&text)?, SeedIdentityEvidence::Request)
        }
    };
    validate_exact_identity(spec.expected_identity.as_deref(), &id)?;
    let kind = spec.kind.clone();
    let key = ResourceKey { account: state.account.clone(), region: spec.scope.region(&state.region), kind: kind.clone(), id: id.clone() };
    Ok(SeedResourceIdentity { step, kind, id, key, evidence })
}

pub(super) fn request_value<'a>(inputs: &'a [(Vec<FieldId>, Value)], path: &[FieldId])
    -> Result<&'a str, SeedError> {
    let (prefix, value) = inputs.iter().find(|(prefix, _)| path.starts_with(prefix))
        .ok_or(SeedError::Invalid("request identity input is missing"))?;
    let value = path.iter().skip(prefix.len()).try_fold(value, |value, field| match value {
        Value::Map(items) => items.get(&inventory_core::record::Key::Field(*field))
            .ok_or(SeedError::Invalid("request identity nested field is missing")),
        _ => Err(SeedError::Invalid("request identity path is not a structure")),
    })?;
    match value { Value::Text(value) => Ok(value), _ => Err(SeedError::Invalid("request identity must select text")) }
}

pub(super) fn render(template: &str, value: &str, state: &SeedState) -> Result<String, SeedError> {
    template.split('{').enumerate().try_fold(String::new(), |mut output, (index, part)| {
        if index == 0 { append(&mut output, part)?; return Ok(output); }
        let (name, rest) = part.split_once('}').ok_or(SeedError::Invalid("identity template token is incomplete"))?;
        let replacement = match name {
            "value" => value, "account" => state.account.as_str(), "region" => state.region.as_str(),
            "partition" => &state.partition,
            _ => return Err(SeedError::Invalid("identity template token is unknown")),
        };
        append(&mut output, replacement)?;
        append(&mut output, rest)?;
        Ok(output)
    })
}

fn append(output: &mut String, text: &str) -> Result<(), SeedError> {
    if output.len().checked_add(text.len()).is_none_or(|length| length > 4096) {
        return Err(SeedError::Invalid("seed identity exceeds its byte limit"));
    }
    output.push_str(text);
    Ok(())
}

pub(super) fn caller_identity(account: &str, arn: &str) -> Result<(AccountId, String), SeedError> {
    let account = AccountId::parse(account)?;
    let mut parts = arn.split(':');
    let prefix = parts.next();
    let partition = parts.next().ok_or(SeedError::Invalid("caller ARN partition is missing"))?;
    let arn_account = parts.nth(2);
    if prefix != Some("arn") || arn_account != Some(account.as_str())
        || partition.is_empty() || !partition.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
        return Err(SeedError::Invalid("caller ARN must confirm account and partition"));
    }
    Ok((account, partition.to_owned()))
}
