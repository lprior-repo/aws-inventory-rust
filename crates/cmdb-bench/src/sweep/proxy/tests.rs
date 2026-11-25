use super::*;
use rustls_pki_types::{CertificateDer, ServerName, pem::PemObject};
use std::{
    error::Error,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{
    TlsConnector,
    rustls::{ClientConfig, RootCertStore},
};
type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
const RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\ncountedbody";
struct Upstream {
    endpoint: String,
    ca: PathBuf,
    count: Arc<AtomicU64>,
    cancel: CancellationToken,
    owner: Option<JoinHandle<Result<(), String>>>,
}
impl Upstream {
    async fn start(work: &Path) -> TestResult<Self> {
        tokio::fs::create_dir(work).await?;
        let acceptor = transport::certificate(work).await?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("https://{}", listener.local_addr()?);
        let count = Arc::new(AtomicU64::new(0));
        let cancel = CancellationToken::new();
        let owner = tokio::spawn(upstream(listener, acceptor, count.clone(), cancel.clone()));
        Ok(Self {
            endpoint,
            ca: work.join("proxy-ca.pem"),
            count,
            cancel,
            owner: Some(owner),
        })
    }
    async fn finish(mut self) -> TestResult {
        self.cancel.cancel();
        self.owner.take().ok_or("upstream owner missing")?.await??;
        Ok(())
    }
}
impl Drop for Upstream {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(owner) = &self.owner {
            owner.abort();
        }
    }
}
async fn upstream(
    listener: TcpListener,
    acceptor: tokio_rustls::TlsAcceptor,
    count: Arc<AtomicU64>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let incoming = stream::try_unfold((listener, cancel), |(listener, cancel)| async move {
        tokio::select! {
            () = cancel.cancelled() => Ok(None),
            accepted = listener.accept() => {
                let (socket, _) = accepted.map_err(text)?;
                Ok::<_, String>(Some((socket, (listener, cancel))))
            }
        }
    });
    incoming
        .take(64)
        .map_ok(|socket| respond(socket, acceptor.clone(), count.clone()))
        .try_buffer_unordered(8)
        .try_collect::<()>()
        .await
}
async fn respond(
    socket: TcpStream,
    acceptor: tokio_rustls::TlsAcceptor,
    count: Arc<AtomicU64>,
) -> Result<(), String> {
    let mut socket = acceptor.accept(socket).await.map_err(text)?;
    let mut bytes = vec![0; REQUEST.len()];
    socket.read_exact(&mut bytes).await.map_err(text)?;
    if bytes != REQUEST {
        return Err("upstream request was changed".into());
    }
    count.fetch_add(1, Ordering::Relaxed);
    socket.write_all(RESPONSE).await.map_err(text)?;
    socket.shutdown().await.map_err(text)
}
async fn within<T, E>(future: impl Future<Output = Result<T, E>>) -> TestResult<T>
where
    E: Into<Box<dyn Error>>,
{
    tokio::select! {
        result = future => result.map_err(Into::into),
        () = stream::iter(0..1_000_000).for_each(|_| async { tokio::task::yield_now().await }) => Err("bounded I/O poll budget exhausted".into()),
    }
}
async fn exchange(front: &Front) -> TestResult<Vec<u8>> {
    let pem = tokio::fs::read(front.ca_bundle()).await?;
    let certificate = CertificateDer::from_pem_slice(&pem)?;
    let mut roots = RootCertStore::empty();
    roots.add(certificate)?;
    let mut config = ClientConfig::builder_with_provider(Arc::new(
        tokio_rustls::rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_root_certificates(roots)
    .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let connector = TlsConnector::from(Arc::new(config));
    let address = front
        .endpoint()
        .strip_prefix("https://")
        .ok_or("HTTPS endpoint missing")?;
    let tcp = TcpStream::connect(address).await?;
    let mut socket = connector
        .connect(ServerName::try_from("127.0.0.1")?, tcp)
        .await?;
    socket.write_all(REQUEST).await?;
    let mut bytes = Vec::new();
    (&mut socket).take(65_536).read_to_end(&mut bytes).await?;
    socket.shutdown().await?;
    Ok(bytes)
}
#[tokio::test(start_paused = true)]
async fn bad_tls_connection_does_not_stop_later_round_trip_or_shutdown() -> TestResult {
    let directory = tempfile::tempdir()?;
    assert_eq!(super::seed_header(), "0xb5045020");
    let upstream = within(Upstream::start(&directory.path().join("upstream"))).await?;
    let front = within(Front::start(
        &upstream.endpoint,
        &upstream.ca,
        ProxySpec {
            latency_ms: 0,
            rps: 2,
        },
        directory.path(),
    ))
    .await?;
    let result = within(async {
        let address = front
            .endpoint()
            .strip_prefix("https://")
            .ok_or("endpoint")?;
        let mut bad = TcpStream::connect(address).await?;
        bad.write_all(b"not a TLS handshake").await?;
        bad.shutdown().await?;
        let body = exchange(&front).await?;
        assert_eq!(body, RESPONSE);
        assert_eq!(upstream.count.load(Ordering::Relaxed), 1);
        Ok::<_, Box<dyn Error>>(())
    })
    .await;
    let drained = within(front.shutdown()).await;
    let stopped = within(upstream.finish()).await;
    result?;
    assert_eq!(drained?.connection_failures, 1);
    stopped?;
    Ok(())
}
#[tokio::test(start_paused = true)]
async fn two_per_second_bucket_throttles_and_recording_observes_responses() -> TestResult {
    let directory = tempfile::tempdir()?;
    let upstream = within(Upstream::start(&directory.path().join("upstream"))).await?;
    let front = within(Front::start(
        &upstream.endpoint,
        &upstream.ca,
        ProxySpec {
            latency_ms: 0,
            rps: 2,
        },
        directory.path(),
    ))
    .await?;
    let proxy = front.proxy.as_ref().ok_or("proxy missing")?;
    let cancel = CancellationToken::new();
    let started = Instant::now();
    let traffic = async {
        assert_eq!(within(exchange(&front)).await?, RESPONSE);
        assert_eq!(within(exchange(&front)).await?, RESPONSE);
        let throttled = within(exchange(&front)).await?;
        assert!(String::from_utf8(throttled)?.contains("ThrottlingException"));
        assert_eq!(started.elapsed(), Duration::ZERO);
        assert_eq!(upstream.count.load(Ordering::Relaxed), 2);
        tokio::time::advance(Duration::from_millis(500)).await;
        assert_eq!(within(exchange(&front)).await?, RESPONSE);
        let snapshot = proxy.snapshot()?;
        assert_eq!(snapshot.counters.rate_limited, 1);
        assert_eq!(upstream.count.load(Ordering::Relaxed), 3);
        Ok::<_, Box<dyn Error>>(())
    };
    let collect = async {
        let result = traffic.await;
        cancel.cancel();
        result
    };
    let (result, samples) = tokio::join!(collect, record(proxy, cancel.clone()));
    let samples = samples?;
    let drained = within(front.shutdown()).await;
    let stopped = within(upstream.finish()).await;
    result?;
    drained?;
    stopped?;
    assert_eq!(samples.first(), Some(&(0, 0, 0)));
    assert_eq!(samples.last(), Some(&(500, 4, 1)));
    Ok(())
}
#[tokio::test(start_paused = true)]
async fn consecutive_collections_each_start_with_a_full_bucket() -> TestResult {
    let directory = tempfile::tempdir()?;
    let upstream = within(Upstream::start(&directory.path().join("upstream"))).await?;
    let spec = ProxySpec {
        latency_ms: 0,
        rps: 2,
    };
    let check = async |front: &Front| {
        assert_eq!(within(exchange(front)).await.map_err(text)?, RESPONSE);
        assert_eq!(within(exchange(front)).await.map_err(text)?, RESPONSE);
        assert!(
            String::from_utf8(within(exchange(front)).await.map_err(text)?)
                .map_err(text)?
                .contains("ThrottlingException")
        );
        Ok::<_, String>(())
    };
    let first = within(super::collect(
        &upstream.endpoint,
        &upstream.ca,
        spec,
        &directory.path().join("first"),
        check,
    ))
    .await;
    let second = within(super::collect(
        &upstream.endpoint,
        &upstream.ca,
        spec,
        &directory.path().join("second"),
        check,
    ))
    .await;
    let count = upstream.count.load(Ordering::Relaxed);
    let stopped = within(upstream.finish()).await;
    first?;
    second?;
    stopped?;
    assert_eq!(count, 4);
    Ok(())
}
