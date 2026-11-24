#[path = "barrier_proxy/framing.rs"]
mod framing;
use anyhow::{Context, Result, ensure};
use aws_wire::transport::{Client, Config as TransportConfig};
use bytes::Bytes;
use framing::*;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Uri, Version, header};
use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

const MAX_REQUEST_BYTES: usize = 1_048_576;
const MAX_READ_BYTES: usize = 1_064_960;
const MAX_REQUESTS: usize = 128;

pub(super) struct ContinuationCheckpoint {
    pub(super) action: &'static str,
    pub(super) token: Vec<u8>,
}

pub(super) struct BarrierProxy {
    address: SocketAddr,
    ca_bundle: PathBuf,
    _directory: TempDir,
    checkpoint: Option<oneshot::Receiver<ContinuationCheckpoint>>,
    shutdown: Option<oneshot::Sender<()>>,
    resume: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}

struct ProxyContext {
    acceptor: TlsAcceptor,
    client: Client,
    upstream: String,
    shutdown: oneshot::Receiver<()>,
    resume: oneshot::Receiver<()>,
    latency: Duration,
    checkpoint: Option<oneshot::Sender<ContinuationCheckpoint>>,
}

impl BarrierProxy {
    pub(super) async fn start(
        upstream: &str,
        upstream_ca: &Path,
        latency: Duration,
    ) -> Result<Self> {
        validate_upstream(upstream)?;
        let directory = tempfile::tempdir()?;
        let (acceptor, ca_bundle) = tls_acceptor(directory.path())?;
        let client = upstream_client(upstream_ca)?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let (shutdown, shutdown_rx) = oneshot::channel();
        let (checkpoint_tx, checkpoint_rx) = oneshot::channel();
        let (resume, resume_rx) = oneshot::channel();
        let context = ProxyContext {
            acceptor,
            client,
            upstream: upstream.to_owned(),
            shutdown: shutdown_rx,
            resume: resume_rx,
            latency,
            checkpoint: Some(checkpoint_tx),
        };
        let task = tokio::spawn(serve(listener, context));
        Ok(Self {
            address,
            ca_bundle,
            _directory: directory,
            checkpoint: Some(checkpoint_rx),
            shutdown: Some(shutdown),
            resume: Some(resume),
            task: Some(task),
        })
    }

    pub(super) fn endpoint(&self) -> String {
        format!("https://{}", self.address)
    }

    pub(super) fn ca_bundle(&self) -> &Path {
        &self.ca_bundle
    }

    pub(super) async fn wait_for_continuation(&mut self) -> Result<ContinuationCheckpoint> {
        let receiver = self
            .checkpoint
            .take()
            .context("continuation checkpoint receiver is missing")?;
        tokio::time::timeout(Duration::from_secs(60), receiver)
            .await
            .context("emulator continuation request was not observed")?
            .context("checkpoint proxy stopped before a continuation request")
    }

    pub(super) fn release(&mut self) -> Result<()> {
        if let Some(sender) = self.resume.take() {
            sender
                .send(())
                .map_err(|()| anyhow::anyhow!("checkpoint proxy stopped before release"))?;
        }
        Ok(())
    }

    pub(super) async fn shutdown(&mut self) -> Result<()> {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).unwrap_or(());
        }
        let mut task = self
            .task
            .take()
            .context("checkpoint proxy task is missing")?;
        match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
            Ok(joined) => joined.context("checkpoint proxy task failed")??,
            Err(_) => {
                task.abort();
                drop(task.await);
                anyhow::bail!("checkpoint proxy did not stop within its deadline");
            }
        }
        Ok(())
    }
}

impl Drop for BarrierProxy {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).unwrap_or(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

fn validate_upstream(endpoint: &str) -> Result<()> {
    let uri: Uri = endpoint.parse().context("parsing emulator upstream URI")?;
    ensure!(
        uri.scheme_str() == Some("https") && uri.host() == Some("127.0.0.1"),
        "checkpoint proxy upstream must be HTTPS on literal IPv4 loopback"
    );
    ensure!(
        uri.port_u16().is_some_and(|port| port > 0) && uri.path() == "/" && uri.query().is_none(),
        "checkpoint proxy upstream must be a bare loopback endpoint"
    );
    Ok(())
}

fn upstream_client(ca_bundle: &Path) -> Result<Client> {
    let config = TransportConfig {
        extra_ca_bundle: Some(ca_bundle.to_owned()),
        allow_plain_http: false,
        ..TransportConfig::default()
    };
    Ok(Client::new(config)?)
}

fn tls_acceptor(directory: &Path) -> Result<(TlsAcceptor, PathBuf)> {
    let mut ca_parameters = CertificateParams::new(vec!["127.0.0.1".to_owned()])?;
    ca_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate()?;
    let ca = ca_parameters.self_signed(&ca_key)?;
    let issuer = rcgen::Issuer::from_params(&ca_parameters, &ca_key);
    let leaf_parameters = CertificateParams::new(vec!["127.0.0.1".to_owned()])?;
    let leaf_key = KeyPair::generate()?;
    let leaf = leaf_parameters.signed_by(&leaf_key, &issuer)?;
    let ca_bundle = directory.join("proxy-ca.pem");
    std::fs::write(&ca_bundle, ca.pem())?;
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der()));
    let provider = std::sync::Arc::new(tokio_rustls::rustls::crypto::aws_lc_rs::default_provider());
    let config = tokio_rustls::rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(vec![leaf.der().clone()], private_key)?;
    Ok((TlsAcceptor::from(std::sync::Arc::new(config)), ca_bundle))
}

