use super::SeedError;
use std::{collections::BTreeMap, path::PathBuf};

pub struct SeedRunArgs {
    pub seeds: PathBuf,
    pub catalog: PathBuf,
    pub endpoint: String,
    pub ca_bundle: PathBuf,
    pub region: String,
    pub services: Vec<String>,
    pub oracle: PathBuf,
    pub matrix: PathBuf,
    pub concurrency: usize,
}

impl SeedRunArgs {
    pub fn parse(arguments: &[String]) -> Result<Self, SeedError> {
        validate_arguments(arguments)?;
        let (options, services) = options(arguments)?;
        let args = Self {
            seeds: required(&options, "--seeds")?.into(),
            catalog: required(&options, "--catalog")?.into(),
            endpoint: required(&options, "--endpoint")?,
            ca_bundle: required(&options, "--ca-bundle")?.into(),
            region: required(&options, "--region")?,
            oracle: required(&options, "--oracle")?.into(),
            matrix: required(&options, "--matrix")?.into(),
            concurrency: concurrency(&options)?,
            services,
        };
        validate_paths(&args)?;
        Ok(args)
    }
}

type Options = (BTreeMap<String, String>, Vec<String>);
fn options(arguments: &[String]) -> Result<Options, SeedError> {
    arguments
        .as_chunks::<2>()
        .0
        .iter()
        .try_fold((BTreeMap::new(), Vec::new()), option)
}

fn option((mut options, mut services): Options, pair: &[String; 2]) -> Result<Options, SeedError> {
    let [flag, value] = pair;
    if value.is_empty() || value.len() > 4096 {
        return Err(SeedError::Invalid(
            "seed option value is empty or oversized",
        ));
    }
    if flag == "--service" {
        services.push(value.clone());
    } else if !known(flag) || options.insert(flag.clone(), value.clone()).is_some() {
        return Err(SeedError::Message(format!(
            "unknown or repeated seed flag: {flag}"
        )));
    }
    Ok((options, services))
}

fn known(flag: &str) -> bool {
    matches!(
        flag,
        "--seeds"
            | "--catalog"
            | "--endpoint"
            | "--ca-bundle"
            | "--region"
            | "--oracle"
            | "--matrix"
            | "--concurrency"
    )
}

fn required(options: &BTreeMap<String, String>, name: &str) -> Result<String, SeedError> {
    options
        .get(name)
        .cloned()
        .ok_or_else(|| SeedError::Message(format!("seed run requires {name}")))
}

fn validate_arguments(arguments: &[String]) -> Result<(), SeedError> {
    if arguments.len() > 128 || !arguments.len().is_multiple_of(2) {
        return Err(SeedError::Invalid(
            "seed run requires at most 64 flag/value pairs",
        ));
    }
    Ok(())
}

fn validate_paths(args: &SeedRunArgs) -> Result<(), SeedError> {
    if args.oracle == args.matrix || args.oracle == args.catalog || args.matrix == args.catalog {
        return Err(SeedError::Invalid(
            "seed report paths must differ from each other and the catalog",
        ));
    }
    Ok(())
}

fn concurrency(options: &BTreeMap<String, String>) -> Result<usize, SeedError> {
    let value = options
        .get("--concurrency")
        .map_or(Ok(1), |value| value.parse::<usize>())
        .map_err(|_| SeedError::Invalid("seed concurrency must be an integer"))?;
    if !(1..=64).contains(&value) {
        return Err(SeedError::Invalid(
            "seed concurrency must be between 1 and 64",
        ));
    }
    Ok(value)
}
