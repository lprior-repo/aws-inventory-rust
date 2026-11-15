use anyhow::{Context, Result, ensure};
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
const MAX_REQUESTS: usize = 4096;

pub(super) struct AccessDeniedPeer {
    address: SocketAddr,
    ca_bundle: PathBuf,
    _directory: TempDir,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}

impl AccessDeniedPeer {
    pub(super) async fn start(account: &str, region: &str) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let (acceptor, ca_bundle) = tls_acceptor(directory.path())?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let (shutdown, receiver) = oneshot::channel();
        let task = tokio::spawn(serve(
            listener,
            acceptor,
            receiver,
            account.to_owned(),
            region.to_owned(),
        ));
        Ok(Self {
            address,
            ca_bundle,
            _directory: directory,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    pub(super) fn endpoint(&self) -> String {
        format!("https://{}", self.address)
    }

    pub(super) fn ca_bundle(&self) -> &Path {
        &self.ca_bundle
    }

    pub(super) async fn shutdown(mut self) -> Result<()> {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).unwrap_or(());
        }
        let mut task = self
            .task
            .take()
            .context("TLS denial peer task is missing")?;
        match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
            Ok(joined) => joined.context("TLS denial peer task failed")??,
            Err(_) => {
                task.abort();
                drop(task.await);
                anyhow::bail!("TLS denial peer did not stop");
            }
        }
        Ok(())
    }
}

impl Drop for AccessDeniedPeer {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).unwrap_or(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
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
    let ca_bundle = directory.join("test-ca.pem");
    std::fs::write(&ca_bundle, ca.pem())?;
    server_acceptor(leaf, leaf_key, ca_bundle)
}

fn server_acceptor(
    leaf: rcgen::Certificate,
    key: KeyPair,
    ca_bundle: PathBuf,
) -> Result<(TlsAcceptor, PathBuf)> {
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
    let provider = std::sync::Arc::new(tokio_rustls::rustls::crypto::aws_lc_rs::default_provider());
    let configuration = tokio_rustls::rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(vec![leaf.der().clone()], private_key)?;
    Ok((
        TlsAcceptor::from(std::sync::Arc::new(configuration)),
        ca_bundle,
    ))
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    mut shutdown: oneshot::Receiver<()>,
    account: String,
    region: String,
) -> Result<()> {
    for _ in 0..MAX_REQUESTS {
        let (stream, _) = tokio::select! {
            _ = &mut shutdown => return Ok(()),
            accepted = listener.accept() => accepted?,
        };
        serve_connection(stream, &acceptor, &account, &region).await?;
    }
    anyhow::bail!("TLS denial peer exceeded its bounded request budget")
}

async fn serve_connection(
    stream: TcpStream,
    acceptor: &TlsAcceptor,
    account: &str,
    region: &str,
) -> Result<()> {
    let mut tls = tokio::time::timeout(Duration::from_secs(5), acceptor.accept(stream)).await??;
    let body = tokio::time::timeout(Duration::from_secs(5), read_request_body(&mut tls)).await??;
    let reply = response(&body, account, region);
    tokio::time::timeout(Duration::from_secs(5), tls.write_all(reply.as_bytes())).await??;
    Ok(())
}

async fn read_request_body(
    stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    for _ in 0..512 {
        let count = stream.read(&mut buffer).await?;
        ensure!(count > 0, "TLS peer received an incomplete AWS request");
        bytes.extend_from_slice(
            buffer
                .get(..count)
                .context("request buffer range is invalid")?,
        );
        if let Some(body) = complete_body(&bytes)? {
            return Ok(body);
        }
    }
    anyhow::bail!("AWS request exceeded its bounded read budget")
}

fn complete_body(bytes: &[u8]) -> Result<Option<Vec<u8>>> {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Ok(None);
    };
    let headers = std::str::from_utf8(
        bytes
            .get(..header_end)
            .context("HTTP header range is invalid")?,
    )?;
    let length = headers
        .lines()
        .find_map(content_length)
        .context("AWS request omitted Content-Length")?;
    ensure!(
        length <= MAX_REQUEST_BYTES,
        "AWS request exceeds the configured body limit"
    );
    let start = header_end
        .checked_add(4)
        .context("HTTP header length overflow")?;
    ensure!(
        headers.lines().any(is_sigv4_authorization),
        "controlled denial peer requires a signed SigV4 AWS request"
    );
    let end = start
        .checked_add(length)
        .context("HTTP body length overflow")?;
    Ok(bytes.get(start..end).map(|body| body.to_vec()))
}

fn content_length(line: &str) -> Option<usize> {
    let (name, value) = line.split_once(':')?;
    name.eq_ignore_ascii_case("content-length")
        .then(|| value.trim().parse().ok())
        .flatten()
}

fn is_sigv4_authorization(line: &str) -> bool {
    let Some((name, value)) = line.split_once(':') else {
        return false;
    };
    name.eq_ignore_ascii_case("authorization")
        && value
            .trim_start()
            .starts_with("AWS4-HMAC-SHA256 Credential=")
        && value.contains("SignedHeaders=")
        && value.contains("Signature=")
}

fn response(request: &[u8], account: &str, region: &str) -> String {
    let (status, body) = protocol_response(request, account, region);
    let reason = if status == 200 { "OK" } else { "Forbidden" };
    let error_header = if status == 200 {
        ""
    } else {
        "x-amzn-errortype: UnauthorizedOperation;Sender\r\n"
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: text/xml\r\ncontent-length: {}\r\nconnection: close\r\nx-amzn-requestid: e2e-controlled\r\n{error_header}\r\n{body}",
        body.len()
    )
}

fn protocol_response(request: &[u8], account: &str, region: &str) -> (u16, String) {
    if contains_action(request, "GetCallerIdentity") {
        (200, caller_identity(account))
    } else if contains_action(request, "DescribeRegions") {
        (200, regions(region))
    } else {
        (403, access_denied())
    }
}

fn contains_action(request: &[u8], action: &str) -> bool {
    let marker = format!("Action={action}");
    request
        .windows(marker.len())
        .any(|window| window == marker.as_bytes())
}

fn caller_identity(account: &str) -> String {
    format!(
        "<GetCallerIdentityResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\"><GetCallerIdentityResult><Arn>arn:aws:iam::{account}:user/e2e</Arn><UserId>e2e</UserId><Account>{account}</Account></GetCallerIdentityResult><ResponseMetadata><RequestId>e2e</RequestId></ResponseMetadata></GetCallerIdentityResponse>"
    )
}

fn regions(region: &str) -> String {
    format!(
        "<DescribeRegionsResponse xmlns=\"http://ec2.amazonaws.com/doc/2016-11-15/\"><requestId>e2e</requestId><regionInfo><item><regionName>{region}</regionName><endpoint>ec2.{region}.amazonaws.com</endpoint><optInStatus>opt-in-not-required</optInStatus></item></regionInfo></DescribeRegionsResponse>"
    )
}

fn access_denied() -> String {
    "<Response><Errors><Error><Code>UnauthorizedOperation</Code><Message>Denied by isolated TLS contract peer.</Message></Error></Errors><RequestID>e2e</RequestID></Response>".to_owned()
}
