use super::report::{Counts, Outcome};
use crate::{
    HarnessError,
    e2e::{collector, stack::Stack, verify},
    emulator::ChildOwner,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::process::Command;

const MAX_KINDS: usize = 10000;

#[cfg(test)]
mod native_tests;

#[cfg(test)]
mod cancel_tests;

pub enum Tool {
    Collector {
        binary: PathBuf,
    },
    External {
        run: Vec<String>,
        reset: Vec<String>,
        counts: Vec<String>,
        api_calls: Option<Vec<String>>,
    },
}

pub struct Definition {
    pub name: String,
    pub tool: Tool,
    pub version: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub source: Option<PathBuf>,
    pub compatibility: String,
    pub services: Option<Vec<String>>,
}

impl Definition {
    pub fn ours(binary: PathBuf) -> Self {
        Self {
            name: "ours".into(),
            version: vec![binary.to_string_lossy().into_owned(), "--version".into()],
            tool: Tool::Collector { binary },
            environment: BTreeMap::new(),
            source: None,
            compatibility: "native emulator endpoint and private CA adapter".into(),
            services: None,
        }
    }

    pub fn load(name: &str, path: &Path) -> Result<Self, HarnessError> {
        if name == "ours"
            || name.is_empty()
            || name.len() > 64
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(HarnessError::invalid(
                "tool name must be 1..64 ASCII letters, digits, '-' or '_'; ours is reserved",
            ));
        }
        Self::load_file(name, path).map_err(|error| {
            HarnessError::invalid(format!(
                "tool {name} definition {}: {error}",
                path.display()
            ))
        })
    }

    fn load_file(name: &str, path: &Path) -> Result<Self, HarnessError> {
        let text = verify::read_text(path)?;
        let table = text
            .parse::<toml::Table>()
            .map_err(|error| HarnessError::invalid(error.to_string()))?;
        let tool = match table.get("collector").and_then(toml::Value::as_str) {
            Some(binary) => Tool::Collector {
                binary: collector_binary(binary, path)?,
            },
            None => Tool::External {
                run: command(&table, "run")?,
                reset: command(&table, "reset")?,
                counts: command(&table, "counts")?,
                api_calls: table
                    .get("api_calls")
                    .map(|_| command(&table, "api_calls"))
                    .transpose()?,
            },
        };
        let environment = environment(&table, &tool)?;
        let services = match &tool {
            Tool::External { .. } => Some(command(&table, "services")?),
            Tool::Collector { .. } => {
                if table.contains_key("services") {
                    return Err(HarnessError::invalid(
                        "collector definitions cannot declare services",
                    ));
                }
                None
            }
        };
        Ok(Self { name: name.into(), tool, version: command(&table, "version")?, environment, services,
            source: Some(path.into()), compatibility: table.get("compatibility").and_then(toml::Value::as_str)
                .map_or_else(|| "endpoint/private CA and free plugin availability unverified; failure defers comparison to J3".into(), str::to_owned) })
    }

    pub fn scope<'a>(&'a self, requested: &'a [String]) -> &'a [String] {
        match &self.services {
            Some(services) => services,
            None => requested,
        }
    }
}

fn collector_binary(binary: &str, definition: &Path) -> Result<PathBuf, HarnessError> {
    if binary.is_empty() {
        return Err(HarnessError::invalid("collector must not be empty"));
    }
    let path = definition
        .parent()
        .ok_or_else(|| HarnessError::invalid("tool definition has no directory"))?
        .join(binary);
    let binary = std::fs::canonicalize(&path)
        .map_err(|error| HarnessError::invalid(format!("collector {}: {error}", path.display())))?;
    if !binary.is_file() {
        return Err(HarnessError::invalid(format!(
            "collector {} is not a regular file",
            binary.display()
        )));
    }
    Ok(binary)
}

fn environment(table: &toml::Table, tool: &Tool) -> Result<BTreeMap<String, String>, HarnessError> {
    if matches!(tool, Tool::Collector { .. }) && table.contains_key("environment") {
        return Err(HarnessError::invalid(
            "collector definitions cannot declare environment",
        ));
    }
    match table.get("environment") {
        None => Ok(BTreeMap::new()),
        Some(toml::Value::Table(values)) => values
            .iter()
            .map(|(key, value)| {
                value
                    .as_str()
                    .map(|value| (key.clone(), value.to_owned()))
                    .ok_or_else(|| HarnessError::invalid("tool environment values must be strings"))
            })
            .collect(),
        Some(_) => Err(HarnessError::invalid("tool environment must be a table")),
    }
}

