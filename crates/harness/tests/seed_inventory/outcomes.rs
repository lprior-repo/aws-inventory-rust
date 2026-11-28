use crate::{
    REGION, TestResult,
    runtime::{self, NativeRuntime},
};
use aws_wire::{
    sign::Credentials,
    transport::{Client, Config},
};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::RegionId};
use inventory_engine::{
    inventory::decode_scope_key,
    limiter::{ApiExecutor, ExecutorConfig},
};
use inventory_sweep::{CredentialFailure, CredentialSource};
use sqlx::SqlitePool;
use std::collections::BTreeMap;

type ProbeTable = BTreeMap<(String, String), String>;
type StoredRow = ((String, String), String, Option<String>);

struct SeedCredentials<'a>(&'a Credentials);

impl CredentialSource for SeedCredentials<'_> {
    fn region(&self) -> &str {
        REGION
    }

    async fn resolve(&self) -> Result<Credentials, CredentialFailure> {
        Ok(self.0.clone())
    }
}

/// Probes the emulator that the sweep just ran against, using the seed's own credentials.
pub(super) async fn probe(native: &NativeRuntime) -> TestResult<ProbeTable> {
    let bytes = std::fs::read(runtime::catalog_path())?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(native.emulator().ca_bundle().to_path_buf()),
        ..Config::default()
    })?;
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let region = RegionId::parse(REGION)?;
    let text = inventory_harness::probe::run(
        &catalog,
        &SeedCredentials(&native.credentials),
        &client,
        &executor,
        &region,
        native.emulator().endpoint(),
        &[],
    )
    .await?;
    probe_table(&text)
}

fn probe_table(report: &str) -> TestResult<ProbeTable> {
    let mut table = ProbeTable::new();
    for line in report.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [service, operation, _protocol, outcome, ..] = fields.as_slice() else {
            continue;
        };
        let outcome = match *outcome {
            "items" | "empty" | "unsupported" | "failed" | "skipped" => *outcome,
            "not-applicable" => "not_applicable",
            _ => continue,
        };
        let key = ((*service).to_owned(), (*operation).to_owned());
        if table.insert(key, outcome.to_owned()).is_some() {
            return Err(format!("duplicate probe entry {service} {operation}").into());
        }
    }
    Ok(table)
}

fn contradicts(stored: &str, probed: &str) -> bool {
    matches!(
        (stored, probed),
        ("failed" | "inaccessible", "items" | "empty")
            | ("unsupported", "items")
            | ("complete", "not_applicable")
            | ("not_applicable", "items" | "empty")
    )
}

pub(super) async fn violations(pool: &SqlitePool, probes: &ProbeTable) -> TestResult<Vec<String>> {
    let rows: Vec<(Vec<u8>, String, Option<String>)> =
        sqlx::query_as("SELECT scope_key,outcome,detail FROM scope_outcomes")
            .fetch_all(pool)
            .await?;
    let mut stored: Vec<StoredRow> = Vec::with_capacity(rows.len());
    for (scope, outcome, detail) in rows {
        let scope = decode_scope_key(&scope)?;
        let key = (
            scope.service().as_str().to_owned(),
            scope.operation().as_str().to_owned(),
        );
        stored.push((key, outcome, detail));
    }
    Ok(judge(&stored, probes))
}

