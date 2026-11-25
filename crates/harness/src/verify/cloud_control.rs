mod client;
mod database;
mod inputs;
mod output;

pub use client::CloudControlError;

use crate::{
    HarnessError,
    assertions::cloud_control::identities::{self, Direction, Rule},
};
use client::Listed;
use database::Reader;
use futures_util::{TryStreamExt, stream};
use inputs::{Inputs, Target};
use std::{fmt::Write, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Args {
    pub database: String,
    pub endpoint: String,
    pub ca_bundle: PathBuf,
    pub region: String,
    pub account: Option<String>,
    pub report: PathBuf,
    pub catalog: PathBuf,
    pub universal: PathBuf,
    pub differences: PathBuf,
    pub seeds: PathBuf,
    pub oracle: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub text: String,
    pub unexplained_seeded: usize,
    pub not_comparable_seeded: usize,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.unexplained_seeded == 0 && self.not_comparable_seeded == 0
    }
}

pub async fn run(args: &Args) -> Result<Report, HarnessError> {
    let owned = args.clone();
    let mut inputs = tokio::task::spawn_blocking(move || {
        crate::inventory_seed::SeedConfig::new(&owned.endpoint, &owned.ca_bundle, &owned.region)
            .map_err(|error| HarnessError::invalid(error.to_string()))?;
        output::validate(&owned)?;
        inputs::read(&owned)
    })
    .await
    .map_err(|error| HarnessError::invalid(format!("Cloud Control input task: {error}")))??;
    let rules = identities::rules(&inputs.explanations)?;
    let client = client::connect(
        &args.endpoint,
        &args.region,
        std::mem::take(&mut inputs.pem),
    )
    .await?;
    let mut reader = Reader::connect(&args.database).await?;
    let result = check(&client, &mut reader, &inputs, &rules, args).await;
    let closed = reader.close().await;
    let report = result.and_then(|report| closed.map(|()| report))?;
    output::write(args, report).await
}

async fn check(
    client: &aws_sdk_cloudcontrol::Client,
    reader: &mut Reader,
    inputs: &Inputs,
    rules: &[Rule<'_>],
    args: &Args,
) -> Result<Report, HarnessError> {
    stream::iter(inputs.targets.iter().map(Ok::<_, HarnessError>))
        .try_fold(
            (
                reader,
                Report {
                    text: unmapped(&inputs.no_cfn)?,
                    unexplained_seeded: 0,
                    not_comparable_seeded: 0,
                },
            ),
            |(reader, mut report), target| async move {
                let seeded = inputs.seeded.contains(&target.kind)
                    || inputs.seeded.contains(&target.type_name);
                let row = check_type(client, reader, target, rules, args, seeded).await?;
                if seeded {
                    report.unexplained_seeded = report
                        .unexplained_seeded
                        .saturating_add(row.unexplained_seeded);
                    report.not_comparable_seeded = report
                        .not_comparable_seeded
                        .saturating_add(row.not_comparable_seeded);
                }
                report.text.push_str(&row.text);
                Ok((reader, report))
            },
        )
        .await
        .map(|(_, report)| report)
}

fn unmapped(kinds: &std::collections::BTreeSet<String>) -> Result<String, HarnessError> {
    if kinds.is_empty() {
        return Ok(String::new());
    }
    let mut text = format!("# no CloudFormation type\tcount\t{}\n", kinds.len());
    kinds.iter().try_for_each(|kind| {
        writeln!(text, "{}\tno CloudFormation type", escape(kind))
            .map_err(|error| HarnessError::invalid(error.to_string()))
    })?;
    text.push_str("# Cloud Control comparisons\n");
    Ok(text)
}

async fn check_type(
    client: &aws_sdk_cloudcontrol::Client,
    reader: &mut Reader,
    target: &Target,
    rules: &[Rule<'_>],
    args: &Args,
    seeded: bool,
) -> Result<Report, HarnessError> {
    let listed = if target.listable {
        client::list(client, &target.type_name).await?
    } else {
        Listed::NotListable
    };
    let listed = match listed {
        Listed::Identities(identities) => identities,
        Listed::NotListable => return Ok(not_listable(&target.type_name)),
    };
    let stored = reader
        .identities(&target.kind, &args.region, args.account.as_deref())
        .await?;
    if listed.is_empty() && !stored.is_empty() {
        return Ok(not_comparable(&target.type_name, stored.len(), seeded));
    }
    let comparison = identities::compare(&listed, &stored).map_err(|source| {
        CloudControlError::AmbiguousIdentity {
            type_name: target.type_name.clone(),
            source,
        }
    })?;
    render(&target.type_name, &comparison, rules)
}

fn not_listable(type_name: &str) -> Report {
    Report {
        text: format!("{type_name}\tnot listable\n"),
        unexplained_seeded: 0,
        not_comparable_seeded: 0,
    }
}

fn not_comparable(type_name: &str, stored: usize, seeded: bool) -> Report {
    Report {
        text: format!(
            "{type_name}\tnot comparable\tlisting returned no identities\tstored\t{stored}\n"
        ),
        unexplained_seeded: 0,
        not_comparable_seeded: usize::from(seeded),
    }
}

fn render(
    type_name: &str,
    comparison: &identities::Comparison<'_>,
    rules: &[Rule<'_>],
) -> Result<Report, HarnessError> {
    let missing = comparison
        .differences
        .iter()
        .filter(|difference| difference.direction == Direction::Missing)
        .count();
    let extra = comparison.differences.len().saturating_sub(missing);
    let mut text = format!(
        "{type_name}\tmatched\t{}\tmissing_from_sweep\t{missing}\textra_in_sweep\t{extra}\n",
        comparison.matched
    );
    let unexplained_seeded =
        comparison
            .differences
            .iter()
            .try_fold(0_usize, |count, difference| {
                let explanation = rules.iter().find(|rule| rule.covers(type_name, difference));
                let (status, reason) =
                    explanation.map_or(("unexplained", ""), |rule| ("explained", rule.reason));
                writeln!(
                    text,
                    "{type_name}\t{}\t{}\t{status}\t{}",
                    difference.direction.label(),
                    escape(difference.identity),
                    escape(reason)
                )
                .map_err(|error| HarnessError::invalid(error.to_string()))?;
                Ok::<_, HarnessError>(count.saturating_add(usize::from(explanation.is_none())))
            })?;
    Ok(Report {
        text,
        unexplained_seeded,
        not_comparable_seeded: 0,
    })
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests;
