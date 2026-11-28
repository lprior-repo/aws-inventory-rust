use crate::TestResult;
use std::collections::BTreeMap;

pub(super) fn exit_violation(code: Option<i32>, stdout: &[u8]) -> TestResult<Option<String>> {
    let text = std::str::from_utf8(stdout)?;
    let mut lines = text.lines();
    let summary = lines.next();
    let not_applicable = lines
        .filter_map(|line| line.strip_prefix("outcomes_v1 "))
        .flat_map(str::split_whitespace)
        .find_map(|token| token.strip_prefix("not_applicable=")?.parse::<u64>().ok())
        .unwrap_or(0);
    let counts: BTreeMap<&str, u64> = summary
        .into_iter()
        .flat_map(str::split_whitespace)
        .filter_map(|token| token.split_once('='))
        .filter_map(|(name, value)| value.parse().ok().map(|value| (name, value)))
        .collect();
    let count = |name: &str| {
        counts
            .get(name)
            .copied()
            .ok_or_else(|| format!("sweep summary lacks {name}"))
    };
    for name in [
        "partial",
        "unobserved",
        "work_remaining",
        "unattributed_resources",
    ] {
        if count(name)? != 0 {
            return Ok(Some(format!("{name} must be zero")));
        }
    }
    if count("complete")? == 0 && not_applicable == 0 {
        return Ok(Some("no scope completed or was not applicable".to_owned()));
    }
    let incomplete = count("inaccessible")?
        .saturating_add(count("unsupported")?)
        .saturating_add(count("failed")?);
    let expected = if incomplete == 0 { 0 } else { 2 };
    if code != Some(expected) {
        return Ok(Some(format!(
            "exit code {code:?}, expected {expected} with {incomplete} incomplete scopes"
        )));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{TestResult, exit_violation};

    fn summary(complete: u64, failed: u64) -> String {
        format!(
            "resource_writes_scope=invocation resource_writes=0 rows=0 scopes=1 complete={complete} partial=0 inaccessible=0 unsupported=0 failed={failed} unobserved=0 graph_pending=0 work_remaining=0 unattributed_resources=0\nmetrics process_metrics=available\n"
        )
    }

    #[test]
    fn given_only_not_applicable_scopes_when_exited_zero_then_there_is_no_violation()
    -> TestResult<()> {
        let stdout = format!("{}outcomes_v1 not_applicable=1\n", summary(0, 0));
        assert_eq!(exit_violation(Some(0), stdout.as_bytes())?, None);
        Ok(())
    }

    #[test]
    fn given_no_completed_or_not_applicable_scope_when_exited_zero_then_it_is_a_violation()
    -> TestResult<()> {
        assert_eq!(
            exit_violation(Some(0), summary(0, 0).as_bytes())?,
            Some("no scope completed or was not applicable".to_owned())
        );
        Ok(())
    }

    #[test]
    fn given_failed_scope_when_exited_zero_then_exit_code_violates() -> TestResult<()> {
        assert_eq!(
            exit_violation(Some(0), summary(1, 1).as_bytes())?,
            Some("exit code Some(0), expected 2 with 1 incomplete scopes".to_owned())
        );
        Ok(())
    }

    #[test]
    fn given_not_applicable_beside_failed_scope_when_exited_two_then_there_is_no_violation()
    -> TestResult<()> {
        let stdout = format!("{}outcomes_v1 not_applicable=1\n", summary(0, 1));
        assert_eq!(exit_violation(Some(2), stdout.as_bytes())?, None);
        Ok(())
    }
}
