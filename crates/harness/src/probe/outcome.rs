use super::report::Outcome;
use inventory_core::catalog::{Target, runtime::CallablePlan};
use inventory_engine::limiter::ApiError;
use inventory_sweep::SweepError;
use std::fmt::{self, Write};

pub(super) fn failed(plan: &CallablePlan<'_>, error: &SweepError) -> Outcome {
    match detail(plan, error) {
        Ok(detail) => match error {
            SweepError::Api(ApiError::UnsupportedType(_)) => Outcome::Unsupported(detail),
            _ => Outcome::Failed {
                class: class(error),
                detail,
            },
        },
        Err(error) => Outcome::Failed {
            class: "format",
            detail: error.to_string(),
        },
    }
}

fn detail(plan: &CallablePlan<'_>, error: &SweepError) -> Result<String, fmt::Error> {
    let mut detail = Detail(String::new());
    if matches!(error, SweepError::Decode(_))
        && let Some(shape) = output_shape(plan)
    {
        write!(detail, "shape={shape} ")?;
    }
    write!(detail, "{error}")?;
    Ok(detail.0)
}

fn output_shape<'a>(plan: &'a CallablePlan<'_>) -> Option<&'a str> {
    match plan.operation().output().ok()? {
        Target::Shape(index) => plan.service().shape(index).ok()??.name().ok(),
        Target::Prelude(name) => Some(name),
        _ => None,
    }
}

fn class(error: &SweepError) -> &'static str {
    match error {
        SweepError::Decode(_) => "decode",
        SweepError::Encode(_) => "protocol",
        SweepError::Transport(_) => "transport",
        SweepError::Credentials(_) | SweepError::Signing(_) => "credentials",
        SweepError::Api(ApiError::Denied(_)) => "denied",
        SweepError::Api(ApiError::Throttled(_)) => "throttled",
        SweepError::Api(ApiError::Timeout) => "timeout",
        SweepError::Api(ApiError::Cancelled) => "cancelled",
        SweepError::Api(_) => "api",
        SweepError::Store(_) => "store",
        _ => "plan",
    }
}

struct Detail(String);

impl Write for Detail {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for character in value.chars() {
            let character = if character.is_control() || character.is_whitespace() {
                ' '
            } else {
                character
            };
            if self
                .0
                .len()
                .checked_add(character.len_utf8())
                .is_none_or(|length| length > 1024)
            {
                break;
            }
            self.0.push(character);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
