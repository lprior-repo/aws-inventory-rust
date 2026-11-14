use super::{
    Binding, MAX_FILE_BYTES, MAX_OPERATIONS, OperationName, OverlayError, Rule, ServiceOverlay,
    error::Context, fields, operation, raw,
};
use std::collections::{BTreeMap, BTreeSet};
use toml::Value;

impl ServiceOverlay {
    pub fn parse(service: &str, text: &str) -> Result<Self, OverlayError> {
        let context = Context {
            service,
            operation: None,
            prefix: String::new(),
        };
        let raw = decode_root(&context, text)?;
        let (reviewed, reviewer) = metadata(&context, &raw)?;
        let operations = operations(&context, raw.operation)?;
        cycles(&context, &operations)?;
        Ok(Self {
            service: service.to_owned(),
            format: 1,
            reviewed,
            reviewer,
            operations,
        })
    }
}
fn decode_root(context: &Context<'_>, text: &str) -> Result<raw::Root, OverlayError> {
    check_service(context)?;
    if text.len() > MAX_FILE_BYTES {
        return Err(context.error(
            "",
            text.len(),
            Rule::Limit {
                max: MAX_FILE_BYTES,
            },
        ));
    }
    let value = toml::from_str(text).map_err(|source| OverlayError::Syntax {
        service: context.service.to_owned(),
        source,
    })?;
    raw::decode(context, value)
}
fn check_service(context: &Context<'_>) -> Result<(), OverlayError> {
    if !context
        .service
        .starts_with(|character: char| character.is_ascii_lowercase() || character.is_ascii_digit())
        || !context
            .service
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(context.error("service", context.service, Rule::Service));
    }
    Ok(())
}
fn metadata(context: &Context<'_>, raw: &raw::Root) -> Result<(String, String), OverlayError> {
    check_format(context, raw.format.as_ref())?;
    let reviewed = reviewed(context, raw.reviewed.as_ref())?;
    let reviewer = reviewer(context, raw.reviewer.as_ref())?;
    Ok((reviewed, reviewer))
}
fn check_format(context: &Context<'_>, value: Option<&Value>) -> Result<(), OverlayError> {
    let value = fields::required(context, "format", value)?;
    let format = value.as_integer().ok_or_else(|| {
        context.error(
            "format",
            fields::spelling(value),
            Rule::WrongType {
                expected: "integer",
            },
        )
    })?;
    if format != 1 {
        return Err(context.error("format", format, Rule::Format));
    }
    Ok(())
}
fn reviewed(context: &Context<'_>, value: Option<&Value>) -> Result<String, OverlayError> {
    let value = fields::required(context, "reviewed", value)?;
    let text = fields::string(context, "reviewed", value)?;
    if !date(text) {
        return Err(context.error("reviewed", text, Rule::Date));
    }
    Ok(text.to_owned())
}
fn reviewer(context: &Context<'_>, value: Option<&Value>) -> Result<String, OverlayError> {
    let value = fields::required(context, "reviewer", value)?;
    let text = fields::string(context, "reviewer", value)?;
    if text.is_empty() || text.len() > 64 {
        return Err(context.error("reviewer", text, Rule::Reviewer));
    }
    Ok(text.to_owned())
}
fn date(text: &str) -> bool {
    if !text
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return false;
    }
    let mut parts = text.split('-');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day), None) => date_parts(year, month, day),
        _ => false,
    }
}
fn date_parts(year: &str, month: &str, day: &str) -> bool {
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return false;
    }
    match (year.parse::<u16>(), month.parse::<u8>(), day.parse::<u8>()) {
        (Ok(year), Ok(month), Ok(day)) => year > 0 && calendar(year, month, day),
        _ => false,
    }
}
fn calendar(y: u16, m: u8, d: u8) -> bool {
    let max = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) => 29,
        2 => 28,
        _ => 0,
    };
    d > 0 && d <= max
}
fn operations(
    context: &Context<'_>,
    value: Option<Value>,
) -> Result<BTreeMap<OperationName, super::OperationOverlay>, OverlayError> {
    let table = fields::table(context, "operation", value)?;
    if table.len() > MAX_OPERATIONS {
        return Err(context.error(
            "operation",
            table.len(),
            Rule::Limit {
                max: MAX_OPERATIONS,
            },
        ));
    }
    table
        .into_iter()
        .map(|entry| operation_entry(context, entry))
        .collect()
}
fn operation_entry(
    context: &Context<'_>,
    (name, value): (String, Value),
) -> Result<(OperationName, super::OperationOverlay), OverlayError> {
    let child = Context {
        service: context.service,
        operation: Some(&name),
        prefix: String::new(),
    };
    let key = OperationName::new(&name)
        .map_err(|_| child.error("operation", &name, Rule::OperationName))?;
    Ok((key, operation::parse(&child, value)?))
}
fn cycles(
    context: &Context<'_>,
    operations: &BTreeMap<OperationName, super::OperationOverlay>,
) -> Result<(), OverlayError> {
    let mut remaining: BTreeSet<_> = operations.keys().collect();
    (0..operations.len()).try_for_each(|_| {
        if let Some(name) = ready_operation(&remaining, operations) {
            remaining.remove(name);
            return Ok(());
        }
        let name = remaining.first().map(|name| name.as_str());
        let child = Context {
            operation: name,
            ..context.clone()
        };
        Err(child.error("bind", name.map_or("", |name| name), Rule::BindCycle))
    })
}
fn ready_operation<'a>(
    remaining: &BTreeSet<&'a OperationName>,
    operations: &BTreeMap<OperationName, super::OperationOverlay>,
) -> Option<&'a OperationName> {
    remaining.iter().copied().find(|name| {
        operations.get(*name).is_some_and(|operation| {
            operation.bind.values().all(|binding| match binding {
                Binding::From { operation, .. } => !remaining.contains(operation),
                Binding::Account | Binding::Region => true,
            })
        })
    })
}
