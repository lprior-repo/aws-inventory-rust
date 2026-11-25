use super::report::Outcome;
use inventory_core::catalog::{Target, runtime::CallablePlan};
use inventory_engine::limiter::ApiError;
use inventory_sweep::{ChildProbeOutcome, SweepError};
use std::fmt::{self, Write};

pub(super) fn child(plan: &CallablePlan<'_>, result: ChildProbeOutcome) -> Outcome {
    match result {
        ChildProbeOutcome::Called { items, errors } => sampled(plan, items, &errors),
        ChildProbeOutcome::NoParentItems => skip("no parent items", None),
        ChildProbeOutcome::Unbound(error) => {
            let detail = error.and_then(|error| {
                let mut detail = Detail(String::new());
                detail.write_str(&error).ok().map(|()| detail.0)
            });
            skip(
                "no sampled parent item satisfies all bindings and row eligibility",
                detail,
            )
        }
        ChildProbeOutcome::TagEndpointBinding => {
            skip("tag endpoint uses dedicated resource tag binding", None)
        }
        _ => skip("unsupported child evidence", None),
    }
}

fn skip(reason: &'static str, detail: Option<String>) -> Outcome {
    Outcome::SkippedChild { reason, detail }
}

fn sampled(plan: &CallablePlan<'_>, items: u64, errors: &[SweepError]) -> Outcome {
    let hard_failure = errors.iter().find(|error| {
        !matches!(
            error,
            SweepError::Absent(_) | SweepError::Api(ApiError::UnsupportedType(_))
        )
    });
    if let Some(error) = hard_failure {
        return failed(plan, error);
    }
    if items != 0 {
        return Outcome::Items(items);
    }
    errors
        .iter()
        .find(|error| matches!(error, SweepError::Api(ApiError::UnsupportedType(_))))
        .or_else(|| errors.first())
        .map_or(Outcome::Empty, |error| failed(plan, error))
}

pub(super) fn failed(plan: &CallablePlan<'_>, error: &SweepError) -> Outcome {
    match detail(plan, error) {
        Ok(detail) => match error {
            SweepError::Absent(_) => Outcome::EmptyReason(detail),
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

pub(super) fn schedule_detail(error: &SweepError) -> Result<String, fmt::Error> {
    let mut detail = Detail(String::new());
    write!(detail, "{error}")?;
    Ok(detail.0)
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
