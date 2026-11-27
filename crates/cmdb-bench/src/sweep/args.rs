use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Profile {
    Thin,
    Full,
    Proxy,
}
impl Profile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Thin => "thin",
            Self::Full => "full",
            Self::Proxy => "proxy",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProxySpec {
    pub latency_ms: u64,
    pub rps: u32,
}
#[derive(Debug)]
pub(crate) struct SweepArgs {
    pub collector: PathBuf,
    pub profile: Profile,
    pub runs: usize,
    pub proxy: Option<ProxySpec>,
    pub out: Option<PathBuf>,
    pub catalog: PathBuf,
    pub work: Option<PathBuf>,
    pub server: String,
    pub volume: String,
    pub build: String,
    pub services: Vec<String>,
    pub concurrency: Option<usize>,
    pub idle_timeout_ms: Option<u64>,
    pub pool_idle_per_host: Option<usize>,
    pub max_rps_milli: Option<u32>,
}

pub(crate) fn parse_proxy(text: &str) -> Result<ProxySpec, String> {
    let pairs = text
        .split(',')
        .map(|part| part.split_once('=').ok_or("proxy part lacks '='"))
        .collect::<Result<Vec<_>, _>>()?;
    if pairs.len() != 2 {
        return Err("proxy needs exactly latency_ms and rps".into());
    }
    let get = |key| {
        pairs
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| *value)
            .ok_or_else(|| format!("proxy key {key} missing"))
    };
    let latency_ms = get("latency_ms")?
        .parse::<u64>()
        .map_err(|error| error.to_string())?;
    let rps = get("rps")?
        .parse::<u32>()
        .map_err(|error| error.to_string())?;
    if latency_ms > 60_000 || rps == 0 || rps > 100_000 {
        return Err("proxy requires latency_ms <= 60000 and rps in 1..=100000".into());
    }
    Ok(ProxySpec { latency_ms, rps })
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> Result<SweepArgs, String> {
    let words: Vec<_> = args.collect();
    let (mut options, services) = options(&words)?;
    let collector = options
        .remove("--collector")
        .ok_or("missing --collector")?
        .into();
    let profile = parse_profile(options.remove("--profile").ok_or("missing --profile")?)?;
    let runs = options
        .remove("--runs")
        .map_or(Ok(3), |text| text.parse::<usize>())
        .map_err(|error| error.to_string())?;
    if !(1..=100).contains(&runs) {
        return Err("--runs must be in 1..=100".into());
    }
    let settings = parse_settings(&mut options, profile)?;
    let out = options.remove("--out").map(PathBuf::from);
    let work = options.remove("--work").map(PathBuf::from);
    let catalog = options.remove("--catalog").map_or_else(
        || inventory_harness::e2e::workspace_root().join("catalog/aws-api.catalog"),
        PathBuf::from,
    );
    let server = choice(
        options.remove("--server"),
        &["container", "native", "managed"],
    )?;
    let volume = choice(
        options.remove("--volume"),
        &["named", "bind", "native-disk"],
    )?;
    let build = choice(options.remove("--build"), &["release", "debug"])?;
    if !options.is_empty() {
        return Err(format!("unknown options: {options:?}"));
    }
    Ok(SweepArgs {
        collector,
        profile,
        runs,
        proxy: settings.proxy,
        out,
        catalog,
        work,
        server,
        volume,
        build,
        services,
        concurrency: settings.concurrency,
        idle_timeout_ms: settings.idle_timeout_ms,
        pool_idle_per_host: settings.pool_idle_per_host,
        max_rps_milli: settings.max_rps_milli,
    })
}

struct Settings {
    proxy: Option<ProxySpec>,
    concurrency: Option<usize>,
    idle_timeout_ms: Option<u64>,
    pool_idle_per_host: Option<usize>,
    max_rps_milli: Option<u32>,
}

