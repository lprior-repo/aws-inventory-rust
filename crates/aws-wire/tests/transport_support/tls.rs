use super::{TestError, TestResult};
use bytes::Bytes;
use http_body_util::Full;
use hyper::{body::Incoming, service::service_fn};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use std::{
    convert::Infallible,
    io,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    net::TcpListener,
    sync::{Notify, oneshot},
    task::{JoinHandle, JoinSet},
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

pub struct TlsServer {
    pub url: String,
    pub ca: PathBuf,
    pub accepts: Arc<AtomicUsize>,
    pub rejected: Arc<AtomicUsize>,
    pub stalled: Arc<Notify>,
    stop: oneshot::Sender<()>,
    task: JoinHandle<io::Result<()>>,
}

#[derive(Default)]
struct Events {
    accepts: Arc<AtomicUsize>,
    rejected: Arc<AtomicUsize>,
    stalled: Arc<Notify>,
}

impl TlsServer {
    pub async fn start(alpn: &[u8]) -> Result<Self, TestError> {
        Self::start_named(alpn, "127.0.0.1").await
    }

    pub async fn start_named(alpn: &[u8], name: &str) -> Result<Self, TestError> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let ca = ca_path(listener.local_addr()?.port());
        let acceptor = credentials_named(&ca, alpn, name)?;
        Ok(Self::spawn(listener, acceptor, ca)?)
    }

    fn spawn(listener: TcpListener, acceptor: TlsAcceptor, ca: PathBuf) -> io::Result<Self> {
        let url = format!("https://{}/", listener.local_addr()?);
        let (stop, stopped) = oneshot::channel();
        let events = Arc::<Events>::default();
        let task = tokio::spawn(run(listener, acceptor, Arc::clone(&events), stopped));
        Ok(Self {
            url,
            ca,
            accepts: Arc::clone(&events.accepts),
            rejected: Arc::clone(&events.rejected),
            stalled: Arc::clone(&events.stalled),
            stop,
            task,
        })
    }

    pub async fn stop(self) -> TestResult {
        self.stop
            .send(())
            .map_err(|()| io::Error::other("TLS server exited early"))?;
        self.task.await??;
        std::fs::remove_file(self.ca)?;
        Ok(())
    }
}

pub(super) fn credentials(path: &Path, alpn: &[u8]) -> Result<TlsAcceptor, TestError> {
    credentials_named(path, alpn, "127.0.0.1")
}

fn credentials_named(path: &Path, alpn: &[u8], name: &str) -> Result<TlsAcceptor, TestError> {
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec![name.to_owned()])?;
    let mut file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(cert.pem().as_bytes())?;
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(signing_key.serialize_der()));
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], key)?;
    config.alpn_protocols = vec![alpn.to_vec()];
    Ok(TlsAcceptor::from(Arc::new(config)))
}

async fn run(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    events: Arc<Events>,
    mut stopped: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut sessions = JoinSet::new();
    for _ in 0..16 {
        tokio::select! {
            result = listener.accept() => {
                let (socket, _) = result?;
                events.accepts.fetch_add(1, Ordering::SeqCst);
                sessions.spawn(handshake(socket, acceptor.clone(), Arc::clone(&events)));
            }
            _ = &mut stopped => break,
        }
    }
    super::drain(&mut sessions).await
}

async fn handshake(
    socket: tokio::net::TcpStream,
    acceptor: TlsAcceptor,
    events: Arc<Events>,
) -> io::Result<()> {
    socket.set_nodelay(true)?;
    match acceptor.accept(socket).await {
        Ok(stream) => serve(stream, Arc::clone(&events.stalled)).await,
        Err(_) => {
            events.rejected.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
}

async fn serve(stream: TlsStream<tokio::net::TcpStream>, stalled: Arc<Notify>) -> io::Result<()> {
    let h2 = stream.get_ref().1.alpn_protocol() == Some(b"h2");
    let service = service_fn(move |request| reply(request, Arc::clone(&stalled)));
    if h2 {
        hyper::server::conn::http2::Builder::new(TokioExecutor::new())
            .serve_connection(TokioIo::new(stream), service)
            .await
            .map_err(io::Error::other)
    } else {
        hyper::server::conn::http1::Builder::new()
            .serve_connection(TokioIo::new(stream), service)
            .await
            .map_err(io::Error::other)
    }
}

async fn reply(
    request: http::Request<Incoming>,
    stalled: Arc<Notify>,
) -> Result<http::Response<Full<Bytes>>, Infallible> {
    if request.uri().path() == "/stall" {
        stalled.notify_one();
        std::future::pending::<()>().await;
    }
    Ok(http::Response::new(Full::new(Bytes::from_static(b"tls"))))
}

pub(super) fn ca_path(port: u16) -> PathBuf {
    static IDENTIFIER: AtomicUsize = AtomicUsize::new(0);
    let id = IDENTIFIER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("aws-wire-{}-{port}-{id}.pem", std::process::id()))
}
