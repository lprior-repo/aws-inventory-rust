use super::{DataWorker, EVENT_CAPACITY, Output, WorkerEvent};

pub(super) struct WaitState {
    active: bool,
    revision: u64,
    snapshot: bool,
    graph: bool,
    selected: bool,
    pending: Vec<WorkerEvent>,
    last_status: Option<(Option<u64>, String)>,
}

impl Default for WaitState {
    fn default() -> Self {
        Self {
            active: false,
            revision: 0,
            snapshot: true,
            graph: false,
            selected: false,
            pending: Vec::new(),
            last_status: None,
        }
    }
}

impl WaitState {
    pub fn begin(&mut self, revision: u64, refresh: bool) -> Result<(), String> {
        if self.active {
            self.ready()?;
        }
        self.active = true;
        self.revision = revision;
        self.selected |= !refresh;
        self.snapshot = refresh;
        self.graph = self.selected;
        Ok(())
    }

    pub(super) fn cancel_graph(&mut self) {
        self.graph = false;
        self.pending
            .retain(|event| !matches!(event, WorkerEvent::Graph { .. }));
    }

    fn ready(&self) -> Result<(), String> {
        let leftover = match (self.graph, self.snapshot) {
            (true, true) => "Graph and Snapshot",
            (true, false) => "Graph",
            (false, true) => "Snapshot",
            (false, false) => return Ok(()),
        };
        Err(format!(
            "real worker command would abandon unconsumed {leftover}"
        ))
    }

    fn receive(&mut self, outputs: impl Iterator<Item = Output>) -> Result<(), String> {
        for output in outputs.take(EVENT_CAPACITY) {
            if let WorkerEvent::GraphStatus { error } = output.event {
                self.last_status = error.map(|error| (output.revision, error));
                continue;
            }
            let kind = match &output.event {
                WorkerEvent::Snapshot { .. } => "Snapshot",
                WorkerEvent::Graph { .. } => "Graph",
                other => return Err(format!("unexpected real worker event: {other:?}")),
            };
            if output.revision != Some(self.revision)
                && !(output.revision.is_none() && self.revision == 0)
            {
                return Err(format!(
                    "real worker {kind} revision mismatch: expected {}, received {:?}",
                    self.revision, output.revision
                ));
            }
            if self.pending.len() >= EVENT_CAPACITY {
                return Err("real worker pending event budget exhausted".to_owned());
            }
            match &output.event {
                WorkerEvent::Snapshot { .. } => self.snapshot = true,
                WorkerEvent::Graph { .. } => self.graph = true,
                _ => {}
            }
            self.pending.push(output.event);
        }
        Ok(())
    }

    fn take(&mut self, snapshot: bool) -> Option<WorkerEvent> {
        let index = self
            .pending
            .iter()
            .position(|event| matches!(event, WorkerEvent::Snapshot { .. }) == snapshot)?;
        let event = self.pending.remove(index);
        if snapshot {
            self.snapshot = false;
        } else {
            self.graph = false;
        }
        Some(event)
    }

    fn drain(
        &mut self,
        outputs: impl Iterator<Item = Output>,
        revisions: &super::Revisions,
    ) -> Vec<WorkerEvent> {
        let revision = Some(self.revision);
        let pending = std::mem::take(&mut self.pending)
            .into_iter()
            .map(|event| Output { revision, event });
        let events = super::current_events(
            pending.chain(outputs).inspect(|output| {
                if let WorkerEvent::GraphStatus { error } = &output.event {
                    self.last_status = error.clone().map(|error| (output.revision, error));
                }
            }),
            revisions,
        );
        for event in &events {
            match event {
                WorkerEvent::Snapshot { .. } => {
                    self.active = true;
                    self.snapshot = false;
                }
                WorkerEvent::Graph { .. } => {
                    self.active = true;
                    self.graph = false;
                }
                _ => {}
            }
        }
        events
    }
}

impl DataWorker {
    pub(super) fn check_test_command(&self) -> Result<(), String> {
        let mut state = self.pending_test_events.borrow_mut();
        if !state.active {
            return Ok(());
        }
        if let Some(outputs) = &self.events {
            state.receive(
                outputs
                    .try_iter()
                    .filter(|output| self.revisions.accepts(output)),
            )?;
        }
        state.ready()
    }

    pub(super) fn drain_test_events(
        &self,
        outputs: impl Iterator<Item = Output>,
    ) -> Vec<WorkerEvent> {
        self.pending_test_events
            .borrow_mut()
            .drain(outputs, &self.revisions)
    }

    pub(crate) fn test_commands_consumed(&self) -> bool {
        self.pending_test_events.borrow().ready().is_ok()
    }

    pub(super) fn test_event(&self, snapshot: bool) -> Result<Option<WorkerEvent>, String> {
        let mut state = self.pending_test_events.borrow_mut();
        state.active = true;
        if let Some(outputs) = &self.events {
            state.receive(outputs.try_iter())?;
        }
        Ok(state.take(snapshot))
    }

    pub(super) fn last_test_graph_status(&self) -> String {
        match &self.pending_test_events.borrow().last_status {
            Some((revision, status)) => format!("revision {revision:?}: {status}"),
            None => "none".to_owned(),
        }
    }
}

#[test]
fn given_active_accounting_when_beginning_another_command_then_begin_rejects_unread_terminals()
-> Result<(), String> {
    let mut state = WaitState::default();
    state.begin(1, true)?;
    assert_eq!(
        state.begin(2, false),
        Err("real worker command would abandon unconsumed Snapshot".to_owned())
    );
    assert_eq!(state.revision, 1);
    assert!(state.active);
    Ok(())
}
