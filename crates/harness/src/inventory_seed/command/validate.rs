use super::{RuntimeCatalog, SeedError, directory};
use crate::inventory_seed::model::SeedModels;
use std::{collections::BTreeMap, path::Path, process::ExitCode};

type Options = (BTreeMap<String, String>, Vec<String>);

pub(super) fn run(arguments: &[String]) -> ExitCode {
    match execute(arguments) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(arguments: &[String]) -> Result<bool, SeedError> {
    let (options, services) = options(arguments)?;
    let seeds = required(&options, "--seeds")?;
    let bytes = directory::read_bounded(Path::new(required(&options, "--catalog")?), 67_108_864)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let paths = directory::select(directory::files(Path::new(seeds))?, &services)?;
    Ok(paths.iter().fold(true, |valid, path| {
        let result = validate(path, &catalog);
        match &result {
            Ok(()) => println!("{}\tvalid", path.display()),
            Err(error) => println!("{}\tinvalid\t{error}", path.display()),
        }
        result.is_ok() && valid
    }))
}

fn validate(path: &Path, catalog: &RuntimeCatalog<'_>) -> Result<(), SeedError> {
    let scenario = directory::read_scenario(path)?;
    SeedModels::load(catalog, &scenario)?;
    Ok(())
}

fn options(arguments: &[String]) -> Result<Options, SeedError> {
    if arguments.is_empty() || arguments.len() > 128 || !arguments.len().is_multiple_of(2) {
        return Err(SeedError::Invalid("seed validate requires at most 64 flag/value pairs"));
    }
    arguments.as_chunks::<2>().0.iter().try_fold(
        (BTreeMap::new(), Vec::new()),
        |(mut options, mut services), [flag, value]| {
            if value.is_empty() || value.len() > 4096 {
                return Err(SeedError::Invalid("seed validate option value is invalid"));
            }
            if flag == "--service" {
                services.push(value.clone());
            } else if !matches!(flag.as_str(), "--seeds" | "--catalog")
                || options.insert(flag.clone(), value.clone()).is_some()
            {
                return Err(SeedError::Invalid("seed validate option is unknown or duplicated"));
            }
            Ok((options, services))
        },
    )
}

fn required<'a>(options: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str, SeedError> {
    options.get(name).map(String::as_str)
        .ok_or_else(|| SeedError::Message(format!("seed validate requires {name}")))
}
