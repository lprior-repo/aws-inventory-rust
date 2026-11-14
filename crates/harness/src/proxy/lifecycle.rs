use super::{
    CONTROL_CAPACITY, Config, MAX_CONNECTIONS, ProxyError, Snapshot, session::Context,
    state::Shared, validated::ValidatedConfig,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{Semaphore, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

/// Terminal task-region certificate. Completed includes failed sessions, whose
/// separate `failed` subset preserves diagnostics without double-counting tasks.
#[derive(Debug, Default)]
pub struct DrainReport {
    /// Total sockets admitted to the region.
    pub accepted: u64,
    /// Sessions ending normally or with a transport/framing failure.
    pub completed: u64,
    /// Subset of completed sessions ending with a typed error.
    pub failed: u64,
    /// Sessions observing cooperative cancellation.
    pub cancelled: u64,
    /// Sessions forcibly aborted after the drain deadline.
    pub aborted: u64,
    /// Sessions whose JoinError reports panic.
    pub panicked: u64,
    /// Most recent session error or unreceived control error, if any.
    pub last_error: Option<ProxyError>,
}

pub(crate) enum Control {
    Refuse,
    RefuseFor(Duration),
    Resume,
}
pub(crate) struct Command {
    pub action: Control,
    pub reply: oneshot::Sender<Result<(), ProxyError>>,
}

/// Owner handle for one loopback listener and its bounded connection region.
/// Drop cancels and aborts the owner; its JoinSet aborts every child on drop.
pub struct Proxy {
    address: SocketAddr,
    endpoint: String,
    state: Shared,
    commands: mpsc::Sender<Command>,
    cancel: CancellationToken,
    owner: Option<JoinHandle<Result<DrainReport, ProxyError>>>,
}
impl Proxy {
    /// Validate bounded configuration, load the configured TLS CA, bind an
    /// ephemeral loopback port and start the single owned supervisor task.
    /// Returns typed configuration, allocation, file or socket failures.
    pub async fn start(config: Config) -> Result<Self, ProxyError> {
        let (config, target) = ValidatedConfig::parse(config).await?;
        let target = Arc::new(target);
        let state = Shared::new(&config)?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let address = listener.local_addr()?;
        let (commands, receiver) = mpsc::channel(CONTROL_CAPACITY);
        let cancel = CancellationToken::new();
        let context = Context::new(config, target, state.clone(), commands.clone(), &cancel);
        let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let owner = tokio::spawn(super::owner::run(listener, receiver, context, permits));
        Ok(Self {
            address,
            endpoint: format!("http://{address}"),
            state,
            commands,
            cancel,
            owner: Some(owner),
        })
    }
    /// Plain HTTP endpoint; upstream TLS is terminated only on the outbound side.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    /// Bound address, stable across refusal/resume cycles.
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    /// Read a bounded, consistent log/counter snapshot; poison is a typed error.
    pub fn snapshot(&self) -> Result<Snapshot, ProxyError> {
        self.state.snapshot()
    }
    /// Close the listening socket and acknowledge once new connects are refused.
    /// Existing sockets are unaffected. Cancellation of this call does not undo it.
    pub async fn refuse(&self) -> Result<(), ProxyError> {
        control(&self.commands, Control::Refuse).await
    }
    /// Rebind the same address and acknowledge readiness, or surface rebind I/O.
    pub async fn resume(&self) -> Result<(), ProxyError> {
        control(&self.commands, Control::Resume).await
    }
    /// Stop intake, cancel sessions, drain to DRAIN_TIMEOUT, then abort and join
    /// stragglers. Auto-resume rebind/accept errors are surfaced after cleanup.
    pub async fn shutdown(mut self) -> Result<DrainReport, ProxyError> {
        self.cancel.cancel();
        let outcome = self.owner.as_mut().ok_or(ProxyError::Stopped)?.await;
        drop(self.owner.take());
        match outcome {
            Ok(result) => result,
            Err(error) if error.is_panic() => Err(ProxyError::OwnerPanicked),
            Err(_) => Err(ProxyError::Stopped),
        }
    }
}
impl Drop for Proxy {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(owner) = &self.owner {
            owner.abort();
        }
    }
}

pub(crate) async fn control(
    commands: &mpsc::Sender<Command>,
    action: Control,
) -> Result<(), ProxyError> {
    let (reply, receiver) = oneshot::channel();
    commands
        .send(Command { action, reply })
        .await
        .map_err(|_| ProxyError::Stopped)?;
    receiver.await.map_err(|_| ProxyError::Stopped)?
}
