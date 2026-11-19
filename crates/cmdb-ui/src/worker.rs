mod graph_tasks;
mod lifecycle;
mod projection;
mod protocol;
mod queries;
mod runner;
#[cfg(test)]
pub(crate) mod tests;

use crate::data::{Counts, Filter, Graph, Row};
use cmdb_view::{
    Database, GraphConnection, GraphLimits, GraphNode, PageRequest, ResourceFilter,
};
use graph_tasks::{GraphAccess, database_error, graph_failure};
use lifecycle::{Cancellation, Shutdown, shutdown_channel};
use makepad_widgets::makepad_platform::thread::SignalToUI;
use projection::{graph_from_edges, row_from_resource};
use protocol::{COMMAND_CAPACITY, Command, EVENT_CAPACITY, Output, Revisions, current_events};
use queries::{query_graph, refresh_page};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{Receiver as CommandReceiver, Sender as CommandSender};

const PAGE_LIMIT: usize = 500;
const TAG_LIMIT: usize = 256;
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
    /// Connection/configuration status, independent of a selected resource.
    GraphStatus {
        error: Option<String>,
    },
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

pub struct DataWorker {
    commands: CommandSender<Command>,
    events: Option<Receiver<Output>>,
    revisions: Arc<Revisions>,
    shutdown: Shutdown,
    thread: Option<JoinHandle<()>>,
}

impl DataWorker {
    pub fn start(config: WorkerConfig, signal: SignalToUI) -> Self {
        let (commands, receiver) = tokio::sync::mpsc::channel(COMMAND_CAPACITY);
        let (events, outputs) = mpsc::sync_channel(EVENT_CAPACITY);
        let revisions = Arc::new(Revisions::default());
        let (shutdown, cancellation) = shutdown_channel();
        let worker_revisions = Arc::clone(&revisions);
        let thread = thread::spawn(move || {
            let context = runner::WorkerContext::new(
                receiver,
                events,
                signal,
                cancellation,
                worker_revisions,
            );
            runner::run_worker(config, context);
        });
        Self {
            commands,
            events: Some(outputs),
            revisions,
            shutdown,
            thread: Some(thread),
        }
    }

    pub fn refresh(&self, filter: Filter) -> Result<(), String> {
        let permit = self.commands.try_reserve().map_err(command_error)?;
        let revision = self.revisions.refresh()?;
        permit.send(Command::Refresh { revision, filter });
        Ok(())
    }

    pub fn expand(&self, node: GraphNode) -> Result<(), String> {
        let permit = self.commands.try_reserve().map_err(command_error)?;
        let revision = self.revisions.expand()?;
        permit.send(Command::Expand { revision, node });
        Ok(())
    }

    pub fn invalidate_graph(&self) -> Result<(), String> {
        self.revisions.expand().map(|_| ())
    }

    pub fn is_closed(&self) -> bool {
        self.commands.is_closed()
    }

    pub fn drain(&self) -> Vec<WorkerEvent> {
        self.events.as_ref().map_or_else(Vec::new, |events| {
            current_events(events.try_iter(), &self.revisions)
        })
    }
}

impl Drop for DataWorker {
    fn drop(&mut self) {
        self.shutdown.stop();
        drop(self.events.take());
        if let Err(error) = self.commands.try_send(Command::Stop) {
            tracing::debug!(error = %error, "CMDB worker stop already signaled");
        }
        if let Some(worker) = self.thread.take()
            && worker.join().is_err()
        {
            tracing::error!("CMDB data worker panicked");
        }
    }
}

fn command_error(error: tokio::sync::mpsc::error::TrySendError<()>) -> String {
    match error {
        tokio::sync::mpsc::error::TrySendError::Full(()) => {
            "inventory worker queue is full".to_owned()
        }
        tokio::sync::mpsc::error::TrySendError::Closed(()) => "inventory worker stopped".to_owned(),
    }
}

struct WorkerState {
    graph: GraphAccess,
    database: Database,
}
async fn startup(mut config: WorkerConfig) -> Result<(WorkerState, WorkerEvent), String> {
    let path = match config.database.take() {
        Some(path) => path,
        None => {
            return Err(match config.argument_error {
                Some(error) => error,
                None => "missing --database PATH".to_owned(),
            });
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
    // Get the initial page using a short-lived snapshot.
    let initial = refresh_page(&database, Filter::default()).await;
    let graph = GraphAccess::new(config.graph_url, config.graph_name);
    let state = WorkerState {
        graph,
        database,
    };
    Ok((state, initial))
}
