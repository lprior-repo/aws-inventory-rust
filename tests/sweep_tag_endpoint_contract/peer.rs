use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, Notify, oneshot},
    task::JoinHandle,
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        self,
        pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Complete,
    MapComplete,
    MissingKey,
    MissingValue,
    MissingCompletion,
    NullEntry,
    MixedNullEntry,
    NullMap,
    MixedNullMap,
    Absent,
    Empty,
    Repeat,
    Denied,
    Gate,
}
#[derive(Default)]
struct State {
    tags: AtomicUsize,
    roots: AtomicUsize,
    arrived: AtomicBool,
    gate: Notify,
    failures: Mutex<Vec<String>>,
}
pub(super) struct Peer {
    pub endpoint: String,
    pub ca: PathBuf,
    state: Arc<State>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}
impl Peer {
    pub async fn start(root: &Path, mode: Mode) -> Result<Self> {
        let mut parameters = rcgen::CertificateParams::new(vec!["signed-tag-fixture-ca".into()])?;
        parameters.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let authority_key = rcgen::KeyPair::generate()?;
        let authority = parameters.self_signed(&authority_key)?;
        let issuer = rcgen::Issuer::from_params(&parameters, &authority_key);
        let key = rcgen::KeyPair::generate()?;
        let leaf = rcgen::CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into()])?
            .signed_by(&key, &issuer)?;
        let ca = root.join("tags-ca.pem");
        std::fs::write(&ca, authority.pem())?;
        let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der()));
        let provider = rustls::crypto::aws_lc_rs::default_provider();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(provider))
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(vec![leaf.der().clone()], private_key)?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("https://localhost:{}", listener.local_addr()?.port());
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let state = Arc::new(State::default());
        let serving = Arc::clone(&state);
        let (shutdown, mut stop) = oneshot::channel();
        let task = tokio::spawn(async move {
            let mut handlers = tokio::task::JoinSet::new();
            let mut requests: u32 = 0;
            loop {
                let socket = tokio::select! { _ = &mut stop => break, accepted = listener.accept() => accepted?.0 };
                requests = requests.saturating_add(1);
                anyhow::ensure!(
                    requests <= 32,
                    "TLS tag fixture exhausted its fixed request budget"
                );
                let acceptor = acceptor.clone();
                let state = Arc::clone(&serving);
                handlers.spawn(async move {
                    let result = tokio::time::timeout(DEADLINE, async {
                        let mut socket = acceptor.accept(socket).await?;
                        let mut bytes = Vec::new();
                        let mut buffer = [0; 2048];
                        loop {
                            let count = socket.read(&mut buffer).await?;
                            anyhow::ensure!(count != 0, "request ended before headers");
                            bytes.extend_from_slice(buffer.get(..count).ok_or_else(|| anyhow::anyhow!("read length"))?);
                            if bytes.windows(4).any(|part| part == b"\r\n\r\n") { break; }
                            anyhow::ensure!(bytes.len() < 32_768, "oversized fixture request");
                        }
                        let request = std::str::from_utf8(&bytes)?;
                        anyhow::ensure!(request.lines().next().is_some_and(|line| line.starts_with("GET ") && line.ends_with(" HTTP/1.1")), "wrong modeled request method/version");
                        let target = request.lines().next().and_then(|line| line.split_whitespace().nth(1)).ok_or_else(|| anyhow::anyhow!("request target"))?;
                        let authorization = request.lines().find_map(|line| line.split_once(':').filter(|(name, _)| name.eq_ignore_ascii_case("authorization")).map(|(_, value)| value)).ok_or_else(|| anyhow::anyhow!("unsigned request"))?;
                        anyhow::ensure!(authorization.contains("Credential=AKIATAGCONTRACT/") && authorization.contains("/us-east-1/taginventory/aws4_request"), "wrong signing identity: {authorization}");
                        let url: http::Uri = target.parse()?;
                        let mut status = "200 OK";
                        let body = match url.path() {
                            "/identity" => json!({"Account":ACCOUNT}),
                            "/regions" => json!({"Regions":[{"RegionName":REGION}]}),
                            "/others" => json!({"Items":[{"Arn":OWNER,"Tags":{"other":"kept"}}]}),
                            "/items" => { state.roots.fetch_add(1, Ordering::SeqCst); json!({"Items":[{"Arn":OWNER,"Tags":{"known":"old"}}]}) },
                            "/tags" => {
                                state.tags.fetch_add(1, Ordering::SeqCst);
                                let query = url.query().unwrap_or_default().split('&').map(|part| part.split_once('=').ok_or_else(|| anyhow::anyhow!("malformed query"))).collect::<Result<std::collections::BTreeMap<_, _>>>()?;
                                anyhow::ensure!(query.get("arn").copied() == Some("arn%3Aaws%3Ataginventory%3Aus-east-1%3A123456789012%3Aitem%2Fsame-id"), "trusted owner missing from encoded signed request");
                                anyhow::ensure!(query.len() == if query.contains_key("marker") { 2 } else { 1 }, "unexpected tag request inputs");
                                match query.get("marker").copied() {
                                    None => match mode {
                                        Mode::MissingValue => json!({"Tags":[{"Key":"owner"}],"IsTruncated":false}),
                                        Mode::MissingKey => json!({"Tags":[{"Value":"owner"}],"IsTruncated":false}),
                                        Mode::MissingCompletion => json!({"Tags":[],"NextToken":"next"}),
                                        Mode::NullEntry => json!({"Tags":[null],"IsTruncated":false}),
                                        Mode::MixedNullEntry => json!({"Tags":[{"Key":"visible","Value":"pair"},null],"IsTruncated":false}),
                                        Mode::NullMap => json!({"Tags":{"known":null},"IsTruncated":false}),
                                        Mode::MixedNullMap => json!({"Tags":{"visible":"pair","malformed":null},"IsTruncated":false}),
                                        Mode::MapComplete => json!({"Tags":{"alpha":"one"},"NextToken":"next","IsTruncated":true}),
                                        Mode::Absent => json!({"IsTruncated":false}),
                                        Mode::Empty => json!({"Tags":[],"IsTruncated":false}),
                                        _ => json!({"Tags":[{"Key":"alpha","Value":"one"}],"NextToken":"next","IsTruncated":true}),
                                    },
                                    Some("next") => {
                                        if matches!(mode, Mode::Gate) && !state.arrived.swap(true, Ordering::SeqCst) {
                                            state.gate.notify_waiters();
                                            let mut byte = [0];
                                            let _ = socket.read(&mut byte).await;
                                            return Ok(());
                                        }
                                        if matches!(mode, Mode::Repeat) { json!({"Tags":[{"Key":"beta","Value":"two"}],"NextToken":"next","IsTruncated":true}) }
                                        else if matches!(mode, Mode::MapComplete) { json!({"Tags":{"omega":""},"IsTruncated":false}) }
                                        else if matches!(mode, Mode::Denied) { status = "403 Forbidden"; json!({"__type":"AccessDeniedException","message":"not allowed"}) }
                                        else { json!({"Tags":[{"Key":"omega","Value":""}],"IsTruncated":false}) }
                                    }
                                    Some(token) => anyhow::bail!("unknown marker {token}"),
                                }
                            }
                            path => anyhow::bail!("unexpected signed request {path}"),
                        };
                        let body = serde_json::to_vec(&body)?;
                        let header = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                        socket.write_all(header.as_bytes()).await?;
                        socket.write_all(&body).await?;
                        socket.shutdown().await?;
                        Result::<()>::Ok(())
                    }).await;
                    match result {
                        Ok(Ok(())) => {},
                        Ok(Err(error)) => state.failures.lock().await.push(format!("{error:#}")),
                        Err(error) => state.failures.lock().await.push(format!("TLS request deadline: {error}")),
                    }
                });
            }
            while let Some(result) = handlers.join_next().await {
                result?;
            }
            Ok(())
        });
        Ok(Self {
            endpoint,
            ca,
            state,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }
    pub async fn wait_for_checkpoint(&self) -> Result<()> {
        tokio::time::timeout(DEADLINE, async {
            while !self.state.arrived.load(Ordering::SeqCst) {
                let notified = self.state.gate.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if !self.state.arrived.load(Ordering::SeqCst) {
                    notified.await;
                }
            }
        })
        .await?;
        Ok(())
    }
    pub fn tag_calls(&self) -> usize {
        self.state.tags.load(Ordering::SeqCst)
    }
    pub fn root_calls(&self) -> usize {
        self.state.roots.load(Ordering::SeqCst)
    }
    pub async fn stop(mut self) -> Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let mut task = self.task.take().context("TLS peer task missing")?;
        match tokio::time::timeout(DEADLINE, &mut task).await {
            Ok(result) => result??,
            Err(error) => {
                task.abort();
                let _ = task.await;
                return Err(error.into());
            }
        }
        let failures = self.state.failures.lock().await;
        anyhow::ensure!(
            failures.is_empty(),
            "TLS peer contract failures: {failures:?}"
        );
        Ok(())
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // Aborting the owner drops its JoinSet, which aborts every connection handler.
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