fn parse_settings(
    options: &mut std::collections::BTreeMap<&str, &str>,
    profile: Profile,
) -> Result<Settings, String> {
    let concurrency = options
        .remove("--concurrency")
        .map(str::parse::<usize>)
        .transpose()
        .map_err(|error| error.to_string())?;
    if concurrency.is_some_and(|value| !(1..=4_096).contains(&value)) {
        return Err("--concurrency must be in 1..=4096".into());
    }
    let idle_timeout_ms = options
        .remove("--idle-timeout-ms")
        .map(str::parse::<u64>)
        .transpose()
        .map_err(|error| error.to_string())?;
    if idle_timeout_ms.is_some_and(|value| !(1..=90_000).contains(&value)) {
        return Err("--idle-timeout-ms must be in 1..=90000".into());
    }
    let pool_idle_per_host = options
        .remove("--pool-idle-per-host")
        .map(str::parse::<usize>)
        .transpose()
        .map_err(|error| error.to_string())?;
    if pool_idle_per_host
        .is_some_and(|value| value == 0 || concurrency.is_some_and(|limit| value > limit))
    {
        return Err("--pool-idle-per-host must be at least 1 and at most --concurrency".into());
    }
    let max_rps_milli = options
        .remove("--max-rps")
        .map(parse_milli_rps)
        .transpose()?;
    let proxy = options
        .remove("--proxy")
        .map(parse_proxy)
        .transpose()?
        .or_else(|| {
            (profile == Profile::Proxy).then_some(ProxySpec {
                latency_ms: 50,
                rps: 20,
            })
        });
    Ok(Settings {
        proxy,
        concurrency,
        idle_timeout_ms,
        pool_idle_per_host,
        max_rps_milli,
    })
}

fn parse_milli_rps(value: &str) -> Result<u32, String> {
    match value.split_once('.') {
        Some(("0", fraction))
            if (1..=3).contains(&fraction.len())
                && fraction.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            let parsed = fraction.parse::<u32>().map_err(|error| error.to_string())?;
            let scale: u32 = match fraction.len() {
                1 => 100,
                2 => 10,
                _ => 1,
            };
            let milli = parsed
                .checked_mul(scale)
                .ok_or_else(|| "--max-rps is out of range".to_owned())?;
            if !(4..=999).contains(&milli) {
                return Err("--max-rps fraction must be in 0.004..=0.999".into());
            }
            Ok(milli)
        }
        Some(_) => Err("--max-rps accepts whole rates or rates below 1.0".into()),
        None => {
            let whole = value.parse::<u32>().map_err(|error| error.to_string())?;
            if !(1..=100_000).contains(&whole) {
                return Err("--max-rps must be in 1..=100000".into());
            }
            whole
                .checked_mul(1_000)
                .ok_or_else(|| "--max-rps is out of range".into())
        }
    }
}

pub(crate) fn collector_options(args: &SweepArgs) -> Vec<String> {
    [
        (
            "--concurrency",
            args.concurrency.map(|value| value.to_string()),
        ),
        (
            "--idle-timeout-ms",
            args.idle_timeout_ms.map(|value| value.to_string()),
        ),
        (
            "--pool-idle-per-host",
            args.pool_idle_per_host.map(|value| value.to_string()),
        ),
        ("--max-rps", args.max_rps_milli.map(rate_text)),
    ]
    .into_iter()
    .filter_map(|(name, value)| value.map(|value| [name.to_owned(), value]))
    .flatten()
    .collect()
}

pub(crate) fn setting_text<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| "default".into(), |value| value.to_string())
}

const CLEARED_ENVIRONMENT: [&str; 2] = [
    "INVENTORY_SWEEP_IDLE_TIMEOUT_MS",
    "INVENTORY_SWEEP_POOL_IDLE_PER_HOST",
];
const RECORDED_ENVIRONMENT: [(&str, &str); 2] = [
    ("postgres_copy_min_rows", "INVENTORY_POSTGRES_COPY_MIN_ROWS"),
    ("postgres_copy_direct", "INVENTORY_POSTGRES_COPY_DIRECT"),
];

pub(crate) fn apply_collector_options(command: &mut tokio::process::Command, args: &SweepArgs) {
    CLEARED_ENVIRONMENT.iter().for_each(|name| {
        command.env_remove(name);
    });
    command.args(collector_options(args));
}

