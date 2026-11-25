use cmdb_view::{Database, ScopeFreshness, ScopeFreshnessReport, ScopeOutcomeCounts, ViewError};
use std::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreshnessRow {
    pub scope: String,
    pub outcome: &'static str,
    pub age: String,
    pub last_complete: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreshnessPanel {
    pub summary: String,
    pub total_scopes: u64,
    pub counts: ScopeOutcomeCounts,
    pub rows: Vec<FreshnessRow>,
    pub truncated: bool,
}

pub fn format_age(age_ms: u64) -> String {
    let seconds = Duration::from_millis(age_ms).as_secs();
    if seconds < SECONDS_PER_MINUTE {
        format!("{seconds}s")
    } else if seconds < SECONDS_PER_HOUR {
        format!("{}m", seconds / SECONDS_PER_MINUTE)
    } else if seconds < SECONDS_PER_DAY {
        format!(
            "{}h {:02}m",
            seconds / SECONDS_PER_HOUR,
            (seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE
        )
    } else {
        format!(
            "{}d {:02}h",
            seconds / SECONDS_PER_DAY,
            (seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR
        )
    }
}

fn row(scope: &ScopeFreshness) -> FreshnessRow {
    let children = if scope.has_parent { " (children)" } else { "" };
    FreshnessRow {
        scope: format!(
            "{}/{} {}:{}{children}",
            scope.account, scope.region, scope.service, scope.operation
        ),
        outcome: scope.outcome.as_str(),
        age: format_age(scope.age_ms),
        last_complete: scope
            .complete_age_ms
            .map_or_else(|| "never".to_owned(), format_age),
    }
}

impl FreshnessPanel {
    pub fn from_report(report: &ScopeFreshnessReport) -> Self {
        let counts = report.counts;
        let listed = u64::try_from(report.scopes.len()).unwrap_or(u64::MAX);
        Self {
            summary: format!(
                "{} scopes: {} complete, {} partial, {} inaccessible, {} unsupported, {} failed, {} unobserved",
                report.total_scopes,
                counts.complete,
                counts.partial,
                counts.inaccessible,
                counts.unsupported,
                counts.failed,
                counts.unobserved
            ),
            total_scopes: report.total_scopes,
            counts,
            rows: report.scopes.iter().map(row).collect(),
            truncated: report.total_scopes > listed,
        }
    }

    pub async fn load(database: &Database, now_ms: i64, limit: usize) -> Result<Self, ViewError> {
        Ok(Self::from_report(
            &database.scope_freshness(now_ms, limit).await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmdb_view::ScopeOutcome;

    fn scope(outcome: ScopeOutcome, age_ms: u64, complete_age_ms: Option<u64>) -> ScopeFreshness {
        ScopeFreshness {
            account: "111111111111".to_owned(),
            region: "us-east-1".to_owned(),
            service: "ec2".to_owned(),
            operation: "DescribeVpcs".to_owned(),
            has_parent: false,
            sweep_id: "sweep-1".to_owned(),
            outcome,
            observed_at_ms: 0,
            last_complete_at_ms: None,
            age_ms,
            complete_age_ms,
        }
    }

    #[test]
    fn ages_pick_the_largest_two_units() {
        assert_eq!(format_age(0), "0s");
        assert_eq!(format_age(59_999), "59s");
        assert_eq!(format_age(60_000), "1m");
        assert_eq!(format_age(3_599_000), "59m");
        assert_eq!(format_age(3_600_000), "1h 00m");
        assert_eq!(format_age(11_100_000), "3h 05m");
        assert_eq!(format_age(86_400_000), "1d 00h");
        assert_eq!(format_age(187_200_000), "2d 04h");
    }

    #[test]
    fn rows_show_stored_outcome_and_both_ages() {
        let report = ScopeFreshnessReport {
            now_ms: 0,
            scopes: vec![
                scope(ScopeOutcome::Partial, 120_000, Some(3_600_000)),
                scope(ScopeOutcome::Failed, 1_000, None),
            ],
            total_scopes: 2,
            counts: ScopeOutcomeCounts {
                partial: 1,
                failed: 1,
                ..ScopeOutcomeCounts::default()
            },
        };
        let panel = FreshnessPanel::from_report(&report);
        assert_eq!(
            panel.rows,
            vec![
                FreshnessRow {
                    scope: "111111111111/us-east-1 ec2:DescribeVpcs".to_owned(),
                    outcome: "partial",
                    age: "2m".to_owned(),
                    last_complete: "1h 00m".to_owned(),
                },
                FreshnessRow {
                    scope: "111111111111/us-east-1 ec2:DescribeVpcs".to_owned(),
                    outcome: "failed",
                    age: "1s".to_owned(),
                    last_complete: "never".to_owned(),
                },
            ]
        );
        assert_eq!(
            panel.summary,
            "2 scopes: 0 complete, 1 partial, 0 inaccessible, 0 unsupported, 1 failed, 0 unobserved"
        );
        assert!(!panel.truncated);
    }

    #[test]
    fn panel_reports_truncation_and_marks_parented_scopes() {
        let mut parented = scope(ScopeOutcome::Complete, 0, Some(0));
        parented.has_parent = true;
        let report = ScopeFreshnessReport {
            now_ms: 0,
            scopes: vec![parented],
            total_scopes: 5,
            counts: ScopeOutcomeCounts {
                complete: 5,
                ..ScopeOutcomeCounts::default()
            },
        };
        let panel = FreshnessPanel::from_report(&report);
        assert!(panel.truncated);
        assert!(
            panel
                .rows
                .iter()
                .all(|row| row.scope.ends_with("(children)"))
        );
    }
}
