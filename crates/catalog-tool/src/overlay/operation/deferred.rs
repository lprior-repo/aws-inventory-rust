use super::{OverlayClass, OverlayError, Rule, fields};
use crate::overlay::error::Context;
use toml::Value;

pub(super) fn reason(
    context: &Context<'_>,
    value: Option<&Value>,
    class: Option<OverlayClass>,
) -> Result<Option<String>, OverlayError> {
    match (class, value) {
        (Some(OverlayClass::Defer), Some(value)) => {
            let text = fields::string(context, "defer_reason", value)?;
            if text.trim().is_empty() || text.len() > 1024 || text.chars().any(char::is_control) {
                return Err(context.error("defer_reason", text, Rule::Value));
            }
            Ok(Some(text.to_owned()))
        }
        (Some(OverlayClass::Defer), None) => Err(context.error("defer_reason", "", Rule::Missing)),
        (_, Some(value)) => {
            Err(context.error("defer_reason", fields::spelling(value), Rule::Value))
        }
        (_, None) => Ok(None),
    }
}
