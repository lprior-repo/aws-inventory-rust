use super::{
    DRAIN_TIMEOUT, MAX_CONNECTIONS, MAX_OWNER_EVENTS, ProxyError,
    lifecycle::{Command, Control, DrainReport},
    session::{Context, Outcome},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore, mpsc},
    task::{JoinError, JoinSet},
    time::Instant,
};

type Joined = Result<Outcome, JoinError>;
type Admitted = Result<(TcpStream, OwnedSemaphorePermit), ProxyError>;
enum Event {
    Cancel,
    Command(Option<Command>),
    Joined(Option<Joined>),
    Accepted(Admitted),
    Resume,
}
enum ResumePolicy {
    Manual,
    Until(Instant),
}
enum ListenerState {
    Accepting(TcpListener),
    Refusing(ResumePolicy),
}
struct Owner {
    intake: ListenerState,
    address: SocketAddr,
    commands: mpsc::Receiver<Command>,
    context: Context,
    permits: Arc<Semaphore>,
    tasks: JoinSet<Outcome>,
    report: DrainReport,
}

pub(crate) async fn run(
    listener: TcpListener,
    commands: mpsc::Receiver<Command>,
    context: Context,
    permits: Arc<Semaphore>,
) -> Result<DrainReport, ProxyError> {
    let address = listener.local_addr()?;
    let mut owner = Owner {
        intake: ListenerState::Accepting(listener),
        address,
        commands,
        context,
        permits,
        tasks: JoinSet::new(),
        report: DrainReport::default(),
    };
    let result = owner.drive().await;
    owner.stop().await;
    result?;
    Ok(owner.report)
}
impl Owner {
    async fn stop(&mut self) {
        self.intake = ListenerState::Refusing(ResumePolicy::Manual);
        self.context.cancel.cancel();
        if tokio::time::timeout(DRAIN_TIMEOUT, self.drain())
            .await
            .is_err()
        {
            self.tasks.abort_all();
            self.drain().await;
        }
    }
    async fn drive(&mut self) -> Result<(), ProxyError> {
        for _ in 0..MAX_OWNER_EVENTS {
            match self.event().await {
                Event::Cancel | Event::Command(None) => return Ok(()),
                Event::Joined(Some(result)) => self.report.record(result),
                Event::Joined(None) => return Err(ProxyError::Stopped),
                Event::Accepted(result) => self.spawn(result?)?,
                Event::Command(Some(command)) => self.command(command).await,
                Event::Resume => {
                    self.rebind().await?;
                }
            }
        }
        Err(ProxyError::Limit("owner event count"))
    }
    async fn event(&mut self) -> Event {
        tokio::select! {
            biased;
            () = self.context.cancel.cancelled() => Event::Cancel,
            result = self.tasks.join_next(), if !self.tasks.is_empty() => Event::Joined(result),
            command = self.commands.recv() => Event::Command(command),
            () = resume_timer(&self.intake) => Event::Resume,
            result = admit(&self.intake, &self.permits), if self.tasks.len() < MAX_CONNECTIONS
                => Event::Accepted(result),
        }
    }
    fn spawn(&mut self, admitted: (TcpStream, OwnedSemaphorePermit)) -> Result<(), ProxyError> {
        let (socket, permit) = admitted;
        self.context.state.accepted()?;
        self.report.accepted = self.report.accepted.saturating_add(1);
        let context = self.context.clone();
        drop(self.tasks.spawn(async move {
            let _permit = permit;
            super::session::run(socket, context).await
        }));
        Ok(())
    }
    async fn command(&mut self, command: Command) {
        let result = match command.action {
            Control::Refuse => {
                self.refuse(ResumePolicy::Manual);
                Ok(())
            }
            Control::RefuseFor(duration) => match Instant::now().checked_add(duration) {
                Some(deadline) => {
                    self.refuse(ResumePolicy::Until(deadline));
                    Ok(())
                }
                None => Err(ProxyError::Limit("refusal deadline")),
            },
            Control::Resume => self.rebind().await,
        };
        // A cancelled caller can discard its acknowledgement; the operation has
        // committed. Preserve an unreceived error in the supervisor report.
        if let Err(Err(error)) = command.reply.send(result) {
            self.report.last_error = Some(error);
        }
    }
    fn refuse(&mut self, policy: ResumePolicy) {
        self.intake = ListenerState::Refusing(policy);
    }
    async fn rebind(&mut self) -> Result<(), ProxyError> {
        if matches!(self.intake, ListenerState::Refusing(_)) {
            let listener = TcpListener::bind(self.address).await?;
            self.intake = ListenerState::Accepting(listener);
        }
        Ok(())
    }
    async fn drain(&mut self) {
        // JoinSet length never exceeds MAX_CONNECTIONS, including completed but
        // not yet reaped tasks; no permit is awaited during drain.
        for _ in 0..MAX_CONNECTIONS {
            match self.tasks.join_next().await {
                Some(result) => self.report.record(result),
                None => return,
            }
        }
    }
}

async fn admit(intake: &ListenerState, permits: &Arc<Semaphore>) -> Admitted {
    let ListenerState::Accepting(listener) = intake else {
        return std::future::pending().await;
    };
    let permit = permits
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ProxyError::Stopped)?;
    let (socket, _) = listener.accept().await?;
    Ok((socket, permit))
}
async fn resume_timer(intake: &ListenerState) {
    match intake {
        ListenerState::Refusing(ResumePolicy::Until(at)) => tokio::time::sleep_until(*at).await,
        ListenerState::Accepting(_) | ListenerState::Refusing(ResumePolicy::Manual) => {
            std::future::pending().await
        }
    }
}
impl DrainReport {
    fn record(&mut self, joined: Joined) {
        match joined {
            Ok(Outcome::Completed) => self.completed = self.completed.saturating_add(1),
            Ok(Outcome::Failed(error)) => {
                self.completed = self.completed.saturating_add(1);
                self.failed = self.failed.saturating_add(1);
                self.last_error = Some(error);
            }
            Ok(Outcome::Cancelled) => self.cancelled = self.cancelled.saturating_add(1),
            Err(error) if error.is_panic() => self.panicked = self.panicked.saturating_add(1),
            Err(_) => self.aborted = self.aborted.saturating_add(1),
        }
    }
}
