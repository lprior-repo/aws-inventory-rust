use super::{
    Batch, Binding, FixedValue, Identity, OverlayError, Rule, error::Context, fields, raw,
};
use toml::Value;

pub(super) fn identity(context: &Context<'_>, value: Value) -> Result<Identity, OverlayError> {
    let raw: raw::Identity = raw::decode(context, value)?;
    match (raw.arn, raw.template, raw.key) {
        (Some(value), None, None) => {
            Ok(Identity::Arn(fields::selected_path(context, "arn", &value)?))
        }
        (None, Some(value), None) => {
            let text = fields::string(context, "template", &value)?;
            template(context, text)?;
            Ok(Identity::Template(text.to_owned()))
        }
        (None, None, Some(value)) => {
            let paths = fields::list(context, "key", Some(&value), 1, 8, fields::selected_path)?;
            Ok(Identity::Key(paths))
        }
        _ => Err(context.error("", "arn/template/key", Rule::Identity)),
    }
}
fn template(context: &Context<'_>, text: &str) -> Result<(), OverlayError> {
    let (open, seen) =
        text.char_indices()
            .try_fold((None, false), |(open, seen), (index, character)| {
                template_char(context, text, open, seen, index, character)
            })?;
    if open.is_some() || !seen {
        return Err(context.error("template", text, Rule::Template));
    }
    Ok(())
}
fn template_char(
    context: &Context<'_>,
    text: &str,
    open: Option<usize>,
    seen: bool,
    index: usize,
    character: char,
) -> Result<(Option<usize>, bool), OverlayError> {
    match (open, character) {
        (None, '{') => Ok((Some(index.saturating_add(1)), seen)),
        (Some(start), '}')
            if text.get(start..index).is_some_and(|path| path == "$" || super::valid_path(path)) =>
        {
            Ok((None, true))
        }
        (_, '{' | '}') => Err(context.error("template", text, Rule::Template)),
        (open, _) => Ok((open, seen)),
    }
}

pub(super) fn binding(context: &Context<'_>, value: Value) -> Result<Binding, OverlayError> {
    let raw: raw::Binding = raw::decode(context, value)?;
    match (raw.from, raw.path, raw.account, raw.region) {
        (Some(from), Some(path), None, None) => {
            let operation = fields::operation(context, "from", &from)?;
            fields::not_self(context, "from", &operation)?;
            Ok(Binding::From {
                operation,
                path: fields::selected_path(context, "path", &path)?,
            })
        }
        (None, None, Some(value), None) => {
            context_binding(context, "account", value, Binding::Account)
        }
        (None, None, None, Some(value)) => {
            context_binding(context, "region", value, Binding::Region)
        }
        _ => Err(context.error("", "from+path/account/region", Rule::Binding)),
    }
}
fn context_binding(
    context: &Context<'_>,
    key: &str,
    value: Value,
    binding: Binding,
) -> Result<Binding, OverlayError> {
    match value {
        Value::Boolean(true) => Ok(binding),
        Value::Boolean(false) => Err(context.error(key, "false", Rule::Binding)),
        value => Err(context.error(
            key,
            fields::spelling(&value),
            Rule::WrongType {
                expected: "boolean",
            },
        )),
    }
}
pub(super) fn batch(context: &Context<'_>, value: Value) -> Result<Batch, OverlayError> {
    let raw: raw::Batch = raw::decode(context, value)?;
    let replaces = fields::required(context, "replaces", raw.replaces.as_ref())?;
    let replaces = fields::operation(context, "replaces", replaces)?;
    fields::not_self(context, "replaces", &replaces)?;
    let member = fields::required(context, "member", raw.member.as_ref())?;
    let member = fields::path(context, "member", member)?;
    let max = fields::required(context, "max", raw.max.as_ref())?;
    let max = fields::number(context, "max", max, 1, 10_000)?;
    Ok(Batch {
        member,
        max,
        replaces,
    })
}
pub(super) fn fixed(context: &Context<'_>, value: Value) -> Result<FixedValue, OverlayError> {
    match value {
        Value::String(text) => Ok(FixedValue::String(text)),
        Value::Integer(number) => Ok(FixedValue::Integer(number)),
        Value::Boolean(boolean) => Ok(FixedValue::Boolean(boolean)),
        Value::Array(_) => fixed_strings(context, &value),
        value => Err(context.error(
            "",
            fields::spelling(&value),
            Rule::WrongType {
                expected: "string, integer, boolean or string array",
            },
        )),
    }
}
fn fixed_strings(context: &Context<'_>, value: &Value) -> Result<FixedValue, OverlayError> {
    let strings = fields::list(context, "", Some(value), 0, 64, |context, key, value| {
        fields::string(context, key, value).map(str::to_owned)
    })?;
    Ok(FixedValue::Strings(strings))
}
