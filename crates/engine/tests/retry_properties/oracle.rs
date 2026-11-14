use super::*;

pub(super) fn retryable_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Transient),
        Just(Step::Throttled),
        Just(Step::AttemptTimeout),
    ]
}

pub(super) fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Success),
        retryable_step(),
        Just(Step::Permanent),
        Just(Step::Denied),
        Just(Step::Cancelled),
    ]
}

pub(super) fn oracle(tape: &[Step], budget: usize) -> (usize, Terminal) {
    tape.iter()
        .take(budget)
        .enumerate()
        .find_map(|(index, step)| {
            let terminal = match step {
                Step::Success => Terminal::Success(index),
                Step::Permanent => Terminal::Permanent,
                Step::Denied => Terminal::Denied,
                Step::Cancelled => Terminal::Cancelled,
                _ => return None,
            };
            Some((index.saturating_add(1), terminal))
        })
        .unwrap_or((budget, Terminal::Exhausted))
}

pub(super) fn terminal(result: &Result<usize, ApiError>) -> Terminal {
    match result {
        Ok(index) => Terminal::Success(*index),
        Err(ApiError::Permanent(_)) => Terminal::Permanent,
        Err(ApiError::Denied(_)) => Terminal::Denied,
        Err(ApiError::Cancelled) => Terminal::Cancelled,
        Err(ApiError::Exhausted(_)) => Terminal::Exhausted,
        Err(_) => Terminal::Unexpected,
    }
}

pub(super) async fn respond(step: Option<Step>, index: usize) -> Result<usize, ApiError> {
    match step {
        Some(Step::Success) => Ok(index),
        Some(Step::Transient) => Err(ApiError::Transient("synthetic transient".into())),
        Some(Step::Throttled) => Err(ApiError::Throttled("synthetic throttle".into())),
        Some(Step::AttemptTimeout) => pending().await,
        Some(Step::Permanent) => Err(ApiError::Permanent("synthetic permanent".into())),
        Some(Step::Denied) => Err(ApiError::Denied("synthetic denial".into())),
        Some(Step::Cancelled) => Err(ApiError::Cancelled),
        None => Err(ApiError::InvalidConfig("outcome tape was overrun".into())),
    }
}
