use std::collections::BTreeMap;

const OUTCOME: &[&str] = &[
    "resource_writes",
    "rows",
    "scopes",
    "complete",
    "partial",
    "inaccessible",
    "unsupported",
    "failed",
    "unobserved",
    "graph_pending",
    "work_remaining",
    "unattributed_resources",
];
const METRICS: &[&str] = &[
    "bootstrap_ms",
    "seed_ms",
    "drain_ms",
    "api_attempts",
    "api_retries",
    "api_throttles",
    "connections_opened",
    "requests_sent",
    "requests_reused",
    "http2_requests",
    "bytes_read",
    "handshake_ns",
    "hosts",
    "pages",
    "decode_ns",
    "store_commit_ns",
    "store_commits",
    "claims",
    "progress_dropped",
    "child_requests_skipped",
    "cpu_user_ms",
    "cpu_system_ms",
    "peak_rss_kb",
];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SweepLines {
    numbers: BTreeMap<String, u64>,
    pub process_metrics_available: bool,
}
impl SweepLines {
    pub(crate) fn get(&self, key: &str) -> Result<u64, String> {
        self.numbers
            .get(key)
            .copied()
            .ok_or_else(|| format!("missing key {key}"))
    }
}
fn line<'a>(output: &'a str, mark: &str) -> Result<&'a str, String> {
    let mut found = output
        .lines()
        .filter(|line| line.split_whitespace().next() == Some(mark));
    let first = found.next().ok_or_else(|| format!("{mark} line missing"))?;
    if found.next().is_some() {
        return Err(format!("{mark} line repeated"));
    }
    Ok(first)
}
fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    line.split_whitespace()
        .skip(1)
        .try_fold(BTreeMap::new(), |mut fields, token| {
            let (key, value) = token
                .split_once('=')
                .ok_or_else(|| format!("token without '=': {token}"))?;
            if fields.insert(key, value).is_some() {
                return Err(format!("repeated key {key}"));
            }
            Ok(fields)
        })
}
fn numbers(fields: &BTreeMap<&str, &str>, keys: &[&str]) -> Result<BTreeMap<String, u64>, String> {
    keys.iter()
        .map(|key| {
            let value = fields
                .get(key)
                .ok_or_else(|| format!("missing key {key}"))?;
            let number = value
                .parse::<u64>()
                .map_err(|_| format!("{key} is not a number: {value}"))?;
            Ok(((*key).to_owned(), number))
        })
        .collect()
}
pub(crate) fn parse(output: &str) -> Result<SweepLines, String> {
    let outcome = fields(line(output, "resource_writes_scope=invocation")?)?;
    let metrics = fields(line(output, "metrics")?)?;
    let process_metrics_available = match metrics.get("process_metrics") {
        Some(&"available") => true,
        Some(&"unavailable") => false,
        _ => return Err("missing or invalid process_metrics".into()),
    };
    let mut all = numbers(&outcome, OUTCOME)?;
    all.extend(numbers(&metrics, METRICS)?);
    Ok(SweepLines {
        numbers: all,
        process_metrics_available,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    pub(crate) fn valid() -> String {
        let fields = |keys: &[&str]| {
            keys.iter()
                .map(|key| format!(" {key}=1"))
                .collect::<String>()
        };
        format!(
            "resource_writes_scope=invocation{}\nmetrics{} process_metrics=available\n",
            fields(OUTCOME),
            fields(METRICS)
        )
    }
    #[test]
    fn every_required_key_is_checked() -> Result<(), String> {
        assert_eq!(parse(&valid())?.get("rows")?, 1);
        OUTCOME
            .iter()
            .chain(METRICS)
            .try_for_each(|key| -> Result<(), String> {
                let missing = valid().replace(&format!(" {key}=1"), "");
                assert_eq!(parse(&missing).err(), Some(format!("missing key {key}")));
                Ok(())
            })
    }
    #[test]
    fn malformed_or_repeated_lines_and_keys_are_rejected() {
        let text = valid();
        assert!(parse(&format!("{text}{text}")).is_err());
        assert!(parse(&text.replace(" rows=1", " rows=no")).is_err());
        assert!(parse(&text.replace(" rows=1", " rows=1 rows=2")).is_err());
        assert!(parse("metrics").is_err());
    }
    #[test]
    fn unavailable_process_marker_is_preserved() -> Result<(), String> {
        let output = valid().replace("process_metrics=available", "process_metrics=unavailable");
        assert!(!parse(&output)?.process_metrics_available);
        assert!(
            parse(&output.replace("process_metrics=unavailable", "process_metrics=invalid"))
                .is_err()
        );
        Ok(())
    }
}
