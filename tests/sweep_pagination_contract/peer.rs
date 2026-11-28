use super::*;
#[derive(Clone, Copy)]
pub(super) enum Pages {
    Advancing,
    Repeated,
    MissingRequired,
    FirstPrerequisite,
    ContinuationPrerequisite,
    Ec2,
}

pub(super) struct Gate {
    pub(super) arrived: oneshot::Sender<()>,
    pub(super) release: oneshot::Receiver<()>,
}

pub(super) struct Peer {
    pub(super) endpoint: String,
    pub(super) ca: PathBuf,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}

impl Peer {
    pub(super) async fn start(directory: &Path, pages: Pages, gate: Option<Gate>) -> Result<Self> {
        let (acceptor, ca) = tls_acceptor(directory)?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let endpoint = format!("https://{}", listener.local_addr()?);
        let (shutdown, receiver) = oneshot::channel();
        let task = tokio::spawn(serve(listener, acceptor, pages, gate, receiver));
        Ok(Self {
            endpoint,
            ca,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    pub(super) async fn stop(mut self) -> Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _closed = shutdown.send(());
        }
        let mut task = self.task.take().context("TLS peer task missing")?;
        match tokio::time::timeout(DEADLINE, &mut task).await {
            Ok(result) => result?,
            Err(error) => {
                task.abort();
                match task.await {
                    Err(join) if join.is_cancelled() => {}
                    Err(join) => return Err(join.into()),
                    Ok(result) => result?,
                }
                Err(error.into())
            }
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

#[tokio::test(start_paused = true)]
async fn timed_out_peer_shutdown_drops_the_serving_owner_before_returning() -> Result<()> {
    let (started, running) = oneshot::channel();
    let (dropped, mut released) = oneshot::channel();
    let task = tokio::spawn(async move {
        struct Owner(Option<oneshot::Sender<()>>);
        impl Drop for Owner {
            fn drop(&mut self) {
                if let Some(dropped) = self.0.take() {
                    let _receiver_closed = dropped.send(());
                }
            }
        }
        let _owner = Owner(Some(dropped));
        started
            .send(())
            .map_err(|_| anyhow::anyhow!("owner readiness receiver closed"))?;
        std::future::pending::<Result<()>>().await
    });
    running.await?;
    let peer = Peer {
        endpoint: String::new(),
        ca: PathBuf::new(),
        shutdown: None,
        task: Some(task),
    };
    let Err(error) = peer.stop().await else {
        return Err(anyhow::anyhow!(
            "the pending owner must hit the shutdown deadline"
        ));
    };
    assert!(
        error
            .downcast_ref::<tokio::time::error::Elapsed>()
            .is_some(),
        "{error:#}"
    );
    assert_eq!(released.try_recv(), Ok(()));
    Ok(())
}

fn tls_acceptor(directory: &Path) -> Result<(TlsAcceptor, PathBuf)> {
    let mut parameters = CertificateParams::new(vec!["127.0.0.1".to_owned()])?;
    parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate()?;
    let ca = parameters.self_signed(&ca_key)?;
    let issuer = rcgen::Issuer::from_params(&parameters, &ca_key);
    let key = KeyPair::generate()?;
    let leaf = CertificateParams::new(vec!["127.0.0.1".to_owned()])?.signed_by(&key, &issuer)?;
    let ca_path = directory.join("pagination-ca.pem");
    std::fs::write(&ca_path, ca.pem())?;
    let provider = Arc::new(tokio_rustls::rustls::crypto::aws_lc_rs::default_provider());
    let config = tokio_rustls::rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(
            vec![leaf.der().clone()],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der())),
        )?;
    Ok((TlsAcceptor::from(Arc::new(config)), ca_path))
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    pages: Pages,
    mut gate: Option<Gate>,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<()> {
    for _ in 0..32 {
        let (stream, _) = tokio::select! {
            _ = &mut shutdown => return Ok(()),
            result = listener.accept() => result?,
        };
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            result = tokio::time::timeout(DEADLINE, serve_connection(stream, &acceptor, pages, &mut gate)) => result??,
        }
    }
    anyhow::bail!("pagination peer exhausted its fixed 32-request budget")
}

async fn serve_connection(
    stream: TcpStream,
    acceptor: &TlsAcceptor,
    pages: Pages,
    gate: &mut Option<Gate>,
) -> Result<()> {
    let mut tls = acceptor.accept(stream).await?;
    let uri = read_request(&mut tls).await?;
    let tuple = if uri.path() == "/items" {
        query_tuple(&uri)?
    } else {
        None
    };
    if uri.path() == "/items"
        && tuple == Some(("same.example.", "A", None))
        && let Some(checkpoint) = gate.take()
    {
        checkpoint
            .arrived
            .send(())
            .map_err(|_| anyhow::anyhow!("checkpoint observer dropped"))?;
        checkpoint.release.await?;
        return Ok(());
    }
    let (status, body) = page_response(&uri, tuple, pages);
    let reply = format!(
        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\nx-amzn-requestid: pagination-contract\r\n\r\n{body}",
        body.len()
    );
    tls.write_all(reply.as_bytes()).await?;
    Ok(())
}

async fn read_request(
    stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
) -> Result<http::Uri> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    for _ in 0..32 {
        let read = stream.read(&mut buffer).await?;
        ensure!(read > 0, "incomplete HTTP request");
        bytes.extend_from_slice(buffer.get(..read).context("HTTP read length overflow")?);
        if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let headers =
                std::str::from_utf8(bytes.get(..end).context("HTTP headers out of bounds")?)?;
            ensure!(
                headers.lines().any(|line| line
                    .to_ascii_lowercase()
                    .starts_with("authorization: aws4-hmac-sha256 ")),
                "unsigned request reached pagination service"
            );
            let line = headers.lines().next().context("missing request line")?;
            let target = line
                .strip_prefix("GET ")
                .and_then(|line| line.strip_suffix(" HTTP/1.1"))
                .context("expected REST-JSON GET request")?;
            return Ok(target.parse()?);
        }
    }
    anyhow::bail!("request exceeds the fixed 32-KiB header bound")
}

