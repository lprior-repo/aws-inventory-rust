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
        proxy,
        out,
        catalog,
        work,
        server,
        volume,
        build,
        services,
    })
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
                rps: 20
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
                rps: 20
            })
        );
        Ok(())
    }
}