pub(crate) fn environment_headers(read: impl Fn(&str) -> Option<String>) -> Vec<(String, String)> {
    RECORDED_ENVIRONMENT
        .iter()
        .map(|(header, name)| {
            (
                (*header).to_owned(),
                read(name).unwrap_or_else(|| "unset".into()),
            )
        })
        .collect()
}

pub(crate) fn rate_text(milli: u32) -> String {
    if milli.is_multiple_of(1_000) {
        (milli / 1_000).to_string()
    } else {
        format!("0.{milli:03}")
    }
}
fn parse_profile(text: &str) -> Result<Profile, String> {
    match text {
        "thin" => Ok(Profile::Thin),
        "full" => Ok(Profile::Full),
        "proxy" => Ok(Profile::Proxy),
        _ => Err("--profile must be thin, full or proxy".into()),
    }
}
fn options(
    words: &[String],
) -> Result<(std::collections::BTreeMap<&str, &str>, Vec<String>), String> {
    let mut options = std::collections::BTreeMap::new();
    let services = words.chunks(2).try_fold(Vec::new(), |mut services, pair| {
        let [flag, value] = pair else {
            return Err("flag missing value".to_string());
        };
        if value.is_empty() || value.starts_with("--") {
            return Err(format!("{flag} missing value"));
        }
        if flag == "--service" {
            services.push(value.clone());
        } else if options.insert(flag.as_str(), value.as_str()).is_some() {
            return Err(format!("{flag} repeated"));
        }
        Ok(services)
    })?;
    Ok((options, services))
}
fn choice(value: Option<&str>, allowed: &[&str]) -> Result<String, String> {
    match value {
        None => Ok("unstated".into()),
        Some(value) if allowed.contains(&value) => Ok(value.into()),
        Some(value) => Err(format!("{value} must be one of {allowed:?}")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs_are_positive_and_bounded() {
        ["0", "101", "x"].iter().for_each(|runs| {
            assert!(
                parse(
                    &mut ["--collector", "c", "--profile", "thin", "--runs", runs]
                        .map(str::to_owned)
                        .into_iter()
                )
                .is_err()
            );
        });
    }
    #[test]
    fn proxy_keys_and_bounds() -> Result<(), String> {
        assert_eq!(
            parse_proxy("rps=20,latency_ms=50")?,
            ProxySpec {
                latency_ms: 50,
                rps: 20,
            }
        );
        [
            "rps=0,latency_ms=1",
            "rps=20",
            "rps=1,rps=2",
            "rps=1,latency_ms=60001",
            "x=1,rps=2",
        ]
        .iter()
        .for_each(|text| assert!(parse_proxy(text).is_err()));
        Ok(())
    }
    #[test]
    fn sweep_choices_services_and_argument_shape_are_checked() -> Result<(), String> {
        let parse_words = |extra: &[&str]| {
            parse(
                &mut ["--collector", "collector", "--profile", "thin"]
                    .into_iter()
                    .chain(extra.iter().copied())
                    .map(str::to_owned),
            )
        };
        ["1", "100"].iter().try_for_each(|bound| {
            let parsed = parse_words(&["--runs", bound])?;
            assert_eq!(parsed.runs.to_string(), *bound);
            Ok::<_, String>(())
        })?;
        let parsed = parse_words(&[
            "--server",
            "native",
            "--volume",
            "native-disk",
            "--build",
            "debug",
            "--service",
            "s3",
            "--service",
            "ec2",
        ])?;
        assert_eq!(parsed.server, "native");
        assert_eq!(parsed.volume, "native-disk");
        assert_eq!(parsed.build, "debug");
        assert_eq!(parsed.services, ["s3", "ec2"]);
        [
            ["--server", "wrong"],
            ["--volume", "wrong"],
            ["--build", "wrong"],
            ["--unknown", "x"],
        ]
        .iter()
        .for_each(|words| assert!(parse_words(words).is_err()));
        assert!(parse_words(&["--runs"]).is_err());
        assert!(
            parse(
                &mut ["--collector", "c", "--profile", "wrong"]
                    .map(str::to_owned)
                    .into_iter()
            )
            .is_err()
        );
        assert_eq!(
            parse(
                &mut ["--collector", "c", "--profile", "full"]
                    .map(str::to_owned)
                    .into_iter()
            )?
            .profile,
            Profile::Full
        );
        Ok(())
    }
    #[test]
    fn proxy_profile_defaults_to_fifty_ms_and_twenty_rps() -> Result<(), String> {
        let parsed = parse(
            &mut ["--collector", "c", "--profile", "proxy"]
                .map(str::to_owned)
                .into_iter(),
        )?;
        assert_eq!(parsed.profile.name(), "proxy");
        assert_eq!(
            parsed.proxy,
            Some(ProxySpec {
                latency_ms: 50,
                rps: 20,
            })
        );
        Ok(())
    }
    #[test]
    fn transport_options_parse_and_forward_to_collector() -> Result<(), String> {
        let parsed = parse(
            &mut [
                "--collector",
                "c",
                "--profile",
                "proxy",
                "--concurrency",
                "32",
                "--idle-timeout-ms",
                "90000",
                "--pool-idle-per-host",
                "32",
                "--max-rps",
                "0.5",
            ]
            .map(str::to_owned)
            .into_iter(),
        )?;
        assert_eq!(parsed.concurrency, Some(32));
        assert_eq!(parsed.idle_timeout_ms, Some(90_000));
        assert_eq!(parsed.pool_idle_per_host, Some(32));
        assert_eq!(parsed.max_rps_milli, Some(500));
        assert_eq!(
            collector_options(&parsed),
            [
                "--concurrency",
                "32",
                "--idle-timeout-ms",
                "90000",
                "--pool-idle-per-host",
                "32",
                "--max-rps",
                "0.500",
            ]
        );
        Ok(())
    }

    #[test]
    fn omitted_transport_options_leave_collector_defaults() -> Result<(), String> {
        let parsed = parse(
            &mut ["--collector", "c", "--profile", "thin"]
                .map(str::to_owned)
                .into_iter(),
        )?;
        assert_eq!(collector_options(&parsed), [""; 0]);
        Ok(())
    }

    #[test]
    fn pool_override_clears_inherited_transport_settings() -> Result<(), String> {
        let pool_only = parse(
            &mut [
                "--collector",
                "c",
                "--profile",
                "thin",
                "--pool-idle-per-host",
                "2",
            ]
            .map(str::to_owned)
            .into_iter(),
        )?;
        assert_eq!(collector_options(&pool_only), ["--pool-idle-per-host", "2"]);
        let mut command = tokio::process::Command::new("c");
        apply_collector_options(&mut command, &pool_only);
        let removed = command
            .as_std()
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .filter_map(|(name, _)| name.to_str())
            .collect::<Vec<_>>();
        assert_eq!(
            removed,
            [
                "INVENTORY_SWEEP_IDLE_TIMEOUT_MS",
                "INVENTORY_SWEEP_POOL_IDLE_PER_HOST"
            ]
        );
        Ok(())
    }

    #[test]
    fn environment_headers_report_set_and_unset_values() {
        assert_eq!(
            environment_headers(|name| {
                (name == "INVENTORY_POSTGRES_COPY_MIN_ROWS").then(|| "64".to_owned())
            }),
            [
                ("postgres_copy_min_rows".to_owned(), "64".to_owned()),
                ("postgres_copy_direct".to_owned(), "unset".to_owned()),
            ]
        );
    }

    #[test]
    fn invalid_transport_options_fail() {
        [
            ["--concurrency", "0", "--runs", "1"],
            ["--idle-timeout-ms", "90001", "--runs", "1"],
            ["--pool-idle-per-host", "0", "--runs", "1"],
            ["--concurrency", "4", "--pool-idle-per-host", "5"],
            ["--max-rps", "0.003", "--runs", "1"],
        ]
        .iter()
        .for_each(|options| {
            assert!(
                parse(
                    &mut ["--collector", "c", "--profile", "thin"]
                        .into_iter()
                        .chain(options.iter().copied())
                        .map(str::to_owned)
                )
                .is_err()
            )
        });
    }
}