fn command(table: &toml::Table, key: &str) -> Result<Vec<String>, HarnessError> {
    let values = table
        .get(key)
        .and_then(toml::Value::as_array)
        .ok_or_else(|| HarnessError::invalid(format!("tool {key} must be a command array")))?;
    if values.is_empty() || values.len() > 256 {
        return Err(HarnessError::invalid(
            "tool command must contain 1..256 arguments",
        ));
    }
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    HarnessError::invalid("tool command arguments must be nonempty strings")
                })
        })
        .collect()
}

pub struct Context<'a> {
    pub stack: &'a Stack,
    pub catalog: &'a Path,
    pub services: &'a [String],
    pub directory: &'a Path,
    pub admin: &'a str,
    pub failures: &'a crate::postgres::CleanupFailures,
}

struct Run {
    elapsed: Duration,
    counts: Counts,
    api_calls: Option<u64>,
    left_processes: bool,
}

impl From<Run> for Outcome {
    fn from(run: Run) -> Self {
        Self::Success {
            seconds: run.elapsed.as_secs_f64(),
            counts: run.counts,
            api_calls: run.api_calls,
            left_processes: run.left_processes,
        }
    }
}

struct ProcessOutput {
    stdout: String,
    left_processes: bool,
}

pub async fn execute(definition: &Definition, context: &Context<'_>) -> Outcome {
    match run(definition, context).await {
        Ok(run) => run.into(),
        Err(error) => Outcome::Failed {
            reason: error.to_string(),
        },
    }
}

async fn run(definition: &Definition, context: &Context<'_>) -> Result<Run, HarnessError> {
    match &definition.tool {
        Tool::Collector { binary } => run_collector(binary, context).await,
        Tool::External { .. } => {
            run_external(definition, &variables(context), context.directory).await
        }
    }
}

async fn run_external(
    definition: &Definition,
    variables: &BTreeMap<String, String>,
    directory: &Path,
) -> Result<Run, HarnessError> {
    let Tool::External {
        run,
        reset,
        counts,
        api_calls,
    } = &definition.tool
    else {
        return Err(HarnessError::invalid(
            "external stages require an external definition",
        ));
    };
    let reset = execute_stage(definition, reset, variables, directory, "reset").await?;
    let started = Instant::now();
    let collected = execute_stage(definition, run, variables, directory, "collect").await?;
    let elapsed = started.elapsed();
    let counted = execute_stage(definition, counts, variables, directory, "counts").await?;
    let counts = parse_counts(&counted.stdout)?;
    let mut left_processes =
        reset.left_processes || collected.left_processes || counted.left_processes;
    let calls = match api_calls {
        Some(command) => {
            let output =
                execute_stage(definition, command, variables, directory, "api-calls").await?;
            left_processes |= output.left_processes;
            Some(output.stdout.trim().parse().map_err(|_| {
                HarnessError::invalid("API calls command must return one unsigned integer")
            })?)
        }
        None => None,
    };
    Ok(Run {
        elapsed,
        counts,
        api_calls: calls,
        left_processes,
    })
}

async fn run_collector(binary: &Path, context: &Context<'_>) -> Result<Run, HarnessError> {
    let database =
        crate::postgres::TestDatabase::create_tracked(context.admin, context.failures).await?;
    let inputs = collector::SweepInputs {
        catalog: context.catalog,
        database: database.url(),
        services: context.services,
    };
    let command = collector::command(
        context.stack,
        binary,
        &inputs,
        context.stack.emulator().endpoint(),
    );
    let cleared = context.stack.graph().clear().await;
    let started = Instant::now();
    let result = match cleared {
        Ok(()) => execute_process(command, context.directory, "collect")
            .await
            .map_err(|error| collector_failure(error, context.directory)),
        Err(error) => Err(error),
    };
    let elapsed = started.elapsed();
    let counted = match result {
        Ok(summary) => read_counts(database.url(), Duration::from_secs(60))
            .await
            .map(|counts| Run {
                elapsed,
                counts,
                api_calls: reported_calls(&summary.stdout),
                left_processes: summary.left_processes,
            }),
        Err(error) => Err(error),
    };
    let cleanup = database.finish().await;
    super::finish_result(counted, cleanup)
}

fn collector_failure(error: HarnessError, directory: &Path) -> HarnessError {
    match verify::read_text(&directory.join("collect.stderr")) {
        Ok(detail) if !detail.trim().is_empty() => {
            HarnessError::invalid(format!("{error}; collector stderr: {}", detail.trim(),))
        }
        Ok(_) => error,
        Err(detail) => {
            HarnessError::invalid(format!("{error}; collector stderr unavailable: {detail}",))
        }
    }
}

fn reported_calls(summary: &str) -> Option<u64> {
    summary.split_whitespace().rev().find_map(|field| {
        field
            .strip_prefix("api_attempts=")
            .and_then(|value| value.parse().ok())
    })
}

