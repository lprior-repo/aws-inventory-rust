use super::{Filter, GraphNode, WorkerEvent};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) const COMMAND_CAPACITY: usize = 32;
pub(super) const EVENT_CAPACITY: usize = 64;

pub(super) enum Command {
    Refresh { revision: u64, filter: Filter },
    Expand { revision: u64, node: GraphNode },
    Stop,
}

pub(super) struct Output {
    pub revision: Option<u64>,
    pub event: WorkerEvent,
}

#[derive(Default)]
pub(super) struct Revisions {
    serial: AtomicU64,
    page: AtomicU64,
    graph: AtomicU64,
}

impl Revisions {
    pub fn refresh(&self) -> Result<u64, String> {
        let revision = self.next()?;
        self.page.store(revision, Ordering::Release);
        self.graph.store(revision, Ordering::Release);
        Ok(revision)
    }

    pub fn expand(&self) -> Result<u64, String> {
        let revision = self.next()?;
        self.graph.store(revision, Ordering::Release);
        Ok(revision)
    }

    fn next(&self) -> Result<u64, String> {
        self.serial
            .try_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map(|previous| previous.saturating_add(1))
            .map_err(|_| "inventory request revision exhausted".to_owned())
    }

    pub fn accepts(&self, output: &Output) -> bool {
        let Some(revision) = output.revision else {
            return true;
        };
        let latest = match &output.event {
            WorkerEvent::Snapshot { .. } | WorkerEvent::GraphStatus { .. } => &self.page,
            WorkerEvent::Graph { .. } | WorkerEvent::HarnessGraph { .. } => &self.graph,
        };
        latest.load(Ordering::Acquire) == revision
    }

    pub fn accepts_command(&self, command: &Command) -> bool {
        match command {
            Command::Refresh { revision, .. } => self.page.load(Ordering::Acquire) == *revision,
            Command::Expand { revision, .. } => self.graph.load(Ordering::Acquire) == *revision,
            Command::Stop => false,
        }
    }
}

pub(super) fn current_events(
    outputs: impl Iterator<Item = Output>,
    revisions: &Revisions,
) -> Vec<WorkerEvent> {
    let (status, graph, page) = outputs
        .take(EVENT_CAPACITY)
        .filter(|output| revisions.accepts(output))
        .fold(
            (None, None, None),
            |(status, graph, page), output| match output.event {
                event @ WorkerEvent::GraphStatus { .. } => (Some(event), graph, page),
                event @ (WorkerEvent::Graph { .. } | WorkerEvent::HarnessGraph { .. }) => (status, Some(event), page),
                event @ WorkerEvent::Snapshot { .. } => (status, graph, Some(event)),
            },
        );
    status.into_iter().chain(graph).chain(page).collect()
}

#[cfg(test)]
#[path = "protocol/tests.rs"]
mod tests;
