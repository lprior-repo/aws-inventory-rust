use super::summary::SweepLines;
use crate::compare::{Range, Row, Table};
#[derive(Debug)]
pub(crate) struct Sample {
    pub cold_wall_ms: u64,
    pub warm_wall_ms: u64,
    pub cold: SweepLines,
    pub warm: SweepLines,
    pub cold_database_bytes: u64,
    pub cold_wal_bytes: u64,
    pub warm_wal_bytes: u64,
    pub warm_rows_logged: u64,
    pub warm_rows_unlogged: u64,
}
pub(crate) fn spread(name: &str, values: &[u64]) -> Result<Row, String> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let min = sorted.first().copied().ok_or("no samples")?;
    let max = sorted.last().copied().ok_or("no samples")?;
    let middle = sorted.len() / 2;
    let upper = sorted.get(middle).copied().ok_or("missing median")?;
    let median = if sorted.len().is_multiple_of(2) {
        let lower = middle
            .checked_sub(1)
            .and_then(|index| sorted.get(index))
            .copied()
            .ok_or("missing lower median")?;
        lower
            .checked_add(upper.saturating_sub(lower) / 2)
            .ok_or("median overflow")?
    } else {
        upper
    };
    Ok(Row {
        name: name.into(),
        range: Some(Range { min, median, max }),
    })
}
fn ratio(numerator: u64, scale: u128, denominator: u64) -> Result<Option<u64>, String> {
    if denominator == 0 {
        return Ok(None);
    }
    let result = u128::from(numerator)
        .checked_mul(scale)
        .and_then(|number| number.checked_div(u128::from(denominator)))
        .ok_or("measurement ratio overflow")?;
    u64::try_from(result)
        .map(Some)
        .map_err(|error| error.to_string())
}
fn values(sample: &Sample) -> Result<Vec<(&'static str, Option<u64>)>, String> {
    let get = |key| sample.cold.get(key);
    if !sample.cold.process_metrics_available || !sample.warm.process_metrics_available {
        return Err(
            "collector process metrics unavailable; published tables require Linux /proc".into(),
        );
    }
    Ok(vec![
        ("P1.cold_wall_ms", Some(sample.cold_wall_ms)),
        ("P2.warm_wall_ms", Some(sample.warm_wall_ms)),
        ("P3.cold_api_attempts", Some(get("api_attempts")?)),
        ("P3.cold_api_retries", Some(get("api_retries")?)),
        ("P3.cold_api_throttles", Some(get("api_throttles")?)),
        (
            "P4.cold_connections_opened",
            Some(get("connections_opened")?),
        ),
        ("P4.cold_requests_reused", Some(get("requests_reused")?)),
        ("P4.cold_distinct_hosts", Some(get("hosts")?)),
        (
            "P4.cold_connect_attempts_incl_permit_wait_and_failures_ppm_of_wall",
            ratio(get("handshake_ns")?, 1, sample.cold_wall_ms)?,
        ),
        (
            "P5.cold_resources_per_second",
            ratio(get("rows")?, 1_000, sample.cold_wall_ms)?,
        ),
        (
            "P6.cold_cpu_ms",
            Some(
                get("cpu_user_ms")?
                    .checked_add(get("cpu_system_ms")?)
                    .ok_or("CPU overflow")?,
            ),
        ),
        ("P7.cold_peak_rss_kb", Some(get("peak_rss_kb")?)),
        ("P8.cold_database_bytes", Some(sample.cold_database_bytes)),
        ("P8.cold_wal_bytes", Some(sample.cold_wal_bytes)),
        ("P8.warm_wal_bytes", Some(sample.warm_wal_bytes)),
        ("P8.warm_rows_logged", Some(sample.warm_rows_logged)),
        ("P8.warm_rows_unlogged", Some(sample.warm_rows_unlogged)),
        ("P9.cold_store_commits", Some(get("store_commits")?)),
        (
            "P9.cold_writer_rows_per_second",
            ratio(get("rows")?, 1_000_000_000, get("store_commit_ns")?)?,
        ),
        (
            "P10.cold_decode_ppm_of_user_cpu",
            ratio(get("decode_ns")?, 1, get("cpu_user_ms")?)?,
        ),
    ])
}
pub(crate) fn rows(samples: &[Sample]) -> Result<Vec<Row>, String> {
    let values = samples.iter().map(values).collect::<Result<Vec<_>, _>>()?;
    let first = values.first().ok_or("no runs")?;
    first
        .iter()
        .enumerate()
        .map(|(index, (name, _))| {
            let column = values
                .iter()
                .map(|row| row.get(index).map(|(_, value)| *value))
                .collect::<Option<Vec<_>>>()
                .ok_or("missing sample")?;
            match column.into_iter().collect::<Option<Vec<_>>>() {
                Some(column) => spread(name, &column),
                None => Ok(Row {
                    name: (*name).into(),
                    range: None,
                }),
            }
        })
        .collect()
}
pub(crate) fn render(table: &Table) -> String {
    let headers = table
        .headers
        .iter()
        .map(|(key, value)| format!("# {key}\t{}\n", value.replace(['\t', '\n', '\r'], " ")))
        .collect::<String>();
    table.rows.iter().fold(
        format!("{headers}measurement\tmin\tmedian\tmax\n"),
        |mut text, row| {
            match row.range {
                Some(range) => text.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    row.name, range.min, range.median, range.max
                )),
                None => text.push_str(&format!(
                    "{}\tunavailable\tunavailable\tunavailable\n",
                    row.name
                )),
            }
            text
        },
    )
}
pub(super) async fn publish(table: &Table, out: Option<&std::path::Path>) -> Result<(), String> {
    let output = render(table);
    if table
        .headers
        .iter()
        .any(|(key, value)| key == "collector.success" && value == "yes")
        && let Some(path) = out
    {
        tokio::fs::write(path, &output)
            .await
            .map_err(|error| error.to_string())?;
    }
    print!("{output}");
    Ok(())
}
pub(super) fn protocol(samples: &[Sample]) -> Result<&'static str, String> {
    let (http1, http2) = samples
        .iter()
        .flat_map(|sample| [&sample.cold, &sample.warm])
        .try_fold((false, false), |(http1, http2), lines| {
            let requests = lines.get("requests_sent")?;
            let h2 = lines.get("http2_requests")?;
            if h2 > requests {
                return Err("HTTP/2 requests exceed all requests".to_string());
            }
            Ok((http1 || requests > h2, http2 || h2 != 0))
        })?;
    Ok(match (http1, http2) {
        (true, false) => "HTTP/1.1",
        (false, true) => "HTTP/2",
        (true, true) => "mixed",
        (false, false) => "unavailable",
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn median_of_one_two_three_and_max_values() -> Result<(), String> {
        assert_eq!(
            spread("x", &[7])?,
            Row {
                name: "x".into(),
                range: Some(Range {
                    min: 7,
                    median: 7,
                    max: 7
                }),
            }
        );
        assert_eq!(
            spread("x", &[4, 1])?.range.ok_or("missing range")?.median,
            2
        );
        assert_eq!(
            spread("x", &[9, 1, 5])?
                .range
                .ok_or("missing range")?
                .median,
            5
        );
        assert_eq!(
            spread("x", &[u64::MAX, u64::MAX])?
                .range
                .ok_or("missing range")?
                .median,
            u64::MAX
        );
        assert!(spread("x", &[]).is_err());
        Ok(())
    }
    #[test]
    fn rates_use_the_steps_per_second_units() -> Result<(), String> {
        let sample = fixed_sample(1)?;
        let values = values(&sample)?;
        assert!(values.contains(&("P5.cold_resources_per_second", Some(200))));
        assert!(values.contains(&("P9.cold_writer_rows_per_second", Some(4000))));
        assert_eq!(ratio(3_000_000, 1, 300)?, Some(10_000));
        assert_eq!(ratio(1, 1, 0)?, None);
        Ok(())
    }
    #[test]
    fn raw_table_round_trips() -> Result<(), String> {
        let table = Table {
            headers: vec![("profile".into(), "thin".into())],
            rows: vec![spread("P1.x", &[1, 2, 3])?],
        };
        assert_eq!(crate::compare::parse_table(&render(&table))?, table);
        Ok(())
    }
    fn fixed_sample(n: u64) -> Result<Sample, String> {
        let times = |value: u64| value.checked_mul(n).ok_or("fixture overflow".to_string());
        let rows = times(400)?;
        let attempts = times(12)?;
        let retries = times(2)?;
        let connections = times(3)?;
        let reused = times(9)?;
        let user = times(100)?;
        let system = times(20)?;
        let rss = times(1000)?;
        let commit_ns = times(100_000_000)?;
        let handshake_ns = times(24_000_000)?;
        let hosts = times(2)?;
        let commits = times(4)?;
        let decode_ns = times(10_000_000)?;
        let summary = format!(
            "resource_writes_scope=invocation resource_writes={rows} rows={rows} scopes=1 complete=1 partial=0 inaccessible=0 unsupported=0 failed=0 unobserved=0 graph_pending=0 work_remaining=0 unattributed_resources=0\nmetrics bootstrap_ms=1 seed_ms=1 drain_ms=1 api_attempts={attempts} api_retries={retries} api_throttles={n} connections_opened={connections} requests_sent={attempts} requests_reused={reused} http2_requests=0 bytes_read=1 handshake_ns={handshake_ns} hosts={hosts} pages={commits} decode_ns={decode_ns} store_commit_ns={commit_ns} store_commits={commits} claims=1 progress_dropped=0 child_requests_skipped=0 cpu_user_ms={user} cpu_system_ms={system} peak_rss_kb={rss} process_metrics=available"
        );
        Ok(Sample {
            cold_wall_ms: times(2000)?,
            warm_wall_ms: times(1000)?,
            cold: super::super::summary::parse(&summary)?,
            warm: super::super::summary::parse(&summary)?,
            cold_database_bytes: times(8000)?,
            cold_wal_bytes: times(6000)?,
            warm_wal_bytes: times(3000)?,
            warm_rows_logged: times(7)?,
            warm_rows_unlogged: times(9)?,
        })
    }
    #[test]
    fn fixed_runs_map_every_p_row_to_hand_computed_medians() -> Result<(), String> {
        let rows = rows(&[fixed_sample(3)?, fixed_sample(1)?, fixed_sample(2)?])?;
        let expected = [
            ("P1.cold_wall_ms", 4000),
            ("P2.warm_wall_ms", 2000),
            ("P3.cold_api_attempts", 24),
            ("P3.cold_api_retries", 4),
            ("P3.cold_api_throttles", 2),
            ("P4.cold_connections_opened", 6),
            ("P4.cold_requests_reused", 18),
            ("P4.cold_distinct_hosts", 4),
            (
                "P4.cold_connect_attempts_incl_permit_wait_and_failures_ppm_of_wall",
                12_000,
            ),
            ("P5.cold_resources_per_second", 200),
            ("P6.cold_cpu_ms", 240),
            ("P7.cold_peak_rss_kb", 2000),
            ("P8.cold_database_bytes", 16000),
            ("P8.cold_wal_bytes", 12000),
            ("P8.warm_wal_bytes", 6000),
            ("P8.warm_rows_logged", 14),
            ("P8.warm_rows_unlogged", 18),
            ("P9.cold_store_commits", 8),
            ("P9.cold_writer_rows_per_second", 4000),
            ("P10.cold_decode_ppm_of_user_cpu", 100000),
        ];
        assert_eq!(rows.len(), expected.len());
        expected.iter().try_for_each(|(name, median)| {
            let row = rows.iter().find(|row| row.name == *name).ok_or(*name)?;
            assert_eq!(row.range.ok_or("missing range")?.median, *median, "{name}");
            Ok::<_, String>(())
        })
    }
    #[test]
    fn unavailable_process_metrics_never_become_numbers() -> Result<(), String> {
        let mut sample = fixed_sample(1)?;
        sample.cold.process_metrics_available = false;
        assert!(rows(&[sample]).is_err());
        Ok(())
    }
    #[test]
    fn zero_denominators_render_unavailable_without_losing_other_rows() -> Result<(), String> {
        let sample = fixed_sample(0)?;
        let rows = rows(&[sample])?;
        let output = render(&Table {
            headers: Vec::new(),
            rows,
        });
        assert!(
            output.contains("P5.cold_resources_per_second\tunavailable\tunavailable\tunavailable")
        );
        assert!(
            output
                .contains("P9.cold_writer_rows_per_second\tunavailable\tunavailable\tunavailable")
        );
        assert!(
            output
                .contains("P10.cold_decode_ppm_of_user_cpu\tunavailable\tunavailable\tunavailable")
        );
        assert!(output.contains("P1.cold_wall_ms\t0\t0\t0"));
        Ok(())
    }
    #[tokio::test]
    async fn failed_collector_table_is_printable_but_never_written() -> Result<(), String> {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let path = directory.path().join("refused.tsv");
        let mut table = Table {
            headers: vec![("collector.success".into(), "no".into())],
            rows: vec![spread("P1.x", &[1])?],
        };
        publish(&table, Some(&path)).await?;
        assert!(!path.exists());
        table.headers = vec![("collector.success".into(), "yes".into())];
        publish(&table, Some(&path)).await?;
        assert_eq!(
            tokio::fs::read_to_string(path)
                .await
                .map_err(|error| error.to_string())?,
            render(&table)
        );
        Ok(())
    }
}
