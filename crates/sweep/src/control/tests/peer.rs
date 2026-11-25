use super::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinSet,
};
use tokio_rustls::{TlsAcceptor, rustls};
use tokio_util::sync::CancellationToken;

pub(super) struct Peer {
    pub endpoint: String,
    pub ca: PathBuf,
    stop: CancellationToken,
    tasks: JoinSet<TestResult>,
}

#[derive(Default)]
pub(super) struct Rejections {
    pub retry_requests: std::sync::atomic::AtomicUsize,
    pub requested: tokio::sync::Notify,
    pub release: tokio::sync::Notify,
}

impl Peer {
    pub async fn start(root: &Path, failures: Option<Arc<Rejections>>) -> TestResult<Self> {
        let (acceptor, ca) = certificates(root)?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("https://localhost:{}", listener.local_addr()?.port());
        let stop = CancellationToken::new();
        let mut tasks = JoinSet::new();
        tasks.spawn(serve(listener, acceptor, stop.clone(), failures));
        Ok(Self {
            endpoint,
            ca,
            stop,
            tasks,
        })
    }

    pub async fn stop(mut self) -> TestResult {
        self.stop.cancel();
        while let Some(result) = self.tasks.join_next().await {
            result??;
        }
        Ok(())
    }
}

fn certificates(root: &Path) -> TestResult<(TlsAcceptor, PathBuf)> {
    let mut parameters = rcgen::CertificateParams::new(vec!["control-fixture-ca".into()])?;
    parameters.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let authority_key = rcgen::KeyPair::generate()?;
    let authority = parameters.self_signed(&authority_key)?;
    let issuer = rcgen::Issuer::from_params(&parameters, &authority_key);
    let key = rcgen::KeyPair::generate()?;
    let leaf = rcgen::CertificateParams::new(vec!["localhost".into()])?.signed_by(&key, &issuer)?;
    let ca = root.join("control-ca.pem");
    std::fs::write(&ca, authority.pem())?;
    let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()),
    );
    let provider = rustls::crypto::aws_lc_rs::default_provider();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(provider))
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(vec![leaf.der().clone()], private_key)?;
    Ok((TlsAcceptor::from(Arc::new(config)), ca))
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    stop: CancellationToken,
    failures: Option<Arc<Rejections>>,
) -> TestResult {
    for _ in 0..32 {
        let (socket, _) = tokio::select! {
            () = stop.cancelled() => return Ok(()),
            accepted = listener.accept() => accepted?,
        };
        tokio::select! {
            () = stop.cancelled() => return Ok(()),
            result = tokio::time::timeout(Duration::from_secs(10), handle(socket, &acceptor, failures.as_deref())) => result??,
        }
    }
    Err("control peer exhausted its fixed request budget".into())
}

async fn handle(
    socket: tokio::net::TcpStream,
    acceptor: &TlsAcceptor,
    failures: Option<&Rejections>,
) -> TestResult {
    let mut socket = acceptor.accept(socket).await?;
    let request = request(&mut socket).await?;
    let (status, body) = response(&request, failures).await?;
    write(&mut socket, status, body).await
}

async fn request(socket: &mut (impl AsyncReadExt + Unpin)) -> TestResult<Vec<u8>> {
    let mut bytes = Vec::with_capacity(32_768);
    let mut buffer = [0; 2048];
    for _ in 0..16 {
        let count = socket.read(&mut buffer).await?;
        if count == 0 {
            return Err("request ended before headers".into());
        }
        bytes.extend_from_slice(buffer.get(..count).ok_or("read length exceeds buffer")?);
        if bytes.windows(4).any(|part| part == b"\r\n\r\n") {
            return Ok(bytes);
        }
    }
    Err("control peer request exceeds fixed header budget".into())
}

async fn response(
    bytes: &[u8],
    failures: Option<&Rejections>,
) -> TestResult<(&'static str, &'static str)> {
    let request = std::str::from_utf8(bytes)?;
    let line = request.lines().next().ok_or("request line absent")?;
    if !line.starts_with("GET ") || !line.ends_with(" HTTP/1.1") {
        return Err("unexpected request method/version".into());
    }
    if !request
        .lines()
        .any(|line| line.to_ascii_lowercase().starts_with("authorization:"))
    {
        return Err("request was not signed".into());
    }
    let target: http::Uri = line
        .split_whitespace()
        .nth(1)
        .ok_or("target absent")?
        .parse()?;
    if let Some(failures) = failures {
        match target.path() {
            "/regions"
                if failures
                    .retry_requests
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    == 0 =>
            {
                return Ok((
                    "500 Internal Server Error",
                    r#"{"__type":"InternalFailure","message":"retry this request"}"#,
                ));
            }
            "/items" => {
                failures.requested.notify_one();
                failures.release.notified().await;
                return Ok((
                    "403 Forbidden",
                    r#"{"__type":"AccessDenied","message":"not authorized"}"#,
                ));
            }
            _ => {}
        }
    }
    match (target.path(), target.query()) {
        ("/identity", None) => Ok(r#"{"Account":"123456789012"}"#),
        ("/regions", None) => Ok(r#"{"Regions":[{"RegionName":"us-east-1"}]}"#),
        ("/items", None) => Ok(
            r#"{"Items":[{"Arn":"item/one","SubscriptionArn":"PendingConfirmation"}],"NextToken":"next"}"#,
        ),
        ("/items", Some("marker=next")) => {
            Ok(r#"{"Items":[{"Arn":"item/two","SubscriptionArn":"Deleted"}]}"#)
        }
        _ => Err(format!("unexpected control request: {target}").into()),
    }.map(|body| ("200 OK", body))
}

async fn write(socket: &mut (impl AsyncWriteExt + Unpin), status: &str, body: &str) -> TestResult {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    socket.write_all(header.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.shutdown().await?;
    Ok(())
}
