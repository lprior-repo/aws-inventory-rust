pub mod report;
pub mod tool;

use crate::{
    HarnessError,
    e2e::{self, stack::Stack, verify},
    inventory_seed::command::{SeedRunArgs, directory, execute_with},
    postgres::CleanupFailures,
};
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use report::{Outcome, Sample};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    ops::ControlFlow,
    path::{Path, PathBuf},
};
use tool::{Context, Definition};

pub const USAGE: &str = "inventory-harness head-to-head --work <empty-dir> --ours <binary> --tool <name>=<definition.toml> ... [--runs 3] [--catalog <file>] [--seeds <dir>] [--service <name>]...";

pub struct Args {
    pub work: PathBuf,
    pub ours: PathBuf,
    pub tools: Vec<(String, PathBuf)>,
    pub runs: usize,
    pub catalog: PathBuf,
    pub seeds: PathBuf,
    pub services: Vec<String>,
}

impl Args {
    pub fn parse(arguments: &[String]) -> Result<Self, HarnessError> {
        let pairs = Pairs::parse(arguments)?;
        let runs = pairs
            .value("--runs")?
            .map_or(Ok(3), str::parse::<usize>)
            .map_err(|_| HarnessError::invalid("--runs must be 1..100"))?;
        if !(1..=100).contains(&runs) {
            return Err(HarnessError::invalid("--runs must be 1..100"));
        }
        let tools = pairs.tools()?;
        let root = e2e::workspace_root();
        Ok(Self {
            work: pairs
                .value("--work")?
                .map(PathBuf::from)
                .ok_or_else(|| HarnessError::invalid(USAGE))?,
            ours: pairs
                .value("--ours")?
                .map(PathBuf::from)
                .ok_or_else(|| HarnessError::invalid(USAGE))?,
            tools,
            runs,
            catalog: pairs
                .value("--catalog")?
                .map_or_else(|| root.join("catalog/aws-api.catalog"), PathBuf::from),
            seeds: pairs
                .value("--seeds")?
                .map_or_else(|| root.join("crates/harness/seeds"), PathBuf::from),
            services: pairs.values("--service").map(str::to_owned).collect(),
        })
    }
}

struct Pairs<'a>(Vec<(&'a str, &'a str)>);