async fn read_counts(database: &str, timeout: Duration) -> Result<Counts, HarnessError> {
    tokio::time::timeout(timeout, async {
    use sqlx::{Connection, Row, postgres::PgConnection};
    let mut connection = PgConnection::connect(database).await?;
    let limit = i64::try_from(MAX_KINDS.saturating_add(1))
        .map_err(|_| HarnessError::invalid("invalid count query limit"))?;
    let result = sqlx::query("SELECT kind, count(*) AS count FROM inventory.resources GROUP BY kind ORDER BY kind LIMIT $1")
        .bind(limit).fetch_all(&mut connection).await;
    let cleanup = connection.close().await;
    let rows = result?;
    cleanup?;
    bounded_rows(rows)?.into_iter()
        .map(|row| {
            let count: i64 = row.try_get("count")?;
            Ok((
                row.try_get("kind")?,
                u64::try_from(count)
                    .map_err(|_| HarnessError::invalid("negative resource count"))?,
            ))
        })
        .collect()
    }).await.map_err(|_| HarnessError::invalid(format!("PostgreSQL counts exceeded {timeout:?}")))?
}

fn bounded_rows<T>(rows: Vec<T>) -> Result<Vec<T>, HarnessError> {
    if rows.len() > MAX_KINDS {
        return Err(HarnessError::invalid("counts exceed 10000 kinds"));
    }
    Ok(rows)
}

pub fn parse_counts(text: &str) -> Result<Counts, HarnessError> {
    text.lines()
        .filter(|line| !line.is_empty())
        .enumerate()
        .try_fold(Counts::new(), |mut counts, (index, line)| {
            if index >= MAX_KINDS {
                return Err(HarnessError::invalid("counts exceed 10000 kinds"));
            }
            let (kind, count) = line.split_once('\t').ok_or_else(|| {
                HarnessError::invalid("count rows must be kind<TAB>unsigned-count")
            })?;
            if kind.is_empty() || counts.contains_key(kind) {
                return Err(HarnessError::invalid("empty or duplicate count kind"));
            }
            let count = count
                .parse()
                .map_err(|_| HarnessError::invalid("count rows must be kind<TAB>unsigned-count"))?;
            counts.insert(kind.into(), count);
            Ok(counts)
        })
}

pub async fn execute_command(
    definition: &Definition,
    argv: &[String],
    context: &Context<'_>,
    stage: &str,
) -> Result<String, HarnessError> {
    let variables = definition_variables(definition, variables(context));
    execute_stage(definition, argv, &variables, context.directory, stage)
        .await
        .map(|output| output.stdout)
}

fn definition_variables(
    definition: &Definition,
    mut variables: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    if let Tool::Collector { binary } = &definition.tool {
        variables.insert(
            "H2H_COLLECTOR".into(),
            binary.to_string_lossy().into_owned(),
        );
    }
    variables
}

async fn execute_stage(
    definition: &Definition,
    argv: &[String],
    variables: &BTreeMap<String, String>,
    directory: &Path,
    stage: &str,
) -> Result<ProcessOutput, HarnessError> {
    let command = external_command(argv, variables, &definition.environment)?;
    execute_process(command, directory, stage).await
}

fn external_command(
    argv: &[String],
    variables: &BTreeMap<String, String>,
    environment: &BTreeMap<String, String>,
) -> Result<Command, HarnessError> {
    let (program, arguments) = argv
        .split_first()
        .ok_or_else(|| HarnessError::invalid("empty command"))?;
    let mut command = Command::new(expand(program, variables));
    command.env_clear();
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    } else {
        command.env("PATH", "/usr/bin:/bin");
    }
    command.envs(variables);
    environment.iter().for_each(|(key, value)| {
        command.env(key, expand(value, variables));
    });
    command.args(arguments.iter().map(|argument| expand(argument, variables)));
    Ok(command)
}

async fn execute_process(
    command: Command,
    directory: &Path,
    stage: &str,
) -> Result<ProcessOutput, HarnessError> {
    execute_process_with_timeout(command, directory, stage, Duration::from_secs(300)).await
}

async fn execute_process_with_timeout(
    command: Command,
    directory: &Path,
    stage: &str,
    timeout: Duration,
) -> Result<ProcessOutput, HarnessError> {
    let stdout = directory.join(format!("{stage}.stdout"));
    let stderr = directory.join(format!("{stage}.stderr"));
    let mut command = command.into_std();
    command
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::fs::File::create(&stdout)?))
        .stderr(Stdio::from(std::fs::File::create(&stderr)?));
    let child = ChildOwner::spawn(&mut command)?;
    let (status, left_processes) = wait_process(child, stage, timeout).await?;
    if !status.success() {
        return Err(HarnessError::invalid(format!(
            "{stage} exited {status}; see {stage}.stderr"
        )));
    }
    Ok(ProcessOutput {
        stdout: verify::read_text(&stdout)?,
        left_processes,
    })
}

