use std::{io, path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        self,
        pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Socket = tokio_rustls::server::TlsStream<TcpStream>;

const ITEMS: &str = r#"{"Items":[{"Id":"one"},{"Id":"two"}],"NextToken":"do-not-fetch"}"#;
const UNSUPPORTED: &str = r#"{"__type":"UnsupportedOperation","message":"not served"}"#;

pub struct Server {
    listener: TcpListener,
    acceptor: TlsAcceptor,
    pub ca: PathBuf,
    _work: tempfile::TempDir,
}

impl Server {
    pub async fn new() -> TestResult<Self> {
        let work = tempfile::tempdir()?;
        let ca = work.path().join("ca.pem");
        let certificate = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()])?;
        tokio::fs::write(&ca, certificate.cert.pem()).await?;
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            certificate.signing_key.serialize_der(),
        ));
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(vec![certificate.cert.der().clone()], key)?;
        Ok(Self {
            listener: TcpListener::bind("127.0.0.1:0").await?,
            acceptor: TlsAcceptor::from(Arc::new(config)),
            ca,
            _work: work,
        })
    }

    pub fn endpoint(&self) -> io::Result<String> {
        Ok(format!("https://{}", self.listener.local_addr()?))
    }

    pub async fn serve(&self, count: usize) -> TestResult<Vec<String>> {
        if count > 32 {
            return Err(io::Error::other("probe test request bound exceeded").into());
        }
        let mut requests = Vec::new();
        requests.try_reserve_exact(count)?;
        for _ in 0..count {
            let (socket, _) = self.listener.accept().await?;
            let mut socket = self.acceptor.accept(socket).await?;
            let request = read_request(&mut socket).await?;
            reply(&mut socket, &request).await?;
            requests.push(request);
        }
        Ok(requests)
    }

    pub async fn serve_until(
        &self,
        bodies: &[&str],
        mut completed: tokio::sync::oneshot::Receiver<()>,
    ) -> TestResult<Vec<String>> {
        let mut requests = Vec::new();
        for _ in 0..=32 {
            let (socket, _) = tokio::select! {
                _ = &mut completed => return Ok(requests),
                socket = self.listener.accept() => socket?,
            };
            let body = bodies
                .get(requests.len())
                .ok_or("unexpected comparison request")?;
            let mut socket = self.acceptor.accept(socket).await?;
            requests.push(read_request(&mut socket).await?);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-amz-json-1.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len(),
            );
            socket.write_all(response.as_bytes()).await?;
            socket.shutdown().await?;
        }
        Err("comparison test request bound exceeded".into())
    }
}

async fn read_request(socket: &mut Socket) -> io::Result<String> {
    let mut head = Vec::new();
    for _ in 0..32_768 {
        head.push(socket.read_u8().await?);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    if !head.ends_with(b"\r\n\r\n") {
        return Err(io::Error::other("probe test request head exceeds bound"));
    }
    let length = content_length(std::str::from_utf8(&head).map_err(io::Error::other)?)?;
    let mut body = vec![0; length];
    socket.read_exact(&mut body).await?;
    head.extend(body);
    String::from_utf8(head).map_err(io::Error::other)
}

fn content_length(head: &str) -> io::Result<usize> {
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()
        .map_err(io::Error::other)?;
    match length {
        Some(length) if length <= 65_536 => Ok(length),
        _ => Err(io::Error::other(
            "probe test body length missing or oversized",
        )),
    }
}

fn response(request: &str) -> TestResult<(u16, &'static str)> {
    let target = request
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("x-amz-target"))
        .map(|(_, value)| value.trim())
        .ok_or("probe target missing")?;
    match target.rsplit('.').next() {
        Some("ListParents0") => Ok((200, ITEMS)),
        Some("ListParents1") => Ok((200, r#"{"Items":[]}"#)),
        Some("ListParents2") => Ok((400, UNSUPPORTED)),
        Some("ListParents3") => Ok((200, "malformed JSON")),
        _ => Err("unexpected probe operation".into()),
    }
}

async fn reply(socket: &mut Socket, request: &str) -> TestResult {
    let (status, body) = response(request)?;
    let response = format!(
        "HTTP/1.1 {status} Response\r\nContent-Type: application/x-amz-json-1.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await?;
    Ok(())
}
