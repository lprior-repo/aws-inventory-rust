//! Comparison of raw benchmark tables; integer cross-products preserve exact bands.
use std::{collections::BTreeSet, path::PathBuf};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub name: String,
    pub range: Option<Range>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Range {
    pub min: u64,
    pub median: u64,
    pub max: u64,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Table {
    pub headers: Vec<(String, String)>,
    pub rows: Vec<Row>,
}
#[derive(Debug, Default)]
pub(crate) struct Options {
    pub band_percent: Option<u64>,
    pub allow_different_server: bool,
}
#[derive(Debug)]
pub(crate) struct CompareArgs {
    pub before: PathBuf,
    pub after: PathBuf,
    pub options: Options,
}
#[derive(Debug, Default)]
pub(crate) struct Report {
    pub lines: Vec<String>,
    pub regressed: bool,
}

pub(crate) fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<CompareArgs, String> {
    let words: Vec<_> = args.collect();
    let mut values = words.iter();
    let (files, options) = (0..words.len()).try_fold(
        (Vec::new(), Options::default()),
        |(mut files, mut options), _| {
            match values.next().map(String::as_str) {
                None => {}
                Some("--band") if options.band_percent.is_none() => {
                    let value = values
                        .next()
                        .ok_or("--band missing value")?
                        .parse::<u64>()
                        .map_err(|error| error.to_string())?;
                    if value > 10_000 {
                        return Err("--band exceeds 10000".to_string());
                    }
                    options.band_percent = Some(value);
                }
                Some("--allow-different-server") if !options.allow_different_server => {
                    options.allow_different_server = true
                }
                Some(flag) if flag.starts_with("--") => {
                    return Err(format!("unknown or repeated flag {flag}"));
                }
                Some(file) => files.push(PathBuf::from(file)),
            }
            Ok((files, options))
        },
    )?;
    let mut files = files.into_iter();
    match (files.next(), files.next(), files.next()) {
        (Some(before), Some(after), None) => Ok(CompareArgs {
            before,
            after,
            options,
        }),
        _ => Err("compare needs exactly <before> <after>".into()),
    }
}
pub(crate) fn run(args: &CompareArgs) -> Result<bool, String> {
    let read = |path: &PathBuf| {
        std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))
            .and_then(|text| parse_table(&text))
    };
    let report = compare(&read(&args.before)?, &read(&args.after)?, &args.options)?;
    report.lines.iter().for_each(|line| println!("{line}"));
    Ok(!report.regressed)
}
pub(crate) fn parse_table(text: &str) -> Result<Table, String> {
    let (table, columns) = text.lines().filter(|line| !line.is_empty()).try_fold(
        (Table::default(), false),
        |(mut table, columns), line| {
            if !columns {
                if line == "measurement\tmin\tmedian\tmax" {
                    return Ok((table, true));
                }
                let (key, value) = line
                    .strip_prefix("# ")
                    .and_then(|header| header.split_once('\t'))
                    .ok_or("expected header or column line")?;
                if table.headers.iter().any(|(known, _)| known == key) {
                    return Err("repeated header".to_string());
                }
                table.headers.push((key.into(), value.into()));
            } else {
                let row = parse_row(line)?;
                if table.rows.iter().any(|known| known.name == row.name) {
                    return Err("repeated measurement".into());
                }
                table.rows.push(row);
            }
            Ok((table, columns))
        },
    )?;
    if !columns || table.rows.is_empty() {
        return Err("missing measurements".into());
    }
    Ok(table)
}
fn parse_row(line: &str) -> Result<Row, String> {
    let mut fields = line.split('\t');
    let (Some(name), Some(min), Some(median), Some(max), None) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        return Err("expected four tab-separated fields".into());
    };
    if name.is_empty() {
        return Err("empty measurement name".into());
    }
    if [min, median, max] == ["unavailable"; 3] {
        return Ok(Row {
            name: name.into(),
            range: None,
        });
    }
    let number = |text: &str| text.parse::<u64>().map_err(|error| error.to_string());
    let range = Range {
        min: number(min)?,
        median: number(median)?,
        max: number(max)?,
    };
    if range.min > range.median || range.median > range.max {
        return Err("invalid measurement range".into());
    }
    Ok(Row {
        name: name.into(),
        range: Some(range),
    })
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Worse {
    Higher,
    Lower,
    Neither,
}
fn direction(name: &str) -> Worse {
    match name.split('.').next() {
        Some("P1" | "P2" | "P3" | "P6" | "P7" | "P8") => Worse::Higher,
        Some("P5" | "P9") => Worse::Lower,
        _ => Worse::Neither,
    }
}
fn regressed(before: &Row, after: &Row, options: &Options) -> bool {
    let (Some(before_range), Some(after_range)) = (before.range, after.range) else {
        return false;
    };
    let worse = u128::from(match direction(&before.name) {
        Worse::Higher => after_range.median.saturating_sub(before_range.median),
        Worse::Lower => before_range.median.saturating_sub(after_range.median),
        Worse::Neither => return false,
    });
    let scaled = worse.saturating_mul(100);
    let median = u128::from(before_range.median);
    match options.band_percent {
        Some(percent) => scaled > u128::from(percent).saturating_mul(median),
        None => {
            scaled > 5_u128.saturating_mul(median)
                && worse
                    > u128::from(before_range.max.saturating_sub(before_range.min))
                        .saturating_mul(2)
        }
    }
}
fn change(before: u64, after: u64) -> String {
    if before == after {
        return "+0.00%".into();
    }
    let Some(points) = u128::from(before.abs_diff(after))
        .saturating_mul(10_000)
        .checked_div(u128::from(before))
    else {
        return "n/a".into();
    };
    let sign = if after > before { '+' } else { '-' };
    format!("{sign}{}.{:02}%", points / 100, points % 100)
}
fn settings(table: &Table) -> BTreeSet<(&str, &str)> {
    table
        .headers
        .iter()
        .filter(|(key, _)| {
            key.starts_with("server.")
                || matches!(
                    key.as_str(),
                    "profile"
                        | "proxy"
                        | "server_kind"
                        | "volume"
                        | "services"
                        | "os"
                        | "build"
                        | "protocol"
                )
        })
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect()
}
pub(crate) fn compare(before: &Table, after: &Table, options: &Options) -> Result<Report, String> {
    let mut report = Report::default();
    if settings(before) != settings(after) {
        if !options.allow_different_server {
            return Err(
                "different profile or server settings; pass --allow-different-server".into(),
            );
        }
        report
            .lines
            .push("note: different profile or server settings; numbers are not comparable".into());
    }
    if let Some(extra) = after
        .rows
        .iter()
        .find(|row| !before.rows.iter().any(|known| known.name == row.name))
    {
        return Err(format!("measurement {} missing from before", extra.name));
    }
    report
        .lines
        .push("measurement\tbefore\tafter\tchange\tverdict".into());
    before.rows.iter().try_fold(report, |mut report, row| {
        let found = after
            .rows
            .iter()
            .find(|candidate| candidate.name == row.name)
            .ok_or_else(|| format!("measurement {} missing from after", row.name))?;
        let baseline = row
            .range
            .ok_or_else(|| format!("measurement {} unavailable in before", row.name))?;
        let measured = found
            .range
            .ok_or_else(|| format!("measurement {} unavailable in after", row.name))?;
        let verdict = match (direction(&row.name), regressed(row, found, options)) {
            (Worse::Neither, _) => "info",
            (_, true) => "regressed",
            (_, false) => "ok",
        };
        report.regressed |= verdict == "regressed";
        report.lines.push(format!(
            "{}\t{}\t{}\t{}\t{verdict}",
            row.name,
            baseline.median,
            measured.median,
            change(baseline.median, measured.median)
        ));
        Ok(report)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(name: &str, min: u64, median: u64, max: u64) -> Row {
        Row {
            name: name.into(),
            range: Some(Range { min, median, max }),
        }
    }
    #[test]
    fn bands_and_directions_are_exact() {
        ["P1", "P2", "P3", "P6", "P7", "P8"]
            .iter()
            .for_each(|name| {
                let before = row(name, 90, 100, 110);
                assert!(!regressed(
                    &before,
                    &row(name, 140, 140, 140),
                    &Options::default()
                ));
                assert!(regressed(
                    &before,
                    &row(name, 141, 141, 141),
                    &Options::default()
                ));
                let fixed = Options {
                    band_percent: Some(5),
                    allow_different_server: false,
                };
                assert!(!regressed(&before, &row(name, 105, 105, 105), &fixed));
                assert!(regressed(&before, &row(name, 106, 106, 106), &fixed));
            });
        ["P5", "P9"].iter().for_each(|name| {
            assert!(regressed(
                &row(name, 100, 100, 100),
                &row(name, 94, 94, 94),
                &Options::default()
            ))
        });
        assert!(!regressed(
            &row("P4", 1, 1, 1),
            &row("P4", 9, 9, 9),
            &Options::default()
        ));
    }
    #[test]
    fn settings_and_missing_measurements_are_refused() -> Result<(), String> {
        let before =
            parse_table("# profile\tthin\nmeasurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\n")?;
        let after = parse_table("# profile\tfull\nmeasurement\tmin\tmedian\tmax\nP2.x\t1\t1\t1\n")?;
        assert!(compare(&before, &after, &Options::default()).is_err());
        assert!(
            compare(
                &before,
                &after,
                &Options {
                    allow_different_server: true,
                    band_percent: None
                }
            )
            .is_err()
        );
        let mut same =
            parse_table("# profile\tfull\nmeasurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\n")?;
        let allowed = Options {
            allow_different_server: true,
            band_percent: None,
        };
        assert!(
            compare(&before, &same, &allowed)?
                .lines
                .first()
                .is_some_and(|line| line.contains("not comparable"))
        );
        same.headers = before.headers.clone();
        assert!(!compare(&before, &same, &Options::default())?.regressed);
        same.headers.push(("os".into(), "macOS".into()));
        assert!(compare(&before, &same, &Options::default()).is_err());
        Ok(())
    }
    #[test]
    fn changed_server_setting_requires_override() -> Result<(), String> {
        let before = parse_table(
            "# server.shared_buffers\t128MB\nmeasurement\tmin\tmedian\tmax\nP1.ms\t100\t100\t100\n",
        )?;
        let after = parse_table(
            "# server.shared_buffers\t256MB\nmeasurement\tmin\tmedian\tmax\nP1.ms\t100\t100\t100\n",
        )?;
        let error = compare(&before, &after, &Options::default())
            .err()
            .ok_or("different settings accepted")?;
        assert!(error.contains("server settings"));
        let allowed = Options {
            band_percent: None,
            allow_different_server: true,
        };
        let report = compare(&before, &after, &allowed)?;
        assert!(!report.regressed);
        assert!(
            report
                .lines
                .iter()
                .any(|line| line.contains("not comparable"))
        );
        Ok(())
    }
    #[test]
    fn missing_after_measurement_is_named() -> Result<(), String> {
        let before = parse_table(
            "measurement\tmin\tmedian\tmax\nP1.ms\t1\t1\t1\nP5.resources_per_second\t2\t2\t2\n",
        )?;
        let after = parse_table("measurement\tmin\tmedian\tmax\nP1.ms\t1\t1\t1\n")?;
        let error = compare(&before, &after, &Options::default())
            .err()
            .ok_or("missing measurement accepted")?;
        assert_eq!(
            error,
            "measurement P5.resources_per_second missing from after"
        );
        Ok(())
    }
    #[test]
    fn comparison_arguments_and_changes_cover_boundaries() -> Result<(), String> {
        let parse = |words: &[&str]| parse_args(&mut words.iter().map(|word| (*word).to_owned()));
        let args = parse(&["a", "b", "--band", "7", "--allow-different-server"])?;
        assert_eq!(args.options.band_percent, Some(7));
        assert!(args.options.allow_different_server);
        [
            &["a", "b", "--band"][..],
            &["a", "b", "--band", "10001"],
            &["a", "b", "--unknown"],
            &["a", "b", "--band", "1", "--band", "2"],
            &[
                "a",
                "b",
                "--allow-different-server",
                "--allow-different-server",
            ],
        ]
        .iter()
        .for_each(|words| assert!(parse(words).is_err()));
        assert_eq!(change(0, 1), "n/a");
        assert_eq!(change(0, 0), "+0.00%");
        assert_eq!(change(3, 4), "+33.33%");
        assert_eq!(change(3, 2), "-33.33%");
        Ok(())
    }
    #[test]
    fn malformed_tables_are_rejected() {
        [
            "# profile\tthin\n# profile\tthin\nmeasurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\n",
            "measurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\nP1.x\t1\t1\t1\n",
            "measurement\tmin\tmedian\tmax\nP1.x\t2\t1\t3\n",
            "measurement\tmin\tmedian\tmax\nP1.x\t1\t3\t2\n",
            "measurement\tmin\tmedian\tmax\nP1.x\t1\t1\n",
            "measurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\t2\n",
            "measurement\tmin\tmedian\tmax\nP1.x\tx\t1\t1\n",
        ]
        .iter()
        .for_each(|text| assert!(parse_table(text).is_err()));
    }
    #[test]
    fn build_and_measured_protocol_are_comparison_settings() -> Result<(), String> {
        ["build", "protocol"].iter().try_for_each(|key| {
            let before = parse_table(&format!(
                "# {key}\tfirst\nmeasurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\n"
            ))?;
            let after = parse_table(&format!(
                "# {key}\tsecond\nmeasurement\tmin\tmedian\tmax\nP1.x\t1\t1\t1\n"
            ))?;
            assert!(
                compare(&before, &after, &Options::default()).is_err(),
                "{key}"
            );
            let report = compare(
                &before,
                &after,
                &Options {
                    allow_different_server: true,
                    band_percent: None,
                },
            )?;
            assert!(
                report
                    .lines
                    .iter()
                    .any(|line| line.contains("not comparable"))
            );
            Ok::<_, String>(())
        })
    }
    #[test]
    fn unavailable_ranges_round_trip_but_are_not_compared() -> Result<(), String> {
        let unavailable = parse_table(
            "measurement\tmin\tmedian\tmax\nP5.x\tunavailable\tunavailable\tunavailable\n",
        )?;
        assert_eq!(unavailable.rows.first().ok_or("missing row")?.range, None);
        let numeric = parse_table("measurement\tmin\tmedian\tmax\nP5.x\t1\t1\t1\n")?;
        assert!(
            compare(&unavailable, &numeric, &Options::default())
                .err()
                .ok_or("unavailable compared")?
                .contains("P5.x unavailable in before")
        );
        assert!(
            compare(&numeric, &unavailable, &Options::default())
                .err()
                .ok_or("unavailable compared")?
                .contains("P5.x unavailable in after")
        );
        assert!(parse_table("measurement\tmin\tmedian\tmax\nP5.x\tunavailable\t1\t1\n").is_err());
        Ok(())
    }
}
