use super::{EncodeError, RequestContext};
use inventory_core::{catalog::Target, record::ValueRef};
use std::borrow::Cow;

pub(crate) mod addressing;
mod host;
mod overrides;
mod routing;

pub(super) fn resolve<'a>(context: RequestContext<'a>) -> Result<Cow<'a, str>, EncodeError> {
    let endpoint = overrides::resolve(context)?.unwrap_or(context.endpoint);
    let endpoint = addressing::resolve(context, endpoint)?;
    if endpoint.len() > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    if routing::suppress_prefix(context)? {
        return Ok(endpoint);
    }
    let Some(template) = context
        .operation
        .host_prefix()?
        .filter(|value| !value.is_empty())
    else {
        return Ok(endpoint);
    };
    let prefix = if template.contains('{') {
        Cow::Owned(expand(context, template)?)
    } else {
        Cow::Borrowed(fragment(template)?)
    };
    host::prepend_host(&endpoint, &prefix, context.limits.uri_bytes()).map(Cow::Owned)
}

fn expand(context: RequestContext<'_>, template: &str) -> Result<String, EncodeError> {
    let mut output = String::new();
    let mut remaining = template;
    while let Some((literal, label)) = remaining.split_once('{') {
        append(&mut output, fragment(literal)?, context.limits.uri_bytes())?;
        let (name, rest) = label.split_once('}').ok_or_else(unresolved)?;
        append(
            &mut output,
            value(context, name)?,
            context.limits.uri_bytes(),
        )?;
        remaining = rest;
    }
    append(
        &mut output,
        fragment(remaining)?,
        context.limits.uri_bytes(),
    )?;
    Ok(output)
}

fn value<'a>(context: RequestContext<'a>, name: &str) -> Result<&'a str, EncodeError> {
    let Target::Shape(index) = context.operation.input()? else {
        return Err(unresolved());
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    for item in shape.members()? {
        let member = item?;
        if member.name()? != name || !member.wire()?.host_label()? {
            continue;
        }
        return label_value(context.input.get_field(member.id)?, member.id);
    }
    Err(unresolved())
}

fn label_value(
    value: Option<ValueRef<'_>>,
    field: inventory_core::record::FieldId,
) -> Result<&str, EncodeError> {
    let Some(ValueRef::Text(label)) = value else {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "missing or non-string host label",
        });
    };
    valid_label(label)
        .then_some(label)
        .ok_or(EncodeError::InvalidValue {
            field: Some(field),
            reason: "invalid host label",
        })
}

fn fragment(text: &str) -> Result<&str, EncodeError> {
    if text
        .bytes()
        .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.')
    {
        return Err(unresolved());
    }
    Ok(text)
}

fn valid_label(label: &str) -> bool {
    label
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.')
        && label.split('.').all(|part| {
            !part.is_empty() && part.len() <= 63 && !part.starts_with('-') && !part.ends_with('-')
        })
}

fn append(output: &mut String, text: &str, limit: usize) -> Result<(), EncodeError> {
    let length = output
        .len()
        .checked_add(text.len())
        .ok_or(EncodeError::Limit("URI"))?;
    if length > limit {
        return Err(EncodeError::Limit("URI"));
    }
    output.try_reserve(text.len())?;
    output.push_str(text);
    Ok(())
}

fn unresolved() -> EncodeError {
    EncodeError::InvalidValue {
        field: None,
        reason: "unresolved host label",
    }
}
