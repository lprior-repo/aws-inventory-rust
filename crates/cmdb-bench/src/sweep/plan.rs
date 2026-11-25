use super::args::{Profile, SweepArgs};
use inventory_harness::{
    e2e::stack::Stack,
    inventory_seed::{
        self,
        command::{SeedRunArgs, execute_with},
    },
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
#[derive(Debug)]
pub(crate) struct Plan {
    pub seeds: PathBuf,
    pub recipes: Vec<String>,
    pub services: Vec<String>,
}
pub(crate) fn plan(workspace: &Path, args: &SweepArgs) -> Result<Plan, String> {
    let seeds = workspace.join(match args.profile {
        Profile::Thin | Profile::Proxy => "crates/harness/seeds/bench",
        Profile::Full => "crates/harness/seeds",
    });
    let selected = selected_paths(&seeds, &args.services)?;
    let recipes = selected
        .iter()
        .map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
                .ok_or("non-UTF8 seed filename".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if recipes.is_empty()
        || args
            .services
            .iter()
            .any(|service| !recipes.contains(service))
    {
        return Err("requested seed service has no recipe".into());
    }
    let services = if args.profile == Profile::Full && args.services.is_empty() {
        Vec::new()
    } else {
        selected
            .iter()
            .try_fold(
                recipes.iter().cloned().collect::<BTreeSet<_>>(),
                |mut services, path| {
                    let content = super::bounded::read_sync(path, super::bounded::RECIPE_LIMIT)?;
                    let scenario = inventory_seed::load(&content).map_err(text)?;
                    services.extend(
                        scenario
                            .steps()
                            .iter()
                            .filter(|step| !step.kind.is_empty())
                            .map(|step| step.service.clone()),
                    );
                    Ok::<_, String>(services)
                },
            )?
            .into_iter()
            .collect()
    };
    Ok(Plan {
        seeds,
        recipes,
        services,
    })
}
fn selected_paths(seeds: &Path, services: &[String]) -> Result<Vec<PathBuf>, String> {
    let paths = std::fs::read_dir(seeds)
        .map_err(text)?
        .take(513)
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(text)?;
    if paths.len() > 512 {
        return Err("seed directory exceeds 512 entries".into());
    }
    let mut paths = paths
        .into_iter()
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "toml")
                && (services.is_empty()
                    || services.iter().any(|service| {
                        path.file_stem()
                            .is_some_and(|stem| stem == service.as_str())
                    }))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}
pub(crate) async fn seed(
    plan: &Plan,
    args: &SweepArgs,
    stack: &Stack,
    work: &Path,
) -> Result<(usize, usize), String> {
    let oracle = work.join("oracle.tsv");
    let matrix = work.join("seed-matrix.tsv");
    let seed = SeedRunArgs {
        seeds: plan.seeds.clone(),
        catalog: args.catalog.clone(),
        endpoint: stack.emulator().endpoint().into(),
        ca_bundle: stack.emulator().ca_bundle().into(),
        region: "us-east-1".into(),
        services: plan.recipes.clone(),
        oracle: oracle.clone(),
        matrix: matrix.clone(),
        concurrency: 4,
    };
    execute_with(&seed, stack.credentials())
        .await
        .map_err(text)?;
    let oracle = super::bounded::read(&oracle, super::bounded::REPORT_LIMIT).await?;
    let matrix = super::bounded::read(&matrix, super::bounded::REPORT_LIMIT).await?;
    Ok((oracle.lines().count(), matrix.lines().count()))
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
#[cfg(test)]
mod tests {
    #[test]
    fn oversized_seed_recipe_is_rejected_before_parsing() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let recipes = directory.path().join("crates/harness/seeds/bench");
        std::fs::create_dir_all(&recipes)?;
        std::fs::File::create(recipes.join("s3.toml"))?.set_len(1_048_577)?;
        let args = super::super::args::parse(
            &mut ["--collector", "c", "--profile", "thin", "--service", "s3"]
                .map(str::to_owned)
                .into_iter(),
        )?;
        let error = super::plan(directory.path(), &args)
            .err()
            .ok_or("oversized recipe accepted")?;
        assert!(error.contains("1048576-byte"), "{error}");
        Ok(())
    }
}
