use super::ProbeError;
use inventory_core::catalog::runtime::RuntimeProtocol;
use std::{
    collections::BTreeMap,
    fmt::{self, Write},
};

pub(super) enum Outcome {
    Items(u64),
    Empty,
    Unsupported(String),
    Failed { class: &'static str, detail: String },
    SkippedChild,
}

pub(super) struct Row<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub protocol: RuntimeProtocol,
    pub outcome: Outcome,
}

pub(super) fn render(mut rows: Vec<Row<'_>>) -> Result<String, ProbeError> {
    rows.sort_unstable_by_key(|row| (row.service, row.operation, protocol(row.protocol)));
    let mut report = Report(String::new());
    let mut total = Totals::default();
    let mut protocols = BTreeMap::<&str, Totals>::new();
    for row in rows {
        total.add(&row.outcome)?;
        protocols
            .entry(protocol(row.protocol))
            .or_default()
            .add(&row.outcome)?;
        row.write(&mut report)?;
    }
    write_totals(&mut report, &total)?;
    for (protocol, total) in protocols {
        writeln!(report, "protocol {protocol} {total}")?;
    }
    Ok(report.0)
}

fn write_totals(report: &mut Report, total: &Totals) -> fmt::Result {
    writeln!(report, "total items {}", total.items)?;
    writeln!(report, "total empty {}", total.empty)?;
    writeln!(report, "total unsupported {}", total.unsupported)?;
    writeln!(report, "total failed {}", total.failed)?;
    writeln!(report, "total skipped-child {}", total.skipped)
}

impl Row<'_> {
    fn write(&self, report: &mut Report) -> fmt::Result {
        write!(
            report,
            "{} {} {} ",
            self.service,
            self.operation,
            protocol(self.protocol)
        )?;
        match &self.outcome {
            Outcome::Items(count) => writeln!(report, "items {count}"),
            Outcome::Empty => writeln!(report, "empty"),
            Outcome::Unsupported(detail) => writeln!(report, "unsupported {detail}"),
            Outcome::Failed { class, detail } => writeln!(report, "failed {class} {detail}"),
            Outcome::SkippedChild => writeln!(report, "skipped child"),
        }
    }
}

#[derive(Default)]
struct Totals {
    items: u64,
    empty: u64,
    unsupported: u64,
    failed: u64,
    skipped: u64,
}

impl Totals {
    fn add(&mut self, outcome: &Outcome) -> Result<(), ProbeError> {
        let total = match outcome {
            Outcome::Items(_) => &mut self.items,
            Outcome::Empty => &mut self.empty,
            Outcome::Unsupported(_) => &mut self.unsupported,
            Outcome::Failed { .. } => &mut self.failed,
            Outcome::SkippedChild => &mut self.skipped,
        };
        *total = total.checked_add(1).ok_or(ProbeError::TotalOverflow)?;
        Ok(())
    }
}

impl fmt::Display for Totals {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "items {} empty {} unsupported {} failed {} skipped-child {}",
            self.items, self.empty, self.unsupported, self.failed, self.skipped,
        )
    }
}

fn protocol(protocol: RuntimeProtocol) -> &'static str {
    match protocol {
        RuntimeProtocol::AwsJson10 => "awsJson1_0",
        RuntimeProtocol::AwsJson11 => "awsJson1_1",
        RuntimeProtocol::RpcV2Cbor => "rpcv2Cbor",
        RuntimeProtocol::AwsQuery => "awsQuery",
        RuntimeProtocol::Ec2Query => "ec2Query",
        RuntimeProtocol::RestJson1 => "restJson1",
        RuntimeProtocol::RestXml => "restXml",
    }
}

struct Report(String);

impl Write for Report {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0
            .len()
            .checked_add(value.len())
            .filter(|length| *length <= 67_108_864)
            .ok_or(fmt::Error)?;
        self.0.try_reserve(value.len()).map_err(|_| fmt::Error)?;
        self.0.push_str(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
