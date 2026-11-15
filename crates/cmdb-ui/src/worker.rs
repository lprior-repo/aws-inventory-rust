#[path = "worker/graph_tasks.rs"]
mod graph_tasks;
use graph_tasks::*;
mod projection;

use crate::data::{Counts, Filter, Graph, Row};
use cmdb_view::{Database, GraphConnection, GraphLimits, GraphNode, PageRequest, ResourceFilter};
use makepad_widgets::makepad_platform::thread::SignalToUI;
use projection::{graph_from_edges, row_from_resource};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::runtime::Builder;

const PAGE_LIMIT: usize = 500;
const GRAPH_LIMITS: GraphLimits = GraphLimits {
    depth: 2,
    nodes: 200,
    edges: 500,
};

#[derive(Clone, Debug, Default)]
pub struct WorkerConfig {
    pub database: Option<PathBuf>,
    pub graph_url: Option<String>,
    pub graph_name: Option<String>,
    pub argument_error: Option<String>,
}

impl WorkerConfig {
    pub fn from_process_args() -> Self {
        let arguments: Vec<String> = std::env::args().skip(1).collect();
        let mut config = Self::default();
        match parse_arguments(&arguments, &mut config) {
            Ok(()) => config,
            Err(error) => Self {
                argument_error: Some(error),
                ..Self::default()
            },
        }
    }
}

fn parse_arguments(arguments: &[String], config: &mut WorkerConfig) -> Result<(), String> {
    match arguments {
        [] => Ok(()),
        [flag, remaining @ ..] if flag == "--hot" || flag == "--stdin-loop" => {
            parse_arguments(remaining, config)
        }
        [flag, remaining @ ..] if flag.starts_with("--linux-backend=") => {
            parse_arguments(remaining, config)
        }
        [flag, value, remaining @ ..] => {
            set_argument(config, flag, value)?;
            parse_arguments(remaining, config)
        }
        [flag] => Err(format!("missing value for {flag}")),
    }
}

fn set_argument(config: &mut WorkerConfig, flag: &str, value: &str) -> Result<(), String> {
    match flag {
        "--database" => config.database = Some(PathBuf::from(value)),
        "--graph-url" => config.graph_url = Some(value.to_owned()),
        "--graph-name" => config.graph_name = Some(value.to_owned()),
        _ => {
            return Err(format!(
                "unknown argument {flag}; expected --database PATH [--graph-url URL --graph-name NAME]"
            ));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum WorkerEvent {
    Snapshot {
        counts: Counts,
        rows: Vec<Row>,
        total: u64,
        error: Option<String>,
    },
    Graph {
        graph: Graph,
        error: Option<String>,
    },
}

enum Command {
    Refresh(Filter),
    Expand(GraphNode),
    Stop,
}

pub struct DataWorker {
    commands: SyncSender<Command>,
    events: Receiver<WorkerEvent>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl DataWorker {
    pub fn start(config: WorkerConfig, signal: SignalToUI) -> Self {
        let (command_tx, command_rx) = mpsc::sync_channel(32);
        let (event_tx, event_rx) = mpsc::sync_channel(64);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker =
            thread::spawn(move || run_worker(config, command_rx, event_tx, signal, worker_stop));
        Self {
            commands: command_tx,
            events: event_rx,
            stop,
            thread: Some(worker),
        }
    }

    pub fn refresh(&self, filter: Filter) -> Result<(), String> {
        self.commands
            .try_send(Command::Refresh(filter))
            .map_err(command_error)
    }

    pub fn expand(&self, node: GraphNode) -> Result<(), String> {
        self.commands
            .try_send(Command::Expand(node))
            .map_err(command_error)
    }

    pub fn drain(&self) -> Vec<WorkerEvent> {
        self.events.try_iter().collect()
    }
}

impl Drop for DataWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Err(error) = self.commands.try_send(Command::Stop) {
            tracing::debug!(error = %command_error(error), "stop flag set without queued stop command");
        }
        if let Some(worker) = self.thread.take()
            && worker.join().is_err()
        {
            tracing::error!("CMDB data worker panicked");
        }
    }
}

fn command_error(error: TrySendError<Command>) -> String {
    match error {
        TrySendError::Full(_) => "inventory worker queue is full".to_owned(),
        TrySendError::Disconnected(_) => "inventory worker stopped".to_owned(),
    }
}

fn run_worker(
    config: WorkerConfig,
    commands: Receiver<Command>,
    events: SyncSender<WorkerEvent>,
    signal: SignalToUI,
    stop: Arc<AtomicBool>,
) {
    let Some(runtime) = worker_runtime(&events, &signal) else {
        return;
    };
    match runtime.block_on(startup(config)) {
        Ok((state, initial)) => {
            let context = WorkerContext {
                commands,
                runtime,
                state,
                events,
                signal,
                stop,
            };
            run_commands(context, initial);
        }
        Err(error) => report_failure(&events, &signal, error, "database unavailable"),
    }
}

fn worker_runtime(
    events: &SyncSender<WorkerEvent>,
    signal: &SignalToUI,
) -> Option<tokio::runtime::Runtime> {
    match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => Some(runtime),
        Err(error) => {
            let context = "database worker runtime unavailable";
            report_failure(events, signal, error.to_string(), context);
            None
        }
    }
}

fn report_failure(
    events: &SyncSender<WorkerEvent>,
    signal: &SignalToUI,
    error: String,
    context: &str,
) {
    send_event(events, signal, database_error(error.clone()));
    send_event(
        events,
        signal,
        graph_failure(format!("{context} · {error}")),
    );
}

struct WorkerContext {
    commands: Receiver<Command>,
    runtime: tokio::runtime::Runtime,
    state: WorkerState,
    events: SyncSender<WorkerEvent>,
    signal: SignalToUI,
    stop: Arc<AtomicBool>,
}

fn run_commands(context: WorkerContext, initial: WorkerEvent) {
    let WorkerContext {
        commands,
        runtime,
        mut state,
        events,
        signal,
        stop,
    } = context;
    send_event(&events, &signal, initial);
    send_event(
        &events,
        &signal,
        WorkerEvent::Graph {
            graph: Graph::default(),
            error: state.graph_error.clone(),
        },
    );
    commands
        .iter()
        .take_while(|command| !matches!(command, Command::Stop) && !stop.load(Ordering::Acquire))
        .for_each(|command| process_command(command, &runtime, &mut state, &events, &signal));
}

struct WorkerState {
    database: Database,
    graph: Option<GraphConnection>,
    graph_error: Option<String>,
}
async fn startup(mut config: WorkerConfig) -> Result<(WorkerState, WorkerEvent), String> {
    let path = match config.database.take() {
        Some(path) => path,
        None => {
            return Err(config
                .argument_error
                .unwrap_or_else(|| "missing --database PATH".to_owned()));
        }
    };
    tokio::time::timeout(Duration::from_secs(10), setup(path, config))
        .await
        .map_err(|_| "database startup timed out after 10 seconds".to_owned())?
}

async fn setup(path: PathBuf, config: WorkerConfig) -> Result<(WorkerState, WorkerEvent), String> {
    let database = Database::open(&path)
        .await
        .map_err(|error| error.to_string())?;
    let counts = database.counts().await.map_err(|error| error.to_string())?;
    let (rows, total) = load_page(&database, Filter::default()).await?;
    let (graph, graph_error) = graph_config(config.graph_url, config.graph_name);
    let state = WorkerState {
        database,
        graph,
        graph_error,
    };
    let initial = WorkerEvent::Snapshot {
        counts: counts.into(),
        rows,
        total,
        error: None,
    };
    Ok((state, initial))
}
