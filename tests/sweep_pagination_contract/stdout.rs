use super::*;
use std::fmt::Write;

type Field = (&'static str, Option<u64>);

pub(super) fn assert(actual: &str, expected: &Summary) -> Result<()> {
    ensure!(actual.len() <= 2048, "sweep stdout exceeds golden bound");
    let (_, metrics) = actual.split_once('\n').context("summary line missing")?;
    let metrics = metrics
        .strip_suffix('\n')
        .context("final newline missing")?;
    let mut golden = String::with_capacity(actual.len());
    write_summary(&mut golden, expected)?;
    write_metrics(&mut golden, metrics, expected)?;
    assert_eq!(actual, golden);
    Ok(())
}

fn write_summary(golden: &mut String, expected: &Summary) -> Result<()> {
    let Summary {
        writes,
        rows,
        complete,
        partial,
        failed,
        pending,
        ..
    } = expected;
    writeln!(
        golden,
        "resource_writes_scope=invocation resource_writes={writes} rows={rows} scopes=1 complete={complete} partial={partial} inaccessible=0 unsupported=0 failed={failed} unobserved=0 graph_pending={pending} work_remaining=0 unattributed_resources=0"
    )?;
    Ok(())
}

fn write_metrics(golden: &mut String, actual: &str, expected: &Summary) -> Result<()> {
    let mut fields = actual.split(' ');
    assert_eq!(fields.next(), Some("metrics"));
    golden.push_str("metrics");
    for (name, expected) in expectations(expected) {
        let value = number(&mut fields, name)?;
        if let Some(expected) = expected {
            assert_eq!(value, expected, "{name}");
        }
        write!(golden, " {name}={value}")?;
    }
    write_status(golden, &mut fields)
}

fn number<'a>(fields: &mut impl Iterator<Item = &'a str>, name: &str) -> Result<u64> {
    let field = fields.next().with_context(|| format!("{name} missing"))?;
    let (key, text) = field.split_once('=').context("metric has no equals sign")?;
    assert_eq!(key, name);
    let value = text.parse::<u64>()?;
    if name == "bytes_read" {
        ensure!(value > 0, "TLS responses must contribute bytes_read");
    }
    Ok(value)
}

fn write_status<'a>(golden: &mut String, fields: &mut impl Iterator<Item = &'a str>) -> Result<()> {
    let status = fields.next().context("process_metrics missing")?;
    ensure!(
        matches!(
            status,
            "process_metrics=available" | "process_metrics=unavailable"
        ),
        "invalid process_metrics status"
    );
    assert_eq!(fields.next(), None);
    writeln!(golden, " {status}")?;
    Ok(())
}

fn expectations(expected: &Summary) -> [Field; 20] {
    [
        ("bootstrap_ms", None),
        ("seed_ms", None),
        ("drain_ms", None),
        ("api_attempts", Some(expected.requests)),
        ("api_retries", Some(0)),
        ("api_throttles", Some(0)),
        ("connections_opened", Some(expected.requests)),
        ("requests_sent", Some(expected.requests)),
        ("requests_reused", Some(0)),
        ("http2_requests", Some(0)),
        ("bytes_read", None),
        ("pages", Some(expected.pages)),
        ("decode_ns", None),
        ("store_commit_ns", None),
        ("store_commits", Some(expected.pages)),
        ("claims", Some(expected.claims)),
        ("progress_dropped", Some(0)),
        ("cpu_user_ms", None),
        ("cpu_system_ms", None),
        ("peak_rss_kb", None),
    ]
}
