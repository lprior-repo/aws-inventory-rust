use super::oracle::optional_value;
use super::*;
use crate::inventory_seed::model::{Continuation, PreparedIdentity, PreparedResource, Selector};
use crate::inventory_seed::{SeedIdentityEvidence, SeedResourceIdentity};
use inventory_core::record::ListRef;

pub(super) fn resources(
    state: &SeedState,
    step: super::super::StepId,
    spec: &PreparedResource,
    response: &Record,
    inputs: &[(Vec<FieldId>, Value)],
) -> Result<SeedResources, SeedError> {
    let (id, evidence) = match &spec.identity {
        PreparedIdentity::Response(path) => (
            selected_id(ValueRef::Map(response.root()), path)?,
            SeedIdentityEvidence::Response,
        ),
        PreparedIdentity::ResponseList {
            list,
            member,
            count,
            token,
        } => {
            let items = complete_list(response.root(), list, *count, token.as_ref())?;
            return items
                .iter()?
                .map(|item| {
                    let id = selected_id(item?, member)?;
                    Ok(resource(
                        state,
                        step,
                        spec,
                        id,
                        SeedIdentityEvidence::ServiceCreated,
                    ))
                })
                .collect::<Result<_, SeedError>>()
                .map(SeedResources::List);
        }
        PreparedIdentity::Request {
            input_field,
            template,
        } => {
            let value = request_value(inputs, input_field)?;
            let id = match template {
                Some(template) => ResourceId::parse(&render(template, value, state)?)?,
                None => ResourceId::parse(value)?,
            };
            (id, SeedIdentityEvidence::Request)
        }
    };
    validate_exact_identity(spec.expected_identity.as_deref(), &id)?;
    Ok(SeedResources::One(resource(
        state, step, spec, id, evidence,
    )))
}

fn complete_list<'a>(
    root: MapRef<'a>,
    list: &[Selector],
    expected: usize,
    token: Option<&Continuation>,
) -> Result<ListRef<'a>, SeedError> {
    let ValueRef::List(items) = value_path(root, list)? else {
        return Err(SeedError::Invalid("seed response identity is not a list"));
    };
    let observed = items.count()?;
    if observed != expected {
        return Err(SeedError::ResponseListCount { expected, observed });
    }
    if let Some(token) = token
        && optional_value(ValueRef::Map(root), &token.path)?.is_some()
    {
        return Err(SeedError::ResponseListContinuation {
            member: token.member.clone(),
        });
    }
    Ok(items)
}

fn selected_id(
    root: ValueRef<'_>,
    path: &[crate::inventory_seed::model::Selector],
) -> Result<ResourceId, SeedError> {
    match select_value(root, path)? {
        ValueRef::Text(text) => Ok(ResourceId::parse(text)?),
        _ => Err(SeedError::Invalid("modeled identity output is not text")),
    }
}

fn resource(
    state: &SeedState,
    step: super::super::StepId,
    spec: &PreparedResource,
    id: ResourceId,
    evidence: SeedIdentityEvidence,
) -> SeedResourceIdentity {
    let kind = spec.kind.clone();
    let key = ResourceKey {
        account: state.account.clone(),
        region: spec.scope.region(&state.region),
        kind: kind.clone(),
        id: id.clone(),
    };
    SeedResourceIdentity {
        step,
        kind,
        id,
        key,
        evidence,
    }
}

pub(super) fn request_value<'a>(
    inputs: &'a [(Vec<FieldId>, Value)],
    path: &[FieldId],
) -> Result<&'a str, SeedError> {
    let (prefix, value) = inputs
        .iter()
        .find(|(prefix, _)| path.starts_with(prefix))
        .ok_or(SeedError::Invalid("request identity input is missing"))?;
    let value = path
        .iter()
        .skip(prefix.len())
        .try_fold(value, |value, field| match value {
            Value::Map(items) => items
                .get(&inventory_core::record::Key::Field(*field))
                .ok_or(SeedError::Invalid(
                    "request identity nested field is missing",
                )),
            _ => Err(SeedError::Invalid(
                "request identity path is not a structure",
            )),
        })?;
    match value {
        Value::Text(value) => Ok(value),
        _ => Err(SeedError::Invalid("request identity must select text")),
    }
}

pub(super) fn render(template: &str, value: &str, state: &SeedState) -> Result<String, SeedError> {
    template
        .split('{')
        .enumerate()
        .try_fold(String::new(), |mut output, (index, part)| {
            if index == 0 {
                append(&mut output, part)?;
                return Ok(output);
            }
            let (name, rest) = part
                .split_once('}')
                .ok_or(SeedError::Invalid("identity template token is incomplete"))?;
            let replacement = match name {
                "value" => value,
                "account" => state.account.as_str(),
                "region" => state.region.as_str(),
                "partition" => &state.partition,
                _ => return Err(SeedError::Invalid("identity template token is unknown")),
            };
            append(&mut output, replacement)?;
            append(&mut output, rest)?;
            Ok(output)
        })
}

fn append(output: &mut String, text: &str) -> Result<(), SeedError> {
    if output
        .len()
        .checked_add(text.len())
        .is_none_or(|length| length > 4096)
    {
        return Err(SeedError::Invalid("seed identity exceeds its byte limit"));
    }
    output.push_str(text);
    Ok(())
}

pub(super) fn caller_identity(account: &str, arn: &str) -> Result<(AccountId, String), SeedError> {
    let account = AccountId::parse(account)?;
    let mut parts = arn.split(':');
    let prefix = parts.next();
    let partition = parts
        .next()
        .ok_or(SeedError::Invalid("caller ARN partition is missing"))?;
    let arn_account = parts.nth(2);
    if prefix != Some("arn")
        || arn_account != Some(account.as_str())
        || partition.is_empty()
        || !partition
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(SeedError::Invalid(
            "caller ARN must confirm account and partition",
        ));
    }
    Ok((account, partition.to_owned()))
}
