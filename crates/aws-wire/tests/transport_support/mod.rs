use aws_wire::transport::{Client, Config};
use bytes::Bytes;
use http::Request;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
    task::{JoinHandle, JoinSet},
};

pub mod clock;
pub mod frames;
pub mod http2;
pub mod tls;
pub mod watchdog;
pub mod wire;

pub type TestError = Box<dyn std::error::Error + Send + Sync>;
pub type TestResult = Result<(), TestError>;

#[derive(Clone, Copy)]
pub enum Mode {
    Good,
    Close,
    LargeKnown,
    LargeChunked,
    BeforeHeaders,
    MidBody,
    MidBodyThenGood,
    Reset,
    BadProtocol,
}

#[derive(Default)]
pub struct Counts {
    pub accepts: AtomicUsize,
    pub open: AtomicUsize,
    pub peak: AtomicUsize,
}

pub struct Server {
    pub url: String,
    pub counts: Arc<Counts>,
    pub ready: mpsc::Receiver<()>,
    stop: oneshot::Sender<()>,
    task: JoinHandle<io::Result<()>>,
}

impl Server {
    pub async fn start(mode: Mode, counts: Arc<Counts>) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/", listener.local_addr()?);
        let (stop, stopped) = oneshot::channel();
        let (ready_tx, ready) = mpsc::channel(64);
        let task = tokio::spawn(run(listener, mode, Arc::clone(&counts), ready_tx, stopped));
        Ok(Self {
            url,
            counts,
            ready,
            stop,
            task,
        })
    }

    pub async fn stop(self) -> TestResult {
        self.stop
            .send(())
            .map_err(|()| io::Error::other("server exited early"))?;
        self.task.await??;
        Ok(())
    }
}

struct Open(Arc<Counts>);
impl Open {
    fn new(counts: Arc<Counts>) -> Self {
        counts.accepts.fetch_add(1, Ordering::SeqCst);
        let open = counts.open.fetch_add(1, Ordering::SeqCst).saturating_add(1);
        counts.peak.fetch_max(open, Ordering::SeqCst);
        Self(counts)
    }
}
impl Drop for Open {
    fn drop(&mut self) {
        self.0.open.fetch_sub(1, Ordering::SeqCst);
    }
}

async fn run(
    listener: TcpListener,
    mode: Mode,
    counts: Arc<Counts>,
    ready: mpsc::Sender<()>,
    mut stopped: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut sessions = JoinSet::new();
    for _ in 0..64 {
        tokio::select! {
            result = listener.accept() => {
                let (stream, _) = result?;
                let guard = Open::new(Arc::clone(&counts));
                let mode = next_mode(mode, &counts);
                sessions.spawn(session((stream, guard), mode, ready.clone()));
            }
            _ = &mut stopped => break,
        }
    }
    drain(&mut sessions).await
}

fn next_mode(mode: Mode, counts: &Counts) -> Mode {
    if matches!(mode, Mode::MidBodyThenGood) && count(&counts.accepts) > 1 {
        return Mode::Good;
    }
    mode
}

pub async fn drain(sessions: &mut JoinSet<io::Result<()>>) -> io::Result<()> {
    sessions.abort_all();
    while let Some(result) = sessions.join_next().await {
        match result {
            Ok(result) => result?,
            Err(e) if e.is_cancelled() => {}
            Err(e) => return Err(io::Error::other(e)),
        }
    }
    Ok(())
}

async fn session(
    connection: (TcpStream, Open),
    mode: Mode,
    ready: mpsc::Sender<()>,
) -> io::Result<()> {
    let (mut stream, _guard) = connection;
    for _ in 0..32 {
        if !request_head(&mut stream).await? {
            return Ok(());
        }
        if matches!(mode, Mode::Reset) {
            stream.set_zero_linger()?;
            return Ok(());
        }
        respond(&mut stream, mode, &ready).await?;
        if matches!(mode, Mode::Close) {
            return Ok(());
        }
        if !matches!(mode, Mode::Good) {
            return wait_closed(&mut stream).await;
        }
    }
    Ok(())
}

async fn respond(stream: &mut TcpStream, mode: Mode, ready: &mpsc::Sender<()>) -> io::Result<()> {
    write_response(stream, mode).await?;
    ready
        .send(())
        .await
        .map_err(|e| io::Error::other(e.to_string()))
}

async fn wait_closed(stream: &mut TcpStream) -> io::Result<()> {
    let mut byte = [0];
    match stream.read(&mut byte).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::ConnectionReset => Ok(()),
        Err(e) => Err(e),
    }
}

async fn request_head(stream: &mut TcpStream) -> io::Result<bool> {
    let mut tail = Vec::with_capacity(8192);
    for _ in 0..8192 {
        let mut byte = [0];
        if stream.read(&mut byte).await? == 0 {
            return Ok(false);
        }
        tail.extend_from_slice(&byte);
        if tail.ends_with(b"\r\n\r\n") {
            return Ok(true);
        }
    }
    Err(io::Error::other("request headers exceed test bound"))
}

async fn write_response(stream: &mut TcpStream, mode: Mode) -> io::Result<()> {
    let head = match mode {
        Mode::Good => "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok",
        Mode::Close => "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        Mode::LargeKnown => "HTTP/1.1 200 OK\r\nContent-Length: 1048576\r\n\r\n",
        Mode::LargeChunked => "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n100000\r\n",
        Mode::MidBody | Mode::MidBodyThenGood => "HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nx",
        Mode::BadProtocol => "NOT HTTP\r\n\r\n",
        Mode::BeforeHeaders | Mode::Reset => "",
    };
    stream.write_all(head.as_bytes()).await?;
    if matches!(mode, Mode::LargeKnown | Mode::LargeChunked) {
        stream.write_all(&[b'x'; 4096]).await?;
    }
    Ok(())
}

pub fn config() -> Config {
    Config {
        allow_plain_http: true,
        request_deadline: Duration::from_secs(2),
        connect_timeout: Duration::from_secs(2),
        idle_timeout: Duration::from_millis(10),
        ..Config::default()
    }
}

pub fn client() -> Result<Client, aws_wire::transport::TransportError> {
    Client::new(config())
}

pub fn request(url: &str) -> Result<Request<Bytes>, http::Error> {
    Request::builder().uri(url).body(Bytes::new())
}

pub fn count(value: &AtomicUsize) -> usize {
    value.load(Ordering::SeqCst)
}
