use super::{TestResult, runtime::NativeRuntime};
use inventory_harness::proxy::{Config, Proxy};
use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::{JoinHandle, JoinSet},
};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use tokio_util::sync::CancellationToken;

const MAX_REQUEST: usize = 8_388_608;
const DENIED: &str = "<Response><Errors><Error><Code>UnauthorizedOperation</Code><Message>owned-native-denied-DescribeSubnets</Message></Error></Errors><RequestID>owned-native-denied</RequestID></Response>";

pub(super) struct DeniedRoute {
    endpoint: String,
    ca: PathBuf,
    proxy: Option<Proxy>,
    cancel: CancellationToken,
    owner: Option<JoinHandle<io::Result<()>>>,
    denied: Arc<AtomicUsize>,
}

impl DeniedRoute {
    pub(super) async fn start(native: &NativeRuntime) -> TestResult<Self> {
        let certificate = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()])?;
        let ca = native.work().join("owned-denied-ca.pem");
        tokio::fs::write(&ca, certificate.cert.pem()).await?;
        let key =
            rustls_pki_types::PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.cert.der().clone()], key.into())?;
        let mut config_proxy = Config::new(native.emulator().endpoint(), 481);
        config_proxy.ca_bundle = Some(native.emulator().ca_bundle().to_path_buf());
        let proxy = Proxy::start(config_proxy).await?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let endpoint = format!("https://{}", listener.local_addr()?);
        let cancel = CancellationToken::new();
        let denied = Arc::new(AtomicUsize::new(0));
        let owner = tokio::spawn(serve(
            listener,
            TlsAcceptor::from(Arc::new(config)),
            proxy.address(),
            cancel.clone(),
            denied.clone(),
        ));
        Ok(Self {
            endpoint,
            ca,
            proxy: Some(proxy),
            cancel,
            owner: Some(owner),
            denied,
        })
    }

    pub(super) fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub(super) fn ca(&self) -> &Path {
        &self.ca
    }

    pub(super) async fn close(mut self) -> TestResult {
        self.cancel.cancel();
        self.owner
            .take()
            .ok_or("denied route owner missing")?
            .await??;
        let report = self
            .proxy
            .take()
            .ok_or("denied route proxy missing")?
            .shutdown()
            .await?;
        assert!(
            self.denied.load(Ordering::SeqCst) > 0,
            "actual collector never hit denied DescribeSubnets route"
        );
        assert_eq!(report.panicked, 0);
        assert_eq!(report.aborted, 0);
        Ok(())
    }
}

impl Drop for DeniedRoute {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(owner) = self.owner.take() {
            owner.abort();
        }
    }
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    upstream: SocketAddr,
    cancel: CancellationToken,
    denied: Arc<AtomicUsize>,
) -> io::Result<()> {
    let mut children = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break,
            result = children.join_next(), if !children.is_empty() => check_child(result)?,
            accepted = listener.accept(), if children.len() < 64 => {
                let (socket, _) = accepted?;
                let acceptor = acceptor.clone();
                let denied = denied.clone();
                children.spawn(async move {
                    tokio::time::timeout(Duration::from_secs(30), forward(socket, acceptor, upstream, denied)).await
                        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "owned denied route request timeout"))?
                });
            }
        }
    }
    // Every admitted connection has its own deadline and is joined before the
    // upstream owner is drained; dropping the owner aborts its entire JoinSet.
    while let Some(result) = children.join_next().await {
        result.map_err(io::Error::other)??;
    }
    Ok(())
}

fn check_child(result: Option<Result<io::Result<()>, tokio::task::JoinError>>) -> io::Result<()> {
    result
        .ok_or_else(|| io::Error::other("owned route child missing"))?
        .map_err(io::Error::other)?
}

async fn forward(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    upstream: SocketAddr,
    denied: Arc<AtomicUsize>,
) -> io::Result<()> {
    let mut tls = acceptor.accept(socket).await?;
    let mut request = Vec::new();
    let header_end = loop {
        let mut chunk = [0_u8; 4096];
        let size = tls.read(&mut chunk).await?;
        if size == 0 {
            return Err(io::Error::other("collector closed before request"));
        }
        request.extend_from_slice(
            chunk
                .get(..size)
                .ok_or_else(|| io::Error::other("invalid request size"))?,
        );
        if let Some(index) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            break index + 4;
        }
        if request.len() > 32_768 {
            return Err(io::Error::other("owned route header bound exceeded"));
        }
    };
    let head = std::str::from_utf8(
        request
            .get(..header_end)
            .ok_or_else(|| io::Error::other("request header missing"))?,
    )
    .map_err(io::Error::other)?;
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map_or(Ok(0), |(_, value)| {
            value.trim().parse::<usize>().map_err(io::Error::other)
        })?;
    if length > MAX_REQUEST {
        return Err(io::Error::other("owned route body bound exceeded"));
    }
    let mut rewritten = head
        .lines()
        .filter(|line| {
            !line.is_empty()
                && !line
                    .split_once(':')
                    .is_some_and(|(name, _)| name.eq_ignore_ascii_case("connection"))
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    rewritten.push_str("\r\nConnection: close\r\n\r\n");
    while request.len() < header_end + length {
        let mut chunk = [0_u8; 4096];
        let size = tls.read(&mut chunk).await?;
        if size == 0 {
            return Err(io::Error::other("collector body truncated"));
        }
        request.extend_from_slice(
            chunk
                .get(..size)
                .ok_or_else(|| io::Error::other("invalid body size"))?,
        );
    }
    let body = request
        .get(header_end..header_end + length)
        .ok_or_else(|| io::Error::other("collector body missing"))?;
    if body
        .split(|byte| *byte == b'&')
        .any(|field| field == b"Action=DescribeSubnets")
    {
        denied.fetch_add(1, Ordering::SeqCst);
        let response = format!(
            "HTTP/1.1 403 Forbidden\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{DENIED}",
            DENIED.len()
        );
        tls.write_all(response.as_bytes()).await?;
    } else {
        let mut peer = TcpStream::connect(upstream).await?;
        peer.write_all(rewritten.as_bytes()).await?;
        peer.write_all(body).await?;
        let mut response = Vec::new();
        peer.take(u64::try_from(MAX_REQUEST + 1).map_err(io::Error::other)?)
            .read_to_end(&mut response)
            .await?;
        if response.len() > MAX_REQUEST {
            return Err(io::Error::other("owned route response bound exceeded"));
        }
        tls.write_all(&response).await?;
    }
    tls.shutdown().await
}
