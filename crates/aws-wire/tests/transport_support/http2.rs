use super::{TestError, TestResult, wire::Endpoint};
use bytes::Bytes;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Notify, mpsc, oneshot},
    task::{JoinHandle, JoinSet},
};
use tokio_rustls::TlsAcceptor;
type Connection = h2::server::Connection<tokio_rustls::server::TlsStream<TcpStream>, Bytes>;

pub struct Peer {
    pub endpoint: Endpoint,
    pub requests: mpsc::Receiver<Stream>,
    pub accepts: Arc<AtomicUsize>,
    stop: oneshot::Sender<()>,
    task: JoinHandle<io::Result<()>>,
}

pub struct Stream {
    respond: h2::server::SendResponse<Bytes>,
    close: Arc<Notify>,
}

pub struct Body {
    sender: h2::SendStream<Bytes>,
    pub close: Arc<Notify>,
}

impl Peer {
    pub async fn start() -> Result<Self, TestError> {
        let endpoint = Endpoint::bind(true, b"h2").await?;
        let acceptor = endpoint
            .acceptor
            .clone()
            .ok_or("missing TLS fixture configuration")?;
        let listener = duplicate_listener(&endpoint)?;
        let (incoming, requests) = mpsc::channel(16);
        let (stop, stopped) = oneshot::channel();
        let accepts = Arc::new(AtomicUsize::new(0));
        let task = tokio::spawn(run(
            listener,
            acceptor,
            incoming,
            Arc::clone(&accepts),
            stopped,
        ));
        Ok(Self {
            endpoint,
            requests,
            accepts,
            stop,
            task,
        })
    }

    pub async fn stop(self) -> TestResult {
        self.stop
            .send(())
            .map_err(|()| io::Error::other("h2 peer exited early"))?;
        self.task.await??;
        self.endpoint.cleanup()?;
        Ok(())
    }
}

fn duplicate_listener(endpoint: &Endpoint) -> io::Result<TcpListener> {
    // The standard listener's safe clone gives the driver its own descriptor.
    let descriptor = std::os::fd::AsFd::as_fd(&endpoint.listener).try_clone_to_owned()?;
    let listener = std::net::TcpListener::from(descriptor);
    listener.set_nonblocking(true)?;
    TcpListener::from_std(listener)
}

impl Stream {
    pub fn headers(mut self, length: usize) -> Result<Body, TestError> {
        let response = http::Response::builder()
            .header("content-length", length)
            .body(())?;
        let sender = self.respond.send_response(response, false)?;
        Ok(Body {
            sender,
            close: self.close,
        })
    }
}

impl Body {
    pub fn data(&mut self, bytes: Bytes, end: bool) -> Result<(), h2::Error> {
        self.sender.send_data(bytes, end)
    }
}

async fn run(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    incoming: mpsc::Sender<Stream>,
    accepts: Arc<AtomicUsize>,
    mut stopped: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut sessions = JoinSet::new();
    for _ in 0..32 {
        tokio::select! {
            result = listener.accept() => {
                let (socket, _) = result?;
                accepts.fetch_add(1, Ordering::SeqCst);
                sessions.spawn(session(socket, acceptor.clone(), incoming.clone()));
            }
            _ = &mut stopped => break,
        }
    }
    super::drain(&mut sessions).await
}

async fn session(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    incoming: mpsc::Sender<Stream>,
) -> io::Result<()> {
    socket.set_nodelay(true)?;
    let tls = acceptor.accept(socket).await?;
    let connection = h2::server::handshake(tls).await.map_err(io::Error::other)?;
    drive(connection, incoming).await
}

async fn drive(mut connection: Connection, incoming: mpsc::Sender<Stream>) -> io::Result<()> {
    let close = Arc::new(Notify::new());
    for _ in 0..64 {
        let event = tokio::select! {
            () = close.notified() => break,
            event = connection.accept() => event,
        };
        let Some(event) = event else {
            break;
        };
        match event {
            Ok((_request, respond)) => offer(&incoming, respond, &close).await?,
            Err(error) => return peer_close(error),
        }
    }
    Ok(())
}

async fn offer(
    incoming: &mpsc::Sender<Stream>,
    respond: h2::server::SendResponse<Bytes>,
    close: &Arc<Notify>,
) -> io::Result<()> {
    incoming
        .send(Stream {
            respond,
            close: Arc::clone(close),
        })
        .await
        .map_err(|_| io::Error::other("h2 controller dropped"))
}

fn peer_close(error: h2::Error) -> io::Result<()> {
    if error.is_io() || (error.is_go_away() && error.reason() == Some(h2::Reason::NO_ERROR)) {
        return Ok(());
    }
    Err(io::Error::other(error))
}
