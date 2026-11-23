use super::{ScenarioSeedOracle, SeedError, StepId};
mod message;
pub(super) use message::message;
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Accepted,
    Rejected { code: String, message: String },
    Skipped { dependency: StepId },
}

pub struct SeedRunReport {
    pub oracle: ScenarioSeedOracle,
    pub outcomes: Vec<StepOutcome>,
}

impl SeedRunReport {
    pub fn oracle_rows(&self) -> Vec<String> {
        let resources = self.oracle.resources.iter().map(resource_row);
        let edges = self.oracle.edges.iter().map(edge_row);
        let mut rows: Vec<_> = resources.chain(edges).collect();
        rows.sort();
        rows
    }

    pub fn matrix_rows(
        &self,
        steps: &[super::SeedStep],
        labels: &[String],
    ) -> Result<Vec<String>, SeedError> {
        if self.outcomes.len() != steps.len() || labels.len() != steps.len() {
            return Err(SeedError::Invalid("seed report labels do not match steps"));
        }
        steps
            .iter()
            .zip(labels)
            .zip(&self.outcomes)
            .map(|((step, label), outcome)| {
                let mut row = format!("{}\t{}\t{}\t", step.service, step.operation, label);
                append_outcome(&mut row, outcome, labels)?;
                Ok(row)
            })
            .collect()
    }
}

pub(super) fn bounded_text(text: &str) -> String {
    text.chars()
        .scan(0_usize, |bytes, character| {
            let character = if character.is_control() {
                ' '
            } else {
                character
            };
            *bytes = bytes.saturating_add(character.len_utf8());
            (*bytes <= 200).then_some(character)
        })
        .collect()
}

pub(super) fn rejection(error: SeedError) -> StepOutcome {
    match error {
        SeedError::Rejected { code, message } => StepOutcome::Rejected { code, message },
        error => StepOutcome::Rejected {
            code: "SeedError".into(),
            message: bounded_text(&error.to_string()),
        },
    }
}

fn resource_row(resource: &super::SeedResourceIdentity) -> String {
    let key = &resource.key;
    format!(
        "resource\t{}\t{}\t{}\t{}",
        key.kind.as_str(),
        key.account.as_str(),
        key.region.as_str(),
        key.id.as_str()
    )
}

fn edge_row(edge: &super::Edge) -> String {
    format!(
        "edge\t{}\t{}\t{}\t{}\t{}",
        edge.from.kind.as_str(),
        edge.from.id.as_str(),
        edge.relation.as_str(),
        edge.to.kind.as_str(),
        edge.to.id.as_str()
    )
}

fn append_outcome(
    row: &mut String,
    outcome: &StepOutcome,
    labels: &[String],
) -> Result<(), SeedError> {
    match outcome {
        StepOutcome::Accepted => row.push_str("accepted"),
        StepOutcome::Rejected { code, message } => {
            let code = bounded_text(code);
            let message = bounded_text(message);
            write!(row, "rejected\t{code}\t{message}")
                .map_err(|_| SeedError::Invalid("seed report formatting failed"))?;
        }
        StepOutcome::Skipped { dependency } => {
            let label = labels
                .get(usize::from(dependency.0))
                .ok_or(SeedError::Invalid("seed dependency label is missing"))?;
            write!(row, "skipped: depends on {label}")
                .map_err(|_| SeedError::Invalid("seed report formatting failed"))?;
        }
    }
    Ok(())
}