fn judge(stored: &[StoredRow], probes: &ProbeTable) -> Vec<String> {
    let mut compared = 0usize;
    let mut violations = Vec::new();
    for (key, outcome, detail) in stored {
        let Some(probed) = probes.get(key) else {
            continue;
        };
        if matches!(probed.as_str(), "items" | "empty" | "not_applicable") {
            compared = compared.saturating_add(1);
        }
        if contradicts(outcome, probed) {
            violations.push(format!(
                "{} {} stored {outcome} (detail: {}) but probe records {probed}",
                key.0,
                key.1,
                detail.as_deref().unwrap_or_default()
            ));
        }
    }
    if compared == 0 {
        violations.push("no stored scope matched any informative probe entry".to_owned());
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::{StoredRow, TestResult, contradicts, judge, probe_table};

    fn row(service: &str, operation: &str, outcome: &str) -> StoredRow {
        (
            (service.to_owned(), operation.to_owned()),
            outcome.to_owned(),
            None,
        )
    }

    #[test]
    fn given_stored_complete_when_probe_is_not_applicable_then_it_is_a_violation() {
        assert!(contradicts("complete", "not_applicable"));
    }

    #[test]
    fn given_stored_not_applicable_when_probe_has_items_then_it_is_a_violation() {
        assert!(contradicts("not_applicable", "items"));
    }

    #[test]
    fn given_stored_not_applicable_when_probe_is_empty_then_it_is_a_violation() {
        assert!(contradicts("not_applicable", "empty"));
    }

    #[test]
    fn given_stored_inaccessible_when_probe_is_inaccessible_then_it_is_not_a_violation() {
        assert!(!contradicts("inaccessible", "inaccessible"));
    }

    #[test]
    fn given_stored_inaccessible_when_probe_is_denied_failure_then_it_is_not_a_violation() {
        assert!(!contradicts("inaccessible", "failed"));
    }

    #[test]
    fn given_live_probe_text_when_parsed_then_denied_probe_is_recorded_as_failed() -> TestResult<()>
    {
        let table = probe_table(
            "sesv2 ListImportJobs restJson1 not-applicable account is in the sandbox\nsnow-device-management ListTasks restJson1 failed denied API executor error: access denied: AWS service failure: AccessDenied (AccessDeniedException)\n",
        )?;
        let key = |service: &str, operation: &str| (service.to_owned(), operation.to_owned());
        assert_eq!(
            table
                .get(&key("sesv2", "ListImportJobs"))
                .map(String::as_str),
            Some("not_applicable")
        );
        assert_eq!(
            table
                .get(&key("snow-device-management", "ListTasks"))
                .map(String::as_str),
            Some("failed")
        );
        Ok(())
    }

    #[test]
    fn given_duplicate_live_probe_entries_when_parsed_then_it_is_rejected() {
        let report =
            "sesv2 ListImportJobs restJson1 items one\nsesv2 ListImportJobs restJson1 empty two\n";
        assert!(probe_table(report).is_err());
    }

    #[test]
    fn given_stored_row_when_live_probe_contradicts_then_violation_names_the_live_outcome()
    -> TestResult<()> {
        let probes = probe_table("sesv2 ListImportJobs restJson1 items found\n")?;
        let violations = judge(&[row("sesv2", "ListImportJobs", "not_applicable")], &probes);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].ends_with("but probe records items"));
        Ok(())
    }

    #[test]
    fn given_only_skipped_probe_matches_when_judged_then_comparison_fails() -> TestResult<()> {
        let probes = probe_table("ec2 DescribeVpcs ec2Query skipped no usable probe\n")?;
        let violations = judge(&[row("ec2", "DescribeVpcs", "complete")], &probes);
        assert_eq!(
            violations,
            vec!["no stored scope matched any informative probe entry".to_owned()]
        );
        Ok(())
    }

    #[test]
    fn given_only_inconclusive_matches_when_judged_then_comparison_fails() -> TestResult<()> {
        for outcome in ["failed", "unsupported"] {
            let probes = probe_table(&format!("ec2 DescribeVpcs ec2Query {outcome} fixture\n"))?;
            let violations = judge(&[row("ec2", "DescribeVpcs", "complete")], &probes);
            assert_eq!(
                violations,
                ["no stored scope matched any informative probe entry"],
                "{outcome}"
            );
        }
        Ok(())
    }

    #[test]
    fn given_failed_and_informative_matches_when_judged_then_comparison_passes() -> TestResult<()> {
        let probes = probe_table(
            "ec2 DescribeVpcs ec2Query failed fixture transport error\nsesv2 ListImportJobs restJson1 empty no items\n",
        )?;
        let stored = [
            row("ec2", "DescribeVpcs", "complete"),
            row("sesv2", "ListImportJobs", "complete"),
        ];
        assert!(judge(&stored, &probes).is_empty());
        Ok(())
    }

    #[test]
    fn given_skipped_and_observed_matches_when_judged_then_observed_match_counts() -> TestResult<()>
    {
        let probes = probe_table(
            "ec2 DescribeVpcs ec2Query skipped no usable probe\nsesv2 ListImportJobs restJson1 empty no items\n",
        )?;
        let stored = [
            row("ec2", "DescribeVpcs", "complete"),
            row("sesv2", "ListImportJobs", "complete"),
        ];
        assert!(judge(&stored, &probes).is_empty());
        Ok(())
    }

    #[test]
    fn given_stored_row_absent_from_live_probe_then_no_fallback_report_is_consulted()
    -> TestResult<()> {
        let probes = probe_table("sesv2 ListImportJobs restJson1 items found\n")?;
        let violations = judge(&[row("ec2", "DescribeVpcs", "complete")], &probes);
        assert_eq!(
            violations,
            vec!["no stored scope matched any informative probe entry".to_owned()]
        );
        Ok(())
    }
}
