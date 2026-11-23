use super::verify::{metric_value, zero};
use crate::HarnessError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CollectorExit {
    Complete,
    CompletedWithOutcomes,
    Fatal,
    DrainPending,
    WorkRemaining,
    GraphPending,
    Unknown(i32),
    Terminated,
}

impl CollectorExit {
    fn parse(code: Option<i32>) -> Self {
        match code {
            Some(0) => Self::Complete,
            Some(1) => Self::Fatal,
            Some(2) => Self::CompletedWithOutcomes,
            Some(3) => Self::DrainPending,
            Some(4) => Self::WorkRemaining,
            Some(5) => Self::GraphPending,
            Some(other) => Self::Unknown(other),
            None => Self::Terminated,
        }
    }

    fn verify(self, summary: &str) -> Result<(), HarnessError> {
        match self {
            Self::Complete | Self::CompletedWithOutcomes => completed(summary),
            other => Err(HarnessError::invalid(format!(
                "collector exit {other:?}; see stage log"
            ))),
        }
    }
}

pub(super) fn accept(code: Option<i32>, summary: &str) -> Result<(), HarnessError> {
    CollectorExit::parse(code).verify(summary)
}

fn completed(summary: &str) -> Result<(), HarnessError> {
    [
        "resource_writes",
        "rows",
        "complete",
        "unsupported",
        "failed",
    ]
    .iter()
    .try_for_each(|metric| metric_value(summary, metric).map(|_| ()))?;
    ["partial", "work_remaining", "graph_pending"]
        .iter()
        .try_for_each(|metric| zero(summary, metric))
}

#[cfg(test)]
mod tests;
