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

#[derive(Debug, Default)]
pub struct DrainReport {
    pub accepted: u64,

    pub completed: u64,

    pub failed: u64,

    pub cancelled: u64,

    pub aborted: u64,

    pub panicked: u64,

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

pub struct Proxy {
    address: SocketAddr,
    endpoint: String,
    state: Shared,
    commands: mpsc::Sender<Command>,
    cancel: CancellationToken,
    owner: Option<JoinHandle<Result<DrainReport, ProxyError>>>,
}
impl Proxy {
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

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn snapshot(&self) -> Result<Snapshot, ProxyError> {
        self.state.snapshot()
    }

    pub async fn refuse(&self) -> Result<(), ProxyError> {
        control(&self.commands, Control::Refuse).await
    }

    pub async fn resume(&self) -> Result<(), ProxyError> {
        control(&self.commands, Control::Resume).await
    }

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
