use super::{OverlayError, Paginator, Rule, TokenPair, error::Context, fields, raw};
use toml::Value;

pub(super) fn paginator(context: &Context<'_>, value: Value) -> Result<Paginator, OverlayError> {
    let raw: raw::Paginator = raw::decode(context, value)?;
    let kind = fields::required(context, "kind", raw.kind.as_ref())?;
    let kind = fields::string(context, "kind", kind)?;
    paginator_keys(context, kind, &raw)?;
    match kind {
        "none" => Ok(Paginator::None),
        "token" => Ok(Paginator::Token(pair_fields(
            context,
            raw.input.as_ref(),
            raw.output.as_ref(),
        )?)),
        "tokens" => tokens(context, raw),
        _ => Err(context.error("kind", kind, Rule::Value)),
    }
}
fn paginator_keys(
    context: &Context<'_>,
    kind: &str,
    raw: &raw::Paginator,
) -> Result<(), OverlayError> {
    let keys = [
        ("input", &raw.input),
        ("output", &raw.output),
        ("pairs", &raw.pairs),
        ("truncated", &raw.truncated),
    ];
    if let Some((key, Some(value))) = keys
        .into_iter()
        .find(|(key, value)| value.is_some() && forbidden_key(kind, key))
    {
        return Err(context.error(key, fields::spelling(value), Rule::PaginatorKey));
    }
    Ok(())
}
fn forbidden_key(kind: &str, key: &str) -> bool {
    match kind {
        "none" => true,
        "token" => matches!(key, "pairs" | "truncated"),
        "tokens" => matches!(key, "input" | "output"),
        _ => false,
    }
}
fn pair_fields(
    context: &Context<'_>,
    input: Option<&Value>,
    output: Option<&Value>,
) -> Result<TokenPair, OverlayError> {
    let input = fields::required(context, "input", input)?;
    let input = fields::path(context, "input", input)?;
    let output = fields::required(context, "output", output)?;
    let output = fields::path(context, "output", output)?;
    Ok(TokenPair { input, output, optional: false })
}
fn tokens(context: &Context<'_>, raw: raw::Paginator) -> Result<Paginator, OverlayError> {
    let value = fields::required(context, "pairs", raw.pairs.as_ref())?;
    let pairs = fields::list(context, "pairs", Some(value), 1, 8, |context, _, value| {
        token_pair(context, value)
    })?;
    distinct_inputs(context, &pairs)?;
    if pairs.iter().all(|pair| pair.optional) {
        return Err(context.error("pairs.optional", "true", Rule::Value));
    }
    Ok(Paginator::Tokens {
        pairs,
        truncated: fields::optional_path(context, "truncated", raw.truncated.as_ref())?,
    })
}
fn distinct_inputs(context: &Context<'_>, pairs: &[TokenPair]) -> Result<(), OverlayError> {
    if let Some((_, pair)) = pairs.iter().enumerate().find(|(index, pair)| {
        pairs
            .iter()
            .take(*index)
            .any(|previous| previous.input == pair.input)
    }) {
        return Err(context.error("pairs.input", &pair.input, Rule::DuplicateInput));
    }
    Ok(())
}

fn token_pair(context: &Context<'_>, value: &Value) -> Result<TokenPair, OverlayError> {
    let child = context.child("pairs");
    let raw: raw::Pair = raw::decode(&child, value.clone())?;
    let mut pair = pair_fields(&child, raw.input.as_ref(), raw.output.as_ref())?;
    pair.optional = match raw.optional {
        None => false,
        Some(Value::Boolean(value)) => value,
        Some(value) => return Err(child.error(
            "optional", fields::spelling(&value), Rule::WrongType { expected: "boolean" },
        )),
    };
    Ok(pair)
}