type Tuple<'a> = Option<(&'a str, &'a str, Option<&'a str>)>;

fn query_tuple(uri: &http::Uri) -> Result<Tuple<'_>> {
    let Some(query) = uri.query() else {
        return Ok(None);
    };
    let entries = query
        .split('&')
        .map(|pair| pair.split_once('=').context("malformed query pair"))
        .collect::<Result<BTreeMap<_, _>>>()?;
    ensure!(
        entries.len() == 2 || entries.len() == 3,
        "unexpected pagination query fields"
    );
    let name = entries
        .get("name")
        .copied()
        .context("name cursor missing")?;
    let kind = entries
        .get("type")
        .copied()
        .context("type cursor missing")?;
    Ok(Some((name, kind, entries.get("identifier").copied())))
}

fn page_response(uri: &http::Uri, tuple: Tuple<'_>, pages: Pages) -> (&'static str, String) {
    if uri.path() == "/instances" {
        return ec2_response(uri, pages);
    }
    let body = match (uri.path(), tuple) {
        ("/identity", None) => json!({"Account": ACCOUNT}),
        ("/regions", None) => json!({"Regions": [{"RegionName": REGION}]}),
        ("/items", None) if matches!(pages, Pages::FirstPrerequisite) => {
            return sandbox_refusal();
        }
        ("/items", None) if matches!(pages, Pages::MissingRequired) => {
            json!({"Items":["must-not-commit"],"IsTruncated":true,"NextName":"same.example."})
        }
        ("/items", None) => {
            json!({"Items":["zero","one"],"IsTruncated":true,"NextName":"same.example.","NextType":"A","NextIdentifier":"blue"})
        }
        ("/items", Some(("same.example.", "A", Some("blue"))))
            if matches!(pages, Pages::ContinuationPrerequisite) =>
        {
            return sandbox_refusal();
        }
        ("/items", Some(("same.example.", "A", Some("blue"))))
            if matches!(pages, Pages::Repeated) =>
        {
            json!({"Items":["two"],"IsTruncated":true,"NextName":"same.example.","NextType":"A","NextIdentifier":"blue"})
        }
        ("/items", Some(("same.example.", "A", Some("blue")))) => {
            json!({"Items":["two"],"IsTruncated":true,"NextName":"same.example.","NextType":"A"})
        }
        ("/items", Some(("same.example.", "A", None))) => {
            json!({"Items":["three","four"],"IsTruncated":true,"NextName":"same.example.","NextType":"AAAA","NextIdentifier":"green"})
        }
        ("/items", Some(("same.example.", "AAAA", Some("green")))) => {
            json!({"Items":["five"],"IsTruncated":false})
        }
        _ => {
            return (
                "400 Bad Request",
                json!({"message":"unknown resource or invalid continuation tuple"}).to_string(),
            );
        }
    };
    ("200 OK", body.to_string())
}

fn sandbox_refusal() -> (&'static str, String) {
    (
        "400 Bad Request",
        json!({"__type":"BadRequestException","message":"Your account is still in the sandbox."})
            .to_string(),
    )
}

fn ec2_response(uri: &http::Uri, pages: Pages) -> (&'static str, String) {
    if !matches!(pages, Pages::Ec2) {
        return (
            "400 Bad Request",
            json!({"message":"unexpected EC2 request"}).to_string(),
        );
    }
    let query = uri.query().map_or("", |query| query);
    let fields = query
        .split('&')
        .filter_map(|entry| entry.split_once('='))
        .collect::<BTreeMap<_, _>>();
    if fields.get("MaxResults") != Some(&"1000") {
        return (
            "400 Bad Request",
            json!({"message":"MaxResults was not 1000"}).to_string(),
        );
    }
    let body = match fields.get("NextToken").copied() {
        None => json!({"Reservations":[
            {"OwnerId":ACCOUNT,"Instances":[
                {"InstanceId":"i-owned-one","Tags":[{"Key":"Name","Value":"owned-one"}]},
                {"InstanceId":"i-owned-two","Tags":[{"Key":"Name","Value":"owned-two"}]}
            ]},
            {"OwnerId":"999999999999","Instances":[
                {"InstanceId":"i-foreign","Tags":[{"Key":"Name","Value":"foreign"}]}
            ]},
            {"OwnerId":ACCOUNT,"Instances":[]}
        ],"NextToken":"ec2-next"}),
        Some("ec2-next") => json!({"Reservations":[
            {"OwnerId":ACCOUNT,"Instances":[
                {"InstanceId":"i-owned-three","Tags":[{"Key":"Name","Value":"owned-three"}]}
            ]}
        ]}),
        _ => {
            return (
                "400 Bad Request",
                json!({"message":"invalid EC2 continuation token"}).to_string(),
            );
        }
    };
    ("200 OK", body.to_string())
}