async fn wait_process(
    mut child: ChildOwner,
    stage: &str,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, bool), HarnessError> {
    match tokio::time::timeout(timeout, wait_exited(&child)).await {
        Ok(Ok(())) => {
            let remaining = remaining_group_members(child.pid());
            let status = child.terminate()?;
            Ok((status, remaining?))
        }
        Ok(Err(error)) => Err(error),
        Err(_) => Err(timeout_error(stage, timeout, child.terminate())),
    }
}

fn timeout_error(
    stage: &str,
    timeout: Duration,
    terminated: Result<std::process::ExitStatus, HarnessError>,
) -> HarnessError {
    match terminated {
        Ok(_) => HarnessError::invalid(format!(
            "{stage} exceeded {timeout:?}; process group stopped"
        )),
        Err(error) => HarnessError::invalid(format!(
            "{stage} exceeded {timeout:?}; process group termination failed: {error}"
        )),
    }
}

async fn wait_exited(child: &ChildOwner) -> Result<(), HarnessError> {
    use futures_util::StreamExt;
    let polls = futures_util::stream::iter(0..30001)
        .then(|_| async {
            match child.exited() {
                Ok(Some(_)) => Some(Ok(())),
                Ok(None) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    None
                }
                Err(error) => Some(Err(error)),
            }
        })
        .filter_map(std::future::ready);
    tokio::pin!(polls);
    polls
        .next()
        .await
        .ok_or_else(|| HarnessError::invalid("process wait exceeded poll budget"))?
}

fn remaining_group_members(pid: rustix::process::Pid) -> Result<bool, HarnessError> {
    let raw_pid = pid.as_raw_nonzero().get();
    std::fs::read_dir("/proc")?
        .map(|entry| {
            let entry = entry?;
            let name = entry.file_name();
            let Ok(process) = name.to_string_lossy().parse::<i32>() else {
                return Ok(false);
            };
            if process == raw_pid {
                return Ok(false);
            }
            match std::fs::read_to_string(entry.path().join("stat")) {
                Ok(stat) => Ok(stat.rsplit_once(") ").is_some_and(|(_, tail)| {
                    let mut fields = tail.split_whitespace();
                    let alive = fields
                        .next()
                        .is_some_and(|state| state != "Z" && state != "X");
                    alive
                        && fields.nth(1).and_then(|group| group.parse::<i32>().ok())
                            == Some(raw_pid)
                })),
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(rustix::io::Errno::SRCH.raw_os_error()) =>
                {
                    Ok(false)
                }
                Err(error) => Err(error.into()),
            }
        })
        .find(|result| !matches!(result, Ok(false)))
        .transpose()
        .map(|found| found == Some(true))
}

pub fn variables(context: &Context<'_>) -> BTreeMap<String, String> {
    let mut configured = Command::new("environment");
    context.stack.configure(&mut configured);
    let environment = configured
        .as_std()
        .get_envs()
        .filter_map(|(name, value)| {
            value.map(|value| {
                (
                    name.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    command_variables(
        environment,
        context.stack.emulator().endpoint(),
        context.stack.emulator().ca_bundle(),
        context.catalog,
        context.services,
        context.directory,
    )
}

fn command_variables(
    environment: BTreeMap<String, String>,
    endpoint: &str,
    ca_bundle: &Path,
    catalog: &Path,
    services: &[String],
    directory: &Path,
) -> BTreeMap<String, String> {
    [
        ("H2H_ENDPOINT", endpoint.into()),
        ("H2H_CA_BUNDLE", ca_bundle.to_string_lossy().into_owned()),
        ("H2H_REGION", "us-east-1".into()),
        ("H2H_SERVICES", services.join(",")),
        ("H2H_WORK", directory.to_string_lossy().into_owned()),
        ("H2H_CATALOG", catalog.to_string_lossy().into_owned()),
        ("AWS_ENDPOINT_URL", endpoint.into()),
        ("AWS_CA_BUNDLE", ca_bundle.to_string_lossy().into_owned()),
        ("AWS_REGION", "us-east-1".into()),
        ("AWS_DEFAULT_REGION", "us-east-1".into()),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value))
    .fold(environment, |mut environment, (key, value)| {
        environment.insert(key, value);
        environment
    })
}

fn expand(value: &str, variables: &BTreeMap<String, String>) -> String {
    variables
        .iter()
        .fold(value.to_owned(), |value, (key, replacement)| {
            value.replace(&format!("${{{key}}}"), replacement)
        })
}

#[cfg(test)]
mod tests;
