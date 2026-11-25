mod frame_harness;
mod graph_tasks;
mod lifecycle;
mod projection;
mod protocol;
mod queries;
mod runner;
#[cfg(test)]
mod test_events;
#[cfg(test)]
pub(crate) mod tests;

use crate::data::{Counts, Filter, Graph, Row};
use cmdb_view::{Database, GraphConnection, GraphLimits, GraphNode, PageRequest, ResourceFilter};
use graph_tasks::{GraphAccess, database_error, graph_failure};
use lifecycle::{Cancellation, Shutdown, shutdown_channel};
use makepad_widgets::makepad_platform::thread::SignalToUI;
use projection::{graph_from_edges, row_from_resource};
use protocol::{COMMAND_CAPACITY, Command, EVENT_CAPACITY, Output, Revisions, current_events};
use queries::{query_graph, refresh_page};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{Receiver as CommandReceiver, Sender as CommandSender};

const PAGE_LIMIT: usize = 500;
const TAG_LIMIT: usize = 256;
pub const DEFAULT_GRAPH_LIMITS: GraphLimits = GraphLimits {
    depth: 2,
    nodes: 200,
    edges: 500,
};

pub fn parse_lod_config(config: &str) -> Result<GraphLimits, String> {
    let parts: Vec<&str> = config.split(':').collect();
    let [depth_part, nodes_part, edges_part] = parts.as_slice() else {
        return Err(format!(
            "CMDB_UI_LOD requires depth:nodes:edges, got: {config}"
        ));
    };
    let depth: u8 = depth_part
        .parse()
        .map_err(|_| format!("invalid depth: {depth_part}"))?;
    let nodes: usize = nodes_part
        .parse()
        .map_err(|_| format!("invalid nodes: {nodes_part}"))?;
    let edges: usize = edges_part
        .parse()
        .map_err(|_| format!("invalid edges: {edges_part}"))?;
    Ok(GraphLimits {
        depth,
        nodes,
        edges,
    })
}

#[derive(Clone, PartialEq)]
pub struct WorkerConfig {
    pub database: Option<String>,
    pub graph_url: Option<String>,
    pub graph_name: Option<String>,
    pub graph_limits: GraphLimits,
    pub frame_harness: Option<crate::frame_harness::FrameConfig>,
}

impl std::fmt::Debug for WorkerConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let database = self.database.as_deref().map(|target| {
            if cmdb_view::is_postgres_url(target) {
                "[PostgreSQL URL redacted]"
            } else {
                target
            }
        });
        formatter
            .debug_struct("WorkerConfig")
            .field("database", &database)
            .field("graph_url", &self.graph_url)
            .field("graph_name", &self.graph_name)
            .field("graph_limits", &self.graph_limits)
            .field("frame_harness", &self.frame_harness)
            .finish()
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            database: None,
            graph_url: None,
            graph_name: None,
            graph_limits: DEFAULT_GRAPH_LIMITS,
            frame_harness: None,
        }
    }
}

#[derive(Debug)]
pub enum WorkerEvent {
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
    HarnessGraph {
        graph: crate::dag::graph::DagGraph,
        resources: u64,
        edges: u64,
    },
}

pub struct DataWorker {
    commands: CommandSender<Command>,
    events: Option<Receiver<Output>>,
    revisions: Arc<Revisions>,
    shutdown: Shutdown,
    thread: Option<JoinHandle<()>>,
    #[cfg(test)]
    pending_test_events: std::cell::RefCell<test_events::WaitState>,
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
            #[cfg(test)]
            pending_test_events: std::cell::RefCell::default(),
        }
    }

    pub fn refresh(&self, filter: Filter) -> Result<(), String> {
        let permit = self.commands.try_reserve().map_err(command_error)?;
        #[cfg(test)]
        self.check_test_command()?;
        let revision = self.revisions.refresh()?;
        #[cfg(test)]
        self.pending_test_events
            .borrow_mut()
            .begin(revision, true)?;
        permit.send(Command::Refresh { revision, filter });
        Ok(())
    }

    pub fn expand(&self, node: GraphNode) -> Result<(), String> {
        let permit = self.commands.try_reserve().map_err(command_error)?;
        #[cfg(test)]
        self.check_test_command()?;
        let revision = self.revisions.expand()?;
        #[cfg(test)]
        self.pending_test_events
            .borrow_mut()
            .begin(revision, false)?;
        permit.send(Command::Expand { revision, node });
        Ok(())
    }

    pub fn invalidate_graph(&self) -> Result<(), String> {
        self.revisions.expand()?;
        #[cfg(test)]
        self.pending_test_events.borrow_mut().cancel_graph();
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.commands.is_closed()
    }

    pub fn drain(&self) -> Vec<WorkerEvent> {
        self.events.as_ref().map_or_else(Vec::new, |events| {
            #[cfg(test)]
            {
                self.drain_test_events(events.try_iter())
            }
            #[cfg(not(test))]
            {
                current_events(events.try_iter(), &self.revisions)
            }
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
        None => return Err("missing --database PATH".to_owned()),
    };
    tokio::time::timeout(Duration::from_secs(10), setup(path, config))
        .await
        .map_err(|_| "database startup timed out after 10 seconds".to_owned())?
}

async fn setup(path: String, config: WorkerConfig) -> Result<(WorkerState, WorkerEvent), String> {
    let database = Database::open(&path)
        .await
        .map_err(|error| error.to_string())?;
    let initial = if config.frame_harness.is_some() {
        let (graph, resources, edges) = frame_harness::load(&database).await?;
        WorkerEvent::HarnessGraph {
            graph,
            resources,
            edges,
        }
    } else {
        refresh_page(&database, Filter::default()).await
    };
    let graph = GraphAccess::new(config.graph_url, config.graph_name, config.graph_limits);
    let state = WorkerState { graph, database };
    Ok((state, initial))
}
