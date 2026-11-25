#[cfg(test)]
mod tests;

use super::*;

pub(super) struct WorkerContext {
    commands: CommandReceiver<Command>,
    events: SyncSender<Output>,
    signal: SignalToUI,
    cancellation: Cancellation,
    revisions: Arc<Revisions>,
}

impl WorkerContext {
    pub fn new(
        commands: CommandReceiver<Command>,
        events: SyncSender<Output>,
        signal: SignalToUI,
        cancellation: Cancellation,
        revisions: Arc<Revisions>,
    ) -> Self {
        Self {
            commands,
            events,
            signal,
            cancellation,
            revisions,
        }
    }

    fn emit(&self, revision: u64, event: WorkerEvent) -> bool {
        let output = Output {
            revision: Some(revision),
            event,
        };
        if self.cancellation.is_stopped() || !self.revisions.accepts(&output) {
            return !self.cancellation.is_stopped();
        }
        self.publish(output)
    }

    fn publish(&self, output: Output) -> bool {
        match self.events.send(output) {
            Ok(()) => {
                self.signal.set();
                true
            }
            Err(error) => {
                tracing::debug!(error = %error, "CMDB UI disconnected worker output");
                false
            }
        }
    }

    fn report_failure(&self, error: String) {
        let graph = graph_failure(format!("database unavailable · {error}"));
        if self.publish(Output {
            revision: None,
            event: graph,
        }) {
            self.publish(Output {
                revision: None,
                event: database_error(error),
            });
        }
    }

    fn run_commands(&mut self, runtime: &tokio::runtime::Runtime, mut state: WorkerState) {
        let result = std::iter::repeat(()).try_for_each(|()| {
            let Some(command) = self.commands.blocking_recv() else {
                return Err(());
            };
            if self.cancellation.is_stopped() || matches!(command, Command::Stop) {
                return Err(());
            }
            if !self.revisions.accepts_command(&command) {
                return Ok(());
            }
            self.process(command, runtime, &mut state)
                .then_some(())
                .ok_or(())
        });
        if result.is_err() {
            tracing::debug!("CMDB worker command intake closed");
        }
    }

    fn process(
        &mut self,
        command: Command,
        runtime: &tokio::runtime::Runtime,
        state: &mut WorkerState,
    ) -> bool {
        let _entered = runtime.enter();
        match command {
            Command::Refresh { revision, filter } => {
                let Some(event) =
                    runtime.block_on(self.cancellation.run(refresh_page(&state.database, filter)))
                else {
                    return false;
                };
                let output = Output {
                    revision: Some(revision),
                    event,
                };
                if !self.revisions.accepts(&output) {
                    return true;
                }
                if !self.publish(output) {
                    return false;
                }
                let status = runtime.block_on(self.cancellation.run(query_graph(state, None)));
                status.is_some_and(|event| {
                    (matches!(event, WorkerEvent::GraphStatus { .. })
                        || self.emit(revision, state.graph.status()))
                        && self.emit(revision, event)
                })
            }
            Command::Expand { revision, node } => {
                let event = runtime.block_on(self.cancellation.run(query_graph(state, Some(node))));
                event.is_some_and(|event| {
                    self.emit(revision, state.graph.status()) && self.emit(revision, event)
                })
            }
            Command::Stop => false,
        }
    }
}

pub(super) fn run_worker(config: WorkerConfig, mut context: WorkerContext) {
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            context.report_failure(format!("worker runtime unavailable · {error}"));
            return;
        }
    };
    let _entered = runtime.enter();
    let startup = runtime.block_on(context.cancellation.run(startup(config)));
    match startup {
        Some(Ok((state, initial))) => {
            if context.emit(0, state.graph.status()) && context.emit(0, initial) {
                context.run_commands(&runtime, state);
            }
        }
        Some(Err(error)) => context.report_failure(error),
        None => tracing::debug!("CMDB worker startup cancelled"),
    }
    runtime.shutdown_timeout(Duration::from_millis(100));
}
