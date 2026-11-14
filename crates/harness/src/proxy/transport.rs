use super::{Config, MAX_CA_BYTES, MAX_CERTIFICATES, MAX_HOST_BYTES, ProxyError, deadline};
use rustls_pki_types::{CertificateDer, ServerName, pem::PemObject};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufReader, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector, client::TlsStream, rustls};

struct Authority {
    host: String,
    port: u16,
}
enum Scheme {
    Http,
    Https,
}
enum Upstream {
    Http(Authority),
    Https {
        authority: Authority,
        server_name: ServerName<'static>,
        tls: TlsConnector,
    },
}
pub(crate) struct Target {
    upstream: Upstream,
}
impl Target {
    pub(crate) async fn new(config: &Config) -> Result<Self, ProxyError> {
        Ok(Self {
            upstream: upstream(config).await?,
        })
    }
    fn authority(&self) -> &Authority {
        match &self.upstream {
            Upstream::Http(authority) | Upstream::Https { authority, .. } => authority,
        }
    }
    pub(crate) async fn connect(&self) -> Result<BufReader<Socket>, ProxyError> {
        let authority = self.authority();
        let tcp = deadline(TcpStream::connect((
            authority.host.as_str(),
            authority.port,
        )))
        .await?;
        tcp.set_nodelay(true)?;
        let socket = match &self.upstream {
            Upstream::Http(_) => Socket::Plain(tcp),
            Upstream::Https {
                server_name, tls, ..
            } => Socket::Tls(Box::new(
                deadline(tls.connect(server_name.clone(), tcp)).await?,
            )),
        };
        Ok(BufReader::new(socket))
    }
}

async fn upstream(config: &Config) -> Result<Upstream, ProxyError> {
    let (authority, scheme) = authority(&config.upstream)?;
    match (&config.ca_bundle, scheme) {
        (None, Scheme::Http) => Ok(Upstream::Http(authority)),
        (Some(path), Scheme::Https) => {
            let server_name = ServerName::try_from(authority.host.clone())
                .map_err(|_| ProxyError::Configuration("TLS server name"))?;
            let tls = load_tls(path).await?;
            Ok(Upstream::Https {
                authority,
                server_name,
                tls,
            })
        }
        _ => Err(ProxyError::Configuration(
            "HTTPS requires CA; HTTP forbids CA",
        )),
    }
}

fn authority(endpoint: &str) -> Result<(Authority, Scheme), ProxyError> {
    let (rest, scheme) = upstream_scheme(endpoint)?;
    let (host, port) = rest
        .rsplit_once(':')
        .ok_or(ProxyError::Configuration("upstream port"))?;
    let host = host_name(host)?;
    let port = port
        .parse::<u16>()
        .map_err(|_| ProxyError::Configuration("upstream port"))?;
    if port == 0 {
        return Err(ProxyError::Configuration("zero upstream port"));
    }
    Ok((
        Authority {
            host: host.to_owned(),
            port,
        },
        scheme,
    ))
}

async fn load_tls(path: &std::path::Path) -> Result<TlsConnector, ProxyError> {
    let pem = read_ca_bundle(path).await?;
    let roots = trusted_roots(&pem)?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| ProxyError::Configuration("TLS versions"))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}

fn upstream_scheme(endpoint: &str) -> Result<(&str, Scheme), ProxyError> {
    if let Some(rest) = endpoint.strip_prefix("https://") {
        Ok((rest, Scheme::Https))
    } else if let Some(rest) = endpoint.strip_prefix("http://") {
        Ok((rest, Scheme::Http))
    } else {
        Err(ProxyError::Configuration("upstream scheme"))
    }
}

fn host_name(host: &str) -> Result<&str, ProxyError> {
    let host = match host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
        Some(unbracketed) => unbracketed,
        None => host,
    };
    if host.is_empty()
        || host.len() > MAX_HOST_BYTES
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-:".contains(&b))
    {
        return Err(ProxyError::Configuration("upstream host"));
    }
    Ok(host)
}

async fn read_ca_bundle(path: &std::path::Path) -> Result<Vec<u8>, ProxyError> {
    let file = tokio::fs::File::open(path).await?;
    let mut pem = Vec::new();
    super::reserve(
        &mut pem,
        usize::try_from(MAX_CA_BYTES.saturating_add(1)).map_err(|_| ProxyError::Allocation)?,
    )?;
    file.take(MAX_CA_BYTES.saturating_add(1))
        .read_to_end(&mut pem)
        .await?;
    if u64::try_from(pem.len()).map_err(|_| ProxyError::Allocation)? > MAX_CA_BYTES {
        return Err(ProxyError::Limit("CA bundle"));
    }
    Ok(pem)
}

fn trusted_roots(pem: &[u8]) -> Result<rustls::RootCertStore, ProxyError> {
    let mut roots = rustls::RootCertStore::empty();
    CertificateDer::pem_slice_iter(pem)
        .take(MAX_CERTIFICATES.saturating_add(1))
        .enumerate()
        .try_for_each(|(index, cert)| {
            if index == MAX_CERTIFICATES {
                return Err(ProxyError::Limit("CA certificates"));
            }
            roots
                .add(cert.map_err(|_| ProxyError::Configuration("CA PEM"))?)
                .map_err(|_| ProxyError::Configuration("CA certificate"))
        })?;
    if roots.is_empty() {
        return Err(ProxyError::Configuration("empty CA bundle"));
    }
    Ok(roots)
}

pub(crate) enum Socket {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}
impl AsyncRead for Socket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        }
    }
}
impl AsyncWrite for Socket {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Plain(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(s) => Pin::new(s).poll_flush(cx),
            Self::Tls(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}
