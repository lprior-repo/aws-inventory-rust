use std::{io, sync::Arc};
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

pub struct Server {
    listener: TcpListener,
    acceptor: TlsAcceptor,
    pub ca: std::path::PathBuf,
    _work: tempfile::TempDir,
}

impl Server {
    pub async fn new() -> TestResult<Self> {
        let work = tempfile::tempdir()?;
        let ca = work.path().join("ca.pem");
        let certificate = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()])?;
        std::fs::write(&ca, certificate.cert.pem())?;
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            certificate.signing_key.serialize_der(),
        ));
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
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

    pub async fn serve(
        &self,
        replies: &[(u16, &str)],
        mut done: tokio::sync::oneshot::Receiver<()>,
    ) -> io::Result<Vec<String>> {
        let mut requests = Vec::new();
        for (status, body) in replies {
            let (socket, _) = tokio::select! {
                biased;
                accepted = self.listener.accept() => accepted?,
                _ = &mut done => return Err(io::Error::other(
                    "Cloud Control fixture ended before expected request",
                )),
            };
            let mut socket = self.acceptor.accept(socket).await?;
            requests.push(request(&mut socket).await?);
            let response = format!(
                "HTTP/1.1 {status} Response\r\nContent-Type: application/x-amz-json-1.0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await?;
            socket.shutdown().await?;
        }
        tokio::select! {
            biased;
            accepted = self.listener.accept() => {
                let _ = accepted?;
                Err(io::Error::other("unexpected Cloud Control request"))
            },
            _ = &mut done => Ok(requests),
        }
    }
}

async fn request(socket: &mut tokio_rustls::server::TlsStream<TcpStream>) -> io::Result<String> {
    let mut head = Vec::new();
    for _ in 0..32_768 {
        head.push(socket.read_u8().await?);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    if !head.ends_with(b"\r\n\r\n") {
        return Err(io::Error::other("request head exceeds bound"));
    }
    let header = std::str::from_utf8(&head).map_err(io::Error::other)?;
    let length = header
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .ok_or_else(|| io::Error::other("missing content length"))?
        .1
        .trim()
        .parse::<usize>()
        .map_err(io::Error::other)?;
    if length > 65_536 {
        return Err(io::Error::other("oversized request body"));
    }
    let mut body = vec![0; length];
    socket.read_exact(&mut body).await?;
    let mut request = head;
    request.extend(body);
    String::from_utf8(request).map_err(io::Error::other)
}
