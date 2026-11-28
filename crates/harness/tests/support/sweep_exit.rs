use crate::TestResult;

fn counter(line: &str, name: &str) -> TestResult<u64> {
    let mut matching = line.split_whitespace().filter_map(|token| {
        let (key, value) = token.split_once('=').unwrap_or((token, ""));
        (key == name).then_some(value)
    });
    let first = matching
        .next()
        .ok_or_else(|| format!("sweep summary lacks {name}"))?;
    if matching.next().is_some() {
        return Err(format!("duplicate sweep counter {name}").into());
    }
    first
        .parse()
        .map_err(|_| format!("invalid sweep counter {name}").into())
}

fn not_applicable_count<'a>(lines: impl Iterator<Item = &'a str>) -> TestResult<u64> {
    let mut extensions = lines.filter(|line| line.split_whitespace().next() == Some("outcomes_v1"));
    let Some(extension) = extensions.next() else {
        return Ok(0);
    };
    if extensions.next().is_some() {
        return Err("duplicate sweep outcomes_v1 line".into());
    }
    counter(extension, "not_applicable")
}

pub(super) fn exit_violation(code: Option<i32>, stdout: &[u8]) -> TestResult<Option<String>> {
    let text = std::str::from_utf8(stdout)?;
    let mut lines = text.lines();
    let summary = lines.next().ok_or("missing sweep summary")?;
    let not_applicable = not_applicable_count(lines)?;
    let count = |name| counter(summary, name);
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
    fn given_ambiguous_or_invalid_counters_when_read_then_receipt_is_rejected() {
        for extension in [
            "outcomes_v1 not_applicable=1\noutcomes_v1 not_applicable=0\n",
            "outcomes_v1 not_applicable=bad\noutcomes_v1 not_applicable=1\n",
            "outcomes_v1 not_applicable=bad not_applicable=1\n",
            "outcomes_v1 not_applicable=1 not_applicable=0\n",
            "outcomes_v1 not_applicable=bad\n",
            "outcomes_v1 not_applicable=18446744073709551616\n",
            "outcomes_v1\n",
        ] {
            let stdout = format!("{}{extension}", summary(0, 0));
            assert!(
                exit_violation(Some(0), stdout.as_bytes()).is_err(),
                "{extension}"
            );
        }
    }

    #[test]
    fn given_duplicate_legacy_count_when_read_then_receipt_is_rejected() {
        for original in ["complete=0", "complete=bad", "complete=1"] {
            let stdout = summary(1, 0).replacen("complete=1", &format!("{original} complete=1"), 1);
            assert!(
                exit_violation(Some(0), stdout.as_bytes()).is_err(),
                "{original}"
            );
        }
    }

    #[test]
    fn given_clean_legacy_summary_without_extension_when_read_then_receipt_passes() -> TestResult<()>
    {
        assert_eq!(exit_violation(Some(0), summary(1, 0).as_bytes())?, None);
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
