use super::{command::RunArgs, stack::Stack};
use crate::{
    HarnessError,
    inventory_seed::{
        SeedError,
        command::{SeedRunArgs, directory, execute_with},
    },
};
use std::collections::BTreeSet;

pub(super) async fn run(args: &RunArgs, stack: &Stack) -> Result<(), HarnessError> {
    let services =
        selected_recipes(args).map_err(|error| HarnessError::invalid(format!("seed: {error}")))?;
    let seed = SeedRunArgs {
        seeds: args.seeds.clone(),
        catalog: args.catalog.clone(),
        endpoint: stack.emulator().endpoint().into(),
        ca_bundle: stack.emulator().ca_bundle().into(),
        region: "us-east-1".into(),
        services,
        oracle: args.work.join("oracle.tsv"),
        matrix: args.work.join("seed-matrix.tsv"),
        concurrency: 4,
    };
    execute_with(&seed, stack.credentials())
        .await
        .map_err(|error| HarnessError::invalid(format!("seed: {error}")))
}

pub(super) fn selected_recipes(args: &RunArgs) -> Result<Vec<String>, SeedError> {
    Ok(args
        .services
        .iter()
        .map(|selector| service_name(selector))
        .map(|service| {
            if !args.seeds.join(format!("{service}.toml")).is_file() {
                return Err(SeedError::Invalid("requested seed service has no file"));
            }
            Ok(service)
        })
        .collect::<Result<BTreeSet<_>, _>>()?
        .into_iter()
        .map(str::to_owned)
        .collect())
}

pub(super) fn collector_services(args: &RunArgs) -> Result<Vec<String>, HarnessError> {
    if args.services.is_empty() {
        return Ok(Vec::new());
    }
    let requested = args
        .services
        .iter()
        .map(|selector| service_name(selector))
        .collect::<BTreeSet<_>>();
    let needed = recipe_services(args)
        .map_err(|error| HarnessError::invalid(format!("seed collector services: {error}")))?;
    Ok(args
        .services
        .iter()
        .cloned()
        .chain(
            needed
                .into_iter()
                .filter(|service| !requested.contains(service.as_str())),
        )
        .collect())
}

fn service_name(selector: &str) -> &str {
    selector
        .split_once(':')
        .map_or(selector, |(service, _)| service)
}

fn recipe_services(args: &RunArgs) -> Result<BTreeSet<String>, SeedError> {
    let selected = selected_recipes(args)?;
    if selected.is_empty() {
        return Err(SeedError::Invalid("selected services have no seed recipes"));
    }
    let files = directory::select(directory::files(&args.seeds)?, &selected)?;
    files
        .into_iter()
        .try_fold(BTreeSet::new(), |services, path| {
            Ok(directory::read_scenario(&path)?
                .steps()
                .iter()
                .filter(|step| !step.kind.is_empty())
                .fold(services, |mut services, step| {
                    services.insert(step.service.clone());
                    services
                }))
        })
}

#[cfg(test)]
mod tests;