impl<'a> Pairs<'a> {
    fn parse(arguments: &'a [String]) -> Result<Self, HarnessError> {
        let (pairs, remainder) = arguments.as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(HarnessError::invalid(USAGE));
        }
        let allowed = [
            "--work",
            "--ours",
            "--tool",
            "--runs",
            "--catalog",
            "--seeds",
            "--service",
        ];
        if pairs
            .iter()
            .any(|[flag, value]| !allowed.contains(&flag.as_str()) || value.is_empty())
        {
            return Err(HarnessError::invalid(USAGE));
        }
        Ok(Self(
            pairs
                .iter()
                .map(|[flag, value]| (flag.as_str(), value.as_str()))
                .collect(),
        ))
    }

    fn values(&self, flag: &'a str) -> impl Iterator<Item = &'a str> {
        self.0
            .iter()
            .filter(move |(name, _)| *name == flag)
            .map(|(_, value)| *value)
    }

    fn value(&self, flag: &'a str) -> Result<Option<&'a str>, HarnessError> {
        let mut values = self.values(flag);
        let value = values.next();
        if values.next().is_some() {
            return Err(HarnessError::invalid(format!("duplicate {flag}")));
        }
        Ok(value)
    }

    fn tools(&self) -> Result<Vec<(String, PathBuf)>, HarnessError> {
        let tools = self
            .values("--tool")
            .map(|value| {
                value
                    .split_once('=')
                    .filter(|(name, path)| !name.is_empty() && !path.is_empty())
                    .map(|(name, path)| (name.to_owned(), PathBuf::from(path)))
                    .ok_or_else(|| HarnessError::invalid("--tool requires name=definition.toml"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if tools.is_empty() || tools.len() > 31 {
            return Err(HarnessError::invalid("supply 1..31 --tool definitions"));
        }
        if tools
            .iter()
            .map(|(name, _)| name)
            .collect::<BTreeSet<_>>()
            .len()
            != tools.len()
        {
            return Err(HarnessError::invalid("duplicate tool name"));
        }
        Ok(tools)
    }
}

pub async fn execute(args: Args, admin: &str) -> Result<Value, HarnessError> {
    execute_with_snapshot(args, admin, |args| {
        snapshot_dataset(args, |path| crate::files::read_bytes(path, 1_048_576))
    })
    .await
}

async fn execute_with_snapshot(
    mut args: Args,
    admin: &str,
    snapshot: impl FnOnce(&Args) -> Result<Value, HarnessError> + Send + 'static,
) -> Result<Value, HarnessError> {
    let mut signals = StopSignals::register()?;
    e2e::prepare_work(&args.work)?;
    args.work = std::fs::canonicalize(&args.work)?;
    args.ours = std::fs::canonicalize(&args.ours)?;
    args.catalog = std::fs::canonicalize(&args.catalog)?;
    args.seeds = std::fs::canonicalize(&args.seeds)?;
    if args.services.is_empty() {
        args.services = vec!["ec2".into(), "s3".into()];
    }
    let tools = std::iter::once(Ok(Definition::ours(args.ours.clone())))
        .chain(
            args.tools
                .iter()
                .map(|(name, path)| Definition::load(name, path)),
        )
        .collect::<Result<Vec<_>, _>>()?;
    let task = tokio::task::spawn_blocking(move || {
        let dataset = snapshot(&args)?;
        Ok::<_, HarnessError>((args, dataset))
    });
    let snapshot = async move {
        task.await.map_err(|error| {
            HarnessError::invalid(format!("dataset snapshot task failed: {error}"))
        })?
    };
    tokio::pin!(snapshot);
    let (args, dataset) = tokio::select! {
        biased;
        signal = signals.received() => {
            signal?;
            return match snapshot.await {
                Ok(_) => Err(HarnessError::invalid("head-to-head run interrupted")),
                Err(error) => Err(HarnessError::invalid(format!("{error}; head-to-head run interrupted during dataset snapshot"))),
            };
        }
        prepared = &mut snapshot => prepared?,
    };
    let stack = start_stack(args.work.clone(), &mut signals).await?;
    let failures = CleanupFailures::default();
    let signal = signals.received();
    tokio::pin!(signal);
    let result = until_stop(
        compare(&args, &tools, &stack, admin, dataset, &failures),
        signal.as_mut(),
    )
    .await;
    let result = finish_stop(result, stop_stack(stack), signal).await;
    write_outcome(&args.work, result, &failures)
}

struct StopSignals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
    hangup: tokio::signal::unix::Signal,
    quit: tokio::signal::unix::Signal,
}

impl StopSignals {
    fn register() -> Result<Self, std::io::Error> {
        use tokio::signal::unix::{SignalKind, signal};
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
            hangup: signal(SignalKind::hangup())?,
            quit: signal(SignalKind::quit())?,
        })
    }

    async fn received(&mut self) -> Result<(), std::io::Error> {
        let received = tokio::select! {
            signal = self.interrupt.recv() => signal,
            signal = self.terminate.recv() => signal,
            signal = self.hangup.recv() => signal,
            signal = self.quit.recv() => signal,
        };
        received.ok_or_else(|| std::io::Error::other("head-to-head signal stream closed"))
    }
}

async fn start_stack(work: PathBuf, signals: &mut StopSignals) -> Result<Stack, HarnessError> {
    let task = tokio::task::spawn_blocking(move || Stack::start(&work));
    let startup = async move {
        task.await
            .map_err(|error| HarnessError::invalid(format!("stack startup task failed: {error}")))?
    };
    tokio::pin!(startup);
    tokio::select! {
        biased;
        signal = signals.received() => {
            let interrupted = match signal {
                Ok(()) => Err(HarnessError::invalid("head-to-head run interrupted")),
                Err(error) => Err(error.into()),
            };
            match startup.await {
                Ok(stack) => finish_result(interrupted, stop_stack(stack).await),
                Err(error) => Err(HarnessError::invalid(format!(
                    "{error}; head-to-head run interrupted during startup",
                ))),
            }
        }
        stack = &mut startup => stack,
    }
}

async fn stop_stack(stack: Stack) -> Result<(), HarnessError> {
    tokio::task::spawn_blocking(move || stack.finish())
        .await
        .map_err(|error| HarnessError::invalid(format!("stack shutdown task failed: {error}")))?
}

async fn until_stop<T>(
    run: impl Future<Output = T>,
    signal: impl Future<Output = Result<(), std::io::Error>>,
) -> ControlFlow<HarnessError, T> {
    tokio::select! {
        biased;
        signal = signal => ControlFlow::Break(stop_error(signal)),
        result = run => ControlFlow::Continue(result),
    }
}

fn stop_error(signal: Result<(), std::io::Error>) -> HarnessError {
    match signal {
        Ok(()) => HarnessError::invalid("head-to-head run interrupted"),
        Err(error) => error.into(),
    }
}

async fn finish_stop<T>(
    comparison: ControlFlow<HarnessError, Result<T, HarnessError>>,
    cleanup: impl Future<Output = Result<(), HarnessError>>,
    signal: impl Future<Output = Result<(), std::io::Error>>,
) -> Result<T, HarnessError> {
    let result = match comparison {
        ControlFlow::Break(error) => return finish_result(Err(error), cleanup.await),
        ControlFlow::Continue(result) => result,
    };
    tokio::pin!(cleanup);
    tokio::pin!(signal);
    let (cleanup, interrupted) = match until_stop(cleanup.as_mut(), signal.as_mut()).await {
        ControlFlow::Continue(cleanup) => (cleanup, signal.now_or_never().map(stop_error)),
        ControlFlow::Break(interrupted) => (cleanup.await, Some(interrupted)),
    };
    let result = match (result, interrupted) {
        (result, None) => result,
        (Ok(_), Some(interrupted)) => Err(interrupted),
        (Err(error), Some(interrupted)) => {
            Err(HarnessError::invalid(format!("{error}; {interrupted}")))
        }
    };
    finish_result(result, cleanup)
}

fn finish_result<T>(
    result: Result<T, HarnessError>,
    cleanup: Result<(), HarnessError>,
) -> Result<T, HarnessError> {
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(HarnessError::invalid(format!(
            "{error}; cleanup failed: {cleanup}",
        ))),
    }
}

fn schedule<T>(tools: &[T], runs: usize) -> impl Iterator<Item = (usize, &T)> {
    (0..runs).flat_map(move |run| report::rotation(tools, run).map(move |tool| (run, tool)))
}

async fn compare(
    args: &Args,
    tools: &[Definition],
    stack: &Stack,
    admin: &str,
    dataset: Value,
    failures: &CleanupFailures,
) -> Result<Value, HarnessError> {
    seed(args, stack).await?;
    let seeded = dataset_counts(&args.work.join("oracle.tsv"))?;
    let versions = versions(args, tools, stack, admin, failures).await?;
    let samples = futures_util::stream::iter(schedule(tools, args.runs))
        .then(|(run, tool)| collect(args, tool, stack, admin, run, &versions, failures))
        .try_collect::<Vec<_>>()
        .await?;
    let names: Vec<_> = tools.iter().map(|tool| tool.name.clone()).collect();
    let dataset = json!({ "files": dataset, "seeded_counts": seeded,
        "oracle": "oracle.tsv", "seed_outcomes": "seed-matrix.tsv", "region": "us-east-1" });
    let scopes = tools
        .iter()
        .map(|tool| (tool.name.clone(), tool.scope(&args.services).to_vec()))
        .collect();
    let evidence = report::Evidence {
        versions,
        dataset,
        seeded_counts: seeded,
        machine: machine()?,
        services: args.services.clone(),
        scopes,
    };
    let report = report::build(&names, &samples, &evidence);
    Ok(report)
}

async fn collect(
    args: &Args,
    tool: &Definition,
    stack: &Stack,
    admin: &str,
    run: usize,
    versions: &Value,
    failures: &CleanupFailures,
) -> Result<Sample, HarnessError> {
    let directory = args.work.join(format!("run-{run}-{}", tool.name));
    std::fs::create_dir(&directory)?;
    let context = Context {
        stack,
        catalog: &args.catalog,
        services: &args.services,
        directory: &directory,
        admin,
        failures,
    };
    let outcome = gated_sample(versions, &tool.name, tool::execute(tool, &context)).await;
    Ok(Sample {
        tool: tool.name.clone(),
        run,
        outcome,
    })
}

async fn gated_sample(versions: &Value, name: &str, run: impl Future<Output = Outcome>) -> Outcome {
    match version_failure(versions, name) {
        Some(failure) => failure,
        None => run.await,
    }
}

fn version_failure(versions: &Value, name: &str) -> Option<Outcome> {
    versions
        .get(name)
        .and_then(|entry| entry.get("error"))
        .and_then(Value::as_str)
        .map(|reason| Outcome::Failed {
            reason: format!("version command failed: {reason}; comparison waits for J3"),
        })
}

async fn versions(
    args: &Args,
    tools: &[Definition],
    stack: &Stack,
    admin: &str,
    failures: &CleanupFailures,
) -> Result<Value, HarnessError> {
    futures_util::stream::iter(tools).then(|tool| async move {
        let directory = args.work.join(format!("version-{}", tool.name));
        std::fs::create_dir(&directory)?;
        let context = Context { stack, catalog: &args.catalog, services: &args.services, directory: &directory, admin, failures };
        let version = match tool::execute_command(tool, &tool.version, &context, "version").await {
            Ok(text) => json!({ "output": text.trim(), "definition": tool.source, "compatibility": tool.compatibility }),
            Err(error) => json!({ "error": error.to_string(), "definition": tool.source, "compatibility": tool.compatibility }),
        };
        Ok::<_, HarnessError>((tool.name.clone(), version))
    }).try_collect::<serde_json::Map<String, Value>>().await.map(Value::Object)
}

async fn seed(args: &Args, stack: &Stack) -> Result<(), HarnessError> {
    let seed = SeedRunArgs {
        seeds: args.work.join("dataset"),
        catalog: args.catalog.clone(),
        endpoint: stack.emulator().endpoint().into(),
        ca_bundle: stack.emulator().ca_bundle().into(),
        region: "us-east-1".into(),
        services: args.services.clone(),
        oracle: args.work.join("oracle.tsv"),
        matrix: args.work.join("seed-matrix.tsv"),
        concurrency: 4,
    };
    execute_with(&seed, stack.credentials())
        .await
        .map_err(|error| HarnessError::invalid(format!("seed: {error}")))
}

fn snapshot_dataset(
    args: &Args,
    read: impl Fn(&Path) -> Result<Vec<u8>, HarnessError>,
) -> Result<Value, HarnessError> {
    use sha2::{Digest, Sha256};
    let files = directory::select(
        directory::files(&args.seeds).map_err(seed_error)?,
        &args.services,
    )
    .map_err(seed_error)?;
    let destination = args.work.join("dataset");
    std::fs::create_dir(&destination)?;
    let manifest = files
        .iter()
        .map(|path| {
            let name = path
                .file_name()
                .ok_or_else(|| HarnessError::invalid("seed path lacks filename"))?;
            let bytes = read(path)?;
            std::fs::write(destination.join(name), &bytes)?;
            Ok(
                json!({ "source": path, "snapshot": Path::new("dataset").join(name),
            "sha256": hex::encode(Sha256::digest(&bytes)) }),
            )
        })
        .collect::<Result<Vec<_>, HarnessError>>()?;
    Ok(json!(manifest))
}

fn seed_error(error: crate::inventory_seed::SeedError) -> HarnessError {
    HarnessError::invalid(format!("seed: {error}"))
}

fn dataset_counts(path: &Path) -> Result<report::Counts, HarnessError> {
    let counts =
        verify::read_text(path)?
            .lines()
            .try_fold(report::Counts::new(), |mut counts, line| {
                let mut fields = line.split('\t');
                match std::array::from_fn(|_| fields.next()) {
                    [
                        Some("resource"),
                        Some(kind),
                        Some(account),
                        Some(region),
                        Some(id),
                        None | Some("service-created"),
                        None,
                    ] if [kind, account, region, id]
                        .iter()
                        .all(|value| !value.is_empty()) =>
                    {
                        let count = counts.entry(kind.into()).or_insert(0);
                        *count = count
                            .checked_add(1)
                            .ok_or_else(|| HarnessError::invalid("seed count overflow"))?;
                    }
                    [
                        Some("edge"),
                        Some(source_kind),
                        Some(source_id),
                        Some(relation),
                        Some(target_kind),
                        Some(target_id),
                        None,
                    ] if [source_kind, source_id, relation, target_kind, target_id]
                        .iter()
                        .all(|value| !value.is_empty()) => {}
                    _ => return Err(HarnessError::invalid("malformed seed oracle row")),
                }
                Ok(counts)
            })?;
    if counts.is_empty() {
        return Err(HarnessError::invalid(
            "head-to-head seed produced no resources",
        ));
    }
    Ok(counts)
}

fn machine() -> Result<Value, HarnessError> {
    Ok(
        json!({ "os": std::env::consts::OS, "architecture": std::env::consts::ARCH,
        "logical_cpus": std::thread::available_parallelism()?.get(),
        "cpu": verify::read_text(Path::new("/proc/cpuinfo"))?,
        "memory": verify::read_text(Path::new("/proc/meminfo"))?,
        "kernel": verify::read_text(Path::new("/proc/sys/kernel/osrelease"))? }),
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod snapshot_tests;

pub(crate) fn write_outcome(
    work: &Path,
    result: Result<Value, HarnessError>,
    failures: &CleanupFailures,
) -> Result<Value, HarnessError> {
    let failures = failures.failures();
    let cleanup = match failures.first() {
        Some(first) => Err(HarnessError::invalid(format!(
            "database {} cleanup failed: {}",
            first.database, first.reason,
        ))),
        None => Ok(()),
    };
    let (mut value, result) = match result {
        Ok(value) => (value, Ok(())),
        Err(error) => (json!({}), Err(error)),
    };
    let result = finish_result(result, cleanup);
    let object = value
        .as_object_mut()
        .ok_or_else(|| HarnessError::invalid("comparison report must be an object"))?;
    let failures: Vec<_> = failures
        .iter()
        .map(|failure| json!({"database": failure.database, "reason": failure.reason}))
        .collect();
    object.insert("cleanup_failures".into(), json!(failures));
    if let Err(error) = &result {
        object.insert("comparison_ready".into(), json!(false));
        object.insert("error".into(), json!(error.to_string()));
        if let Some(tools) = object.get_mut("tools").and_then(Value::as_array_mut) {
            tools
                .iter_mut()
                .filter_map(Value::as_object_mut)
                .for_each(|tool| {
                    tool.insert("comparison_timing".into(), Value::Null);
                });
        }
    }
    report::write(&work.join("report.json"), &value)?;
    result.map(|()| value)
}
