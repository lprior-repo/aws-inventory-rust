use super::{Credentials, RuntimeCatalog, SeedError, SeedRunArgs};
use crate::inventory_seed::{SeedConfig, file, run_report_with_concurrency};
use futures_util::TryStreamExt;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

type Rows = (Vec<String>, Vec<String>);

pub(super) async fn run(
    args: &SeedRunArgs,
    catalog: &RuntimeCatalog<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
) -> Result<Rows, SeedError> {
    let selected = select(files(&args.seeds)?, &args.services)?;
    futures_util::stream::iter(selected.into_iter().map(Ok::<_, SeedError>))
        .try_fold((Vec::new(), Vec::new()), |rows, path| async move {
            let added = seed_file(&path, args, catalog, config, credentials).await?;
            append(rows, added)
        })
        .await
}

async fn seed_file(
    path: &Path,
    args: &SeedRunArgs,
    catalog: &RuntimeCatalog<'_>,
    config: &SeedConfig,
    credentials: &Credentials,
) -> Result<Rows, SeedError> {
    let bytes = read_bounded(path, 1_048_576)?;
    let content =
        std::str::from_utf8(&bytes).map_err(|_| SeedError::Invalid("seed file must be UTF-8"))?;
    let (scenario, labels) = file::load_labeled(content)?;
    let report =
        run_report_with_concurrency(catalog, config, credentials, &scenario, args.concurrency)
            .await?;
    Ok((
        report.oracle_rows(),
        report.matrix_rows(scenario.steps(), &labels)?,
    ))
}

fn append((mut oracle, mut matrix): Rows, (resources, outcomes): Rows) -> Result<Rows, SeedError> {
    oracle.extend(resources);
    matrix.extend(outcomes);
    if oracle.len() > 131_072 || matrix.len() > 65_536 {
        return Err(SeedError::Invalid(
            "seed directory report exceeds row limit",
        ));
    }
    Ok((oracle, matrix))
}

fn files(directory: &Path) -> Result<Vec<PathBuf>, SeedError> {
    let entries = std::fs::read_dir(directory)?
        .take(65)
        .collect::<Result<Vec<_>, _>>()?;
    if entries.len() > 64 {
        return Err(SeedError::Invalid("seed directory exceeds 64 entries"));
    }
    let mut paths: Vec<_> = entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect();
    paths.sort();
    if paths.is_empty() {
        return Err(SeedError::Invalid("seed directory has no TOML files"));
    }
    Ok(paths)
}

fn select(files: Vec<PathBuf>, services: &[String]) -> Result<Vec<PathBuf>, SeedError> {
    let selected: Vec<_> = files
        .into_iter()
        .filter(|path| services.is_empty() || services.iter().any(|service| matches(path, service)))
        .collect();
    if services
        .iter()
        .any(|service| !selected.iter().any(|path| matches(path, service)))
    {
        return Err(SeedError::Invalid("requested seed service has no file"));
    }
    Ok(selected)
}

fn matches(path: &Path, service: &str) -> bool {
    path.file_stem().is_some_and(|stem| stem == service)
}

pub(super) fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, SeedError> {
    let limit = maximum
        .checked_add(1)
        .ok_or(SeedError::Invalid("seed read limit overflow"))?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit)
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).map_err(|_| SeedError::Invalid("seed file size overflow"))?
        > maximum
    {
        return Err(SeedError::Invalid("seed file exceeds byte limit"));
    }
    Ok(bytes)
}

pub(super) fn write_rows(path: &Path, mut rows: Vec<String>) -> Result<(), SeedError> {
    rows.sort();
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    rows.iter().try_for_each(|row| writeln!(file, "{row}"))?;
    file.flush()?;
    Ok(())
}
