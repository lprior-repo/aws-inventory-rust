use std::{error::Error, io, sync::Arc};
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

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
pub const CALLER: &str = "<GetCallerIdentityResponse><GetCallerIdentityResult><Account>123456789012</Account><Arn>arn:aws:iam::123456789012:user/seed-report</Arn><UserId>seed-report</UserId></GetCallerIdentityResult></GetCallerIdentityResponse>";

pub struct Server {
    pub listener: TcpListener,
    acceptor: TlsAcceptor,
    pub ca: std::path::PathBuf,
    _work: tempfile::TempDir,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let work = tempfile::tempdir()?;
        let ca = work.path().join("ca.pem");
        let certificate = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()])?;
        std::fs::write(&ca, certificate.cert.pem())?;
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

    pub async fn serve(&self, replies: &[(u16, &str)]) -> Result<Vec<String>> {
        use futures_util::TryStreamExt;
        Ok(
            futures_util::stream::iter(replies.iter().map(Ok::<_, io::Error>))
                .try_fold(Vec::new(), |mut requests, (status, body)| async move {
                    let mut socket = self.accept().await?;
                    requests.push(read_request(&mut socket).await?);
                    reply(&mut socket, *status, body).await?;
                    Ok(requests)
                })
                .await?,
        )
    }

    async fn accept(&self) -> io::Result<tokio_rustls::server::TlsStream<TcpStream>> {
        let (socket, _) = self.listener.accept().await?;
        self.acceptor.accept(socket).await
    }
}

async fn read_request(
    socket: &mut tokio_rustls::server::TlsStream<TcpStream>,
) -> io::Result<String> {
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
        _ => Err(io::Error::other("request body length missing or oversized")),
    }
}

impl Server {
    pub async fn serve_topics(&self, count: usize) -> Result<usize> {
        use futures_util::{StreamExt, TryStreamExt};
        use std::sync::atomic::{AtomicUsize, Ordering};
        if count > 128 {
            return Err("topic fixture exceeds request bound".into());
        }
        self.serve(&[(200, CALLER)]).await?;
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        futures_util::stream::iter(0..count)
            .map(|_| self.topic(&active, &peak))
            .buffer_unordered(64)
            .try_collect::<Vec<_>>()
            .await?;
        Ok(peak.load(Ordering::SeqCst))
    }

    async fn topic(
        &self,
        active: &std::sync::atomic::AtomicUsize,
        peak: &std::sync::atomic::AtomicUsize,
    ) -> io::Result<()> {
        use std::sync::atomic::Ordering;
        let mut socket = self.accept().await?;
        let body = topic_body(&read_request(&mut socket).await?)?;
        peak.fetch_max(
            active.fetch_add(1, Ordering::SeqCst).saturating_add(1),
            Ordering::SeqCst,
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        reply(&mut socket, 200, &body).await?;
        active.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

fn topic_body(request: &str) -> io::Result<String> {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .ok_or_else(|| io::Error::other("topic request body missing"))?;
    let name = body
        .split('&')
        .find_map(|field| field.strip_prefix("Name="))
        .ok_or_else(|| io::Error::other("topic name missing"))?;
    Ok(format!(
        "<CreateTopicResponse><CreateTopicResult><TopicArn>arn:aws:sns:us-east-1:123456789012:{name}</TopicArn></CreateTopicResult></CreateTopicResponse>"
    ))
}

async fn reply(
    socket: &mut tokio_rustls::server::TlsStream<TcpStream>,
    status: u16,
    body: &str,
) -> io::Result<()> {
    let content = if body.starts_with('{') {
        "application/x-amz-json-1.0"
    } else {
        "text/xml"
    };
    let response = format!(
        "HTTP/1.1 {status} Response\r\nContent-Type: {content}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await
}
