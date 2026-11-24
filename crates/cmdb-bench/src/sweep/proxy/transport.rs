//! Bounded TLS-to-HTTP pipes; preserves signed request bytes unchanged.
use futures_util::{StreamExt, TryStreamExt, stream};
use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use tokio_util::sync::CancellationToken;
#[derive(Debug, Default)]
pub(crate) struct Report {
    pub connection_failures: u64,
}
pub(super) async fn certificate(work: &Path) -> Result<TlsAcceptor, String> {
    let (config, pem) = tokio::task::spawn_blocking(|| {
        let certificate =
            rcgen::generate_simple_self_signed(vec!["127.0.0.1".into()]).map_err(text)?;
        let key =
            rustls_pki_types::PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
        let mut config = ServerConfig::builder_with_provider(Arc::new(
            tokio_rustls::rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(text)?
        .with_no_client_auth()
        .with_single_cert(vec![certificate.cert.der().clone()], key.into())
        .map_err(text)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok::<_, String>((config, certificate.cert.pem()))
    })
    .await
    .map_err(text)??;
    tokio::fs::write(work.join("proxy-ca.pem"), pem)
        .await
        .map_err(text)?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
pub(super) async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    upstream: SocketAddr,
    cancel: CancellationToken,
) -> Result<Report, String> {
    let incoming = stream::try_unfold(
        (listener, cancel.clone()),
        |(listener, cancel)| async move {
            tokio::select! {
                biased;
                () = cancel.cancelled() => Ok(None),
                accepted = listener.accept() => {
                    let (socket, _) = accepted.map_err(text)?;
                    Ok::<_, String>(Some((socket, (listener, cancel))))
                }
            }
        },
    );
    incoming
        .take(1_000_000_000)
        .map_ok(|socket| pipe(socket, acceptor.clone(), upstream, cancel.clone()))
        .try_buffer_unordered(inventory_harness::proxy::MAX_CONNECTIONS)
        .try_fold(Report::default(), |mut report, failed| async move {
            report.connection_failures = report
                .connection_failures
                .checked_add(u64::from(failed))
                .ok_or("connection failure counter overflow")?;
            Ok(report)
        })
        .await
}
async fn pipe(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    upstream: SocketAddr,
    cancel: CancellationToken,
) -> Result<bool, String> {
    tokio::select! {
        biased;
        () = cancel.cancelled() => Ok(false),
        result = relay(socket, acceptor, upstream) => match result {
            Ok(()) => Ok(false),
            Err(error) => {
                eprintln!("proxy TLS connection failed: {error}");
                Ok(true)
            }
        },
    }
}
async fn relay(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    upstream: SocketAddr,
) -> Result<(), std::io::Error> {
    let mut tls = timeout(Duration::from_secs(10), acceptor.accept(socket))
        .await
        .map_err(std::io::Error::other)??;
    let mut tcp = timeout(Duration::from_secs(10), TcpStream::connect(upstream))
        .await
        .map_err(std::io::Error::other)??;
    match tokio::io::copy_bidirectional(&mut tls, &mut tcp).await {
        Ok(_) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
            ) => {}
        Err(error) => return Err(error),
    }
    Ok(())
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
