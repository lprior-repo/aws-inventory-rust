use crate::TestResult;
use std::collections::BTreeMap;

pub(super) fn exit_violation(code: Option<i32>, stdout: &[u8]) -> TestResult<Option<String>> {
    let summary = std::str::from_utf8(stdout)?.lines().next();
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
    if count("complete")? == 0 {
        return Ok(Some("no scope completed".to_owned()));
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