async fn serve(listener: TcpListener, mut context: ProxyContext) -> Result<()> {
    for _ in 0..MAX_REQUESTS {
        let (stream, _) = tokio::select! { _ = &mut context.shutdown => return Ok(()), accepted = listener.accept() => accepted? };
        if serve_connection(stream, &mut context).await? {
            return Ok(());
        }
    }
    anyhow::bail!("checkpoint proxy exceeded its bounded request budget")
}

async fn serve_connection(stream: TcpStream, context: &mut ProxyContext) -> Result<bool> {
    let mut tls =
        tokio::time::timeout(Duration::from_secs(5), context.acceptor.accept(stream)).await??;
    let (request, checkpoint) = tokio::time::timeout(
        Duration::from_secs(5),
        read_request(&mut tls, &context.upstream),
    )
    .await??;
    if let Some(checkpoint) = checkpoint.filter(|_| context.checkpoint.is_some())
        && !hold_checkpoint(context, checkpoint).await?
    {
        return Ok(true);
    }
    tokio::time::sleep(context.latency).await;
    let response =
        tokio::time::timeout(Duration::from_secs(30), context.client.send(request)).await??;
    write_response(&mut tls, &response).await?;
    Ok(false)
}

async fn hold_checkpoint(
    context: &mut ProxyContext,
    checkpoint: ContinuationCheckpoint,
) -> Result<bool> {
    let sender = context
        .checkpoint
        .take()
        .context("checkpoint request was already observed")?;
    sender
        .send(checkpoint)
        .map_err(|_| anyhow::anyhow!("checkpoint observer stopped"))?;
    tokio::select! {
        _ = &mut context.shutdown => Ok(false),
        resumed = &mut context.resume => {
            resumed.context("checkpoint release sender stopped")?;
            Ok(true)
        },
        _ = tokio::time::sleep(Duration::from_secs(60)) => anyhow::bail!("checkpoint barrier exceeded its deadline"),
    }
}

async fn read_request(
    stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
    upstream: &str,
) -> Result<(Request<Bytes>, Option<ContinuationCheckpoint>)> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    for _ in 0..512 {
        let count = stream.read(&mut buffer).await?;
        ensure!(count > 0, "proxy received an incomplete AWS request");
        bytes.extend_from_slice(buffer.get(..count).context("proxy read range is invalid")?);
        ensure!(
            bytes.len() <= MAX_READ_BYTES,
            "proxy request exceeds its bounded read limit"
        );
        if let Some(parsed) = parse_request(&bytes, upstream)? {
            return Ok(parsed);
        }
    }
    anyhow::bail!("proxy request exceeded its bounded read budget")
}

fn parse_request(
    bytes: &[u8],
    upstream: &str,
) -> Result<Option<(Request<Bytes>, Option<ContinuationCheckpoint>)>> {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Ok(None);
    };
    let header_bytes = bytes
        .get(..header_end)
        .context("proxy header range is invalid")?;
    let header_text = std::str::from_utf8(header_bytes)?;
    let mut lines = header_text.split("\r\n");
    let (method, target) = request_line(lines.next().context("proxy request line is missing")?)?;
    let headers = request_headers(lines)?;
    ensure!(
        headers.get(header::TRANSFER_ENCODING).is_none(),
        "proxy does not accept chunked AWS requests"
    );
    let length = content_length(&headers)?;
    ensure!(
        length <= MAX_REQUEST_BYTES,
        "proxy request body exceeds its configured limit"
    );
    let start = header_end
        .checked_add(4)
        .context("proxy header size overflow")?;
    let end = start
        .checked_add(length)
        .context("proxy request body size overflow")?;
    let Some(body) = bytes.get(start..end) else {
        return Ok(None);
    };
    let uri: Uri = format!("{upstream}{target}")
        .parse()
        .context("building loopback emulator request URI")?;
    ensure!(
        uri.host() == Some("127.0.0.1") && uri.scheme_str() == Some("https"),
        "proxy constructed a non-loopback upstream request"
    );
    let checkpoint = continuation_checkpoint(body);
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .version(Version::HTTP_11)
        .body(Bytes::copy_from_slice(body))?;
    request.headers_mut().extend(headers);
    Ok(Some((request, checkpoint)))
}
