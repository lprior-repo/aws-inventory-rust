use super::{command::RunArgs, stack::Stack};
use crate::{
    HarnessError,
    inventory_seed::command::{SeedRunArgs, execute_with},
};
use std::collections::BTreeSet;

pub(super) async fn run(args: &RunArgs, stack: &Stack) -> Result<(), HarnessError> {
    let services = selected_recipes(args);
    if !args.services.is_empty() && services.is_empty() {
        return Err(HarnessError::invalid(
            "selected services have no seed recipes",
        ));
    }
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

pub(super) fn selected_recipes(args: &RunArgs) -> Vec<String> {
    args.services
        .iter()
        .map(|selector| {
            selector
                .split_once(':')
                .map_or(selector.as_str(), |(service, _)| service)
        })
        .filter(|service| args.seeds.join(format!("{service}.toml")).is_file())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}
