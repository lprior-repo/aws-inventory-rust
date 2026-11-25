use super::*;
use crate::credentials::TestCredentials;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, WorkLease};
use std::{net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) struct Resources {
    home: RegionId,
    client: Client,
    executor: ApiExecutor,
    credentials: TestCredentials,
    config: SweepConfig,
    metrics: crate::metrics::Accumulators,
    resolved: resolved_endpoints::ResolvedEndpoints,
}

impl Resources {
    pub(super) fn new(home: &str) -> TestResult<Self> {
        Ok(Self {
            home: RegionId::parse(home)?,
            client: Client::new(TransportConfig {
                allow_plain_http: true,
                request_deadline: Duration::from_secs(3),
                connect_timeout: Duration::from_secs(1),
                ..TransportConfig::default()
            })?,
            executor: ApiExecutor::new(ExecutorConfig {
                max_attempts: 1,
                ..ExecutorConfig::default()
            })?,
            credentials: TestCredentials::new("us-east-1"),
            config: SweepConfig {
                region: Some("us-east-1".to_owned()),
                max_pages: 10,
                concurrency: 1,
                resume: false,
                endpoint: None,
                ca_bundle: None,
                idle_timeout: None,
                pool_idle_per_host: None,
                slice_timeout_secs: None,
                max_rps: None,
                max_page_size: None,
            },
            metrics: Default::default(),
            resolved: Default::default(),
        })
    }

    pub(super) fn bootstrap<'a, 'catalog>(
        &'a self,
        catalog: &'a RuntimeCatalog<'catalog>,
    ) -> BootstrapContext<'a, 'catalog, TestCredentials> {
        BootstrapContext {
            catalog,
            client: &self.client,
            executor: &self.executor,
            credentials: &self.credentials,
            home: &self.home,
            config: &self.config,
            metrics: &self.metrics,
            resolved_endpoints: &self.resolved,
        }
    }

    pub(super) fn runtime<'a, 'catalog>(
        &'a self,
        catalog: &'a RuntimeCatalog<'catalog>,
        store: &'a BinaryStore,
        progress: &'a crate::control::Progress,
    ) -> TestResult<RuntimeContext<'a, 'catalog, TestCredentials, BinaryStore>> {
        Ok(RuntimeContext {
            bootstrap: self.bootstrap(catalog),
            store,
            account: AccountId::parse("123456789012")?,
            lease_ms: 60_000,
            endpoints: Default::default(),
            consumers: page::consumer_index::ConsumerIndex::build(catalog)?,
            progress,
        })
    }
}

pub(super) async fn lease(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    directory: &std::path::Path,
    region: Region,
) -> TestResult<(BinaryStore, WorkLease)> {
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.join("endpoint-call.sqlite"), registry).await?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        region,
        ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,
        None,
    )?;
    let bindings = input::root_bindings(plan, Some(scope.account()), Some(scope.region()))?;
    let sweep = SweepId::new("endpoint-call-sites".to_owned())?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            bindings.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, now_ms()?, 60_000)
        .await?
        .ok_or("lease absent")?;
    Ok((store, lease))
}

pub(super) struct Captured {
    headers: String,
    body: String,
}

impl Captured {
    pub(super) fn assert_route(
        &self,
        address: SocketAddr,
        region: &str,
        signing_name: &str,
    ) -> TestResult {
        let target = self
            .headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or("request target absent")?;
        let host = header(&self.headers, "host")?;
        assert_eq!(
            format!("http://{host}{target}"),
            format!("http://{address}/{region}/")
        );
        let authorization = header(&self.headers, "authorization")?;
        let credential = authorization
            .split("Credential=")
            .nth(1)
            .and_then(|value| value.split(',').next())
            .ok_or("credential scope absent")?;
        let parts = credential.split('/').collect::<Vec<_>>();
        assert_eq!(parts.first().copied(), Some("FIXTUREKEY"));
        assert_eq!(parts.get(2).copied(), Some(region));
        assert_eq!(parts.get(3).copied(), Some(signing_name));
        assert_eq!(parts.get(4).copied(), Some("aws4_request"));
        Ok(())
    }

    pub(super) fn assert_operation(&self, service: &str, operation: &str) -> TestResult {
        let target = self.headers.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case("x-amz-target"))
                .map(|(_, value)| value.trim())
        });
        if let Some(value) = target {
            assert_eq!(value, format!("{service}.{operation}"));
        } else {
            assert!(
                self.body.contains(&format!("Action={operation}")),
                "body missing Action={operation}"
            );
        }
        Ok(())
    }
}

fn header<'a>(headers: &'a str, name: &str) -> TestResult<&'a str> {
    headers
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.trim())
        })
        .ok_or_else(|| format!("{name} header absent").into())
}

pub(super) async fn serve(listener: TcpListener, body: &str) -> TestResult<Captured> {
    let (mut socket, _) = listener.accept().await?;
    let (headers, request_body) = read_request(&mut socket).await?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-amz-json-1.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await?;
    Ok(Captured {
        headers,
        body: request_body,
    })
}

async fn read_request(socket: &mut TcpStream) -> TestResult<(String, String)> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 2048];
    for _ in 0..16 {
        let count = socket.read(&mut buffer).await?;
        if count == 0 {
            return Err("request ended before body".into());
        }
        bytes.extend_from_slice(buffer.get(..count).ok_or("read exceeds buffer")?);
        let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
            continue;
        };
        let end = end.checked_add(4).ok_or("header offset overflow")?;
        let (headers, body) = bytes
            .split_at_checked(end)
            .ok_or("header exceeds request")?;
        let headers = std::str::from_utf8(headers)?;
        let length = header(headers, "content-length")?.parse::<usize>()?;
        if body.len() == length {
            let body_str = std::str::from_utf8(body)?.to_owned();
            return Ok((headers.to_owned(), body_str));
        }
    }
    Err("request exceeds fixed read budget".into())
}
