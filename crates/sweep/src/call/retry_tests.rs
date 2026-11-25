use super::*;
use crate::{consumer_fixture as fixture, credentials::TestCredentials};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::RegionId, record::Value};
use inventory_engine::limiter::ExecutorConfig;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const PAGE: &str = r#"{"DeliveryStreamNames":["alpha"],"HasMoreDeliveryStreams":false}"#;

struct RetryFixture {
    client: Client,
    executor: ApiExecutor,
    credentials: TestCredentials,
    metrics: crate::metrics::Accumulators,
    encodes: AtomicUsize,
}

impl RetryFixture {
    fn new() -> TestResult<Self> {
        Ok(Self {
            client: Client::new(aws_wire::transport::Config {
                allow_plain_http: true,
                ..Default::default()
            })?,
            executor: ApiExecutor::new(ExecutorConfig {
                max_attempts: 2,
                max_backoff_ms: 1,
                requests_per_second: 1000,
                ..Default::default()
            })?,
            credentials: TestCredentials::new("us-east-1"),
            metrics: Default::default(),
            encodes: AtomicUsize::new(0),
        })
    }

    async fn call(
        &self,
        plan: &CallablePlan<'_>,
        url: &str,
        limit: inventory_core::record::FieldId,
        now: impl Fn() -> SystemTime,
    ) -> Result<DecodedResponse, SweepError> {
        let region = RegionId::parse("us-east-1")?;
        let scope = inventory_core::inventory::Region::Regional(region.clone());
        let endpoint = plan.endpoint(&region, &scope)?;
        let base = Record::encode(&Value::Map(BTreeMap::new()))?;
        let fields = [crate::input::InputField {
            path: vec![limit].into_boxed_slice(),
            value: Value::Int(37),
        }];
        Call {
            client: &self.client,
            executor: &self.executor,
            credentials: &self.credentials,
            plan,
            endpoint: &endpoint,
            url,
            base: &base,
            fields: &fields,
            metrics: &self.metrics,
            account: "123456789012",
            scope: &scope,
        }
        .run_with_clock(now, || {
            self.encodes.fetch_add(1, Ordering::Relaxed);
            request::encode(plan, url, &base, &fields)
        })
        .await
    }
}

#[tokio::test]
async fn given_retryable_response_when_page_is_retried_then_signed_bodies_are_byte_identical()
-> TestResult {
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("firehose", "ListDeliveryStreams")
        .ok_or("plan absent")?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}", listener.local_addr()?);
    let fixture = RetryFixture::new()?;
    let limit = compiled.fields.id("Limit").ok_or("limit field absent")?;
    let advanced = AtomicBool::new(false);
    let first = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_704_067_200))
        .ok_or("clock overflow")?;
    let second = first
        .checked_add(Duration::from_secs(600))
        .ok_or("clock overflow")?;
    let now = || {
        if advanced.swap(true, Ordering::Relaxed) {
            second
        } else {
            first
        }
    };
    let (response, bodies) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(fixture.call(plan, &url, limit, now), serve(listener))
    })
    .await?;
    let response = response?;
    let requests = bodies?;
    assert_retry_requests(&requests)?;
    assert_eq!(fixture.executor.stats().attempts, 2);
    assert_eq!(fixture.encodes.load(Ordering::Relaxed), 1);
    let decoded =
        crate::record::policy_dfgk_tests::fixture::decode_as_sweep(plan, PAGE.as_bytes())?;
    assert_eq!(response.record.as_bytes(), decoded.as_bytes());
    Ok(())
}

fn assert_retry_requests(requests: &[SignedRequest]) -> TestResult {
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.as_slice())
            .collect::<Vec<_>>(),
        vec![b"{\"Limit\":37}".as_slice(); 2]
    );
    assert_eq!(
        requests
            .iter()
            .map(|request| request.date.as_str())
            .collect::<Vec<_>>(),
        vec!["20240101T000000Z", "20240101T001000Z"]
    );
    assert_ne!(
        requests
            .first()
            .ok_or("first request absent")?
            .authorization,
        requests.last().ok_or("retry request absent")?.authorization
    );
    Ok(())
}

#[tokio::test]
async fn given_invalid_request_when_executed_then_permanent_error_is_accounted() -> TestResult {
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("firehose", "ListDeliveryStreams")
        .ok_or("plan absent")?;
    let fixture = RetryFixture::new()?;
    let limit = compiled.fields.id("Limit").ok_or("limit field absent")?;
    let error = fixture
        .call(plan, "http://[", limit, SystemTime::now)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "API executor error: permanent API failure: AWS wire request error: invalid request endpoint"
    );
    let stats = fixture.executor.stats();
    assert_eq!((stats.attempts, stats.failures, stats.cancelled), (1, 1, 0));
    Ok(())
}

#[tokio::test]
async fn given_cancelled_executor_when_request_cannot_encode_then_cancellation_wins() -> TestResult
{
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("firehose", "ListDeliveryStreams")
        .ok_or("plan absent")?;
    let fixture = RetryFixture::new()?;
    fixture.executor.cancel();
    let limit = compiled.fields.id("Limit").ok_or("limit field absent")?;
    let error = fixture
        .call(plan, "http://[", limit, SystemTime::now)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "API executor error: API execution cancelled"
    );
    let stats = fixture.executor.stats();
    assert_eq!((stats.attempts, stats.failures, stats.cancelled), (0, 0, 1));
    Ok(())
}

struct SignedRequest {
    body: Vec<u8>,
    date: String,
    authorization: String,
}

impl SignedRequest {
    fn parse(headers: &str, body: Vec<u8>) -> TestResult<Self> {
        let header = |name| {
            headers
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.trim().to_owned())
                .ok_or("signing header absent")
        };
        let authorization = header("authorization")?;
        let signed = authorization
            .split("SignedHeaders=")
            .nth(1)
            .and_then(|value| value.split(',').next())
            .ok_or("signed headers absent")?;
        assert!(signed.split(';').any(|name| name == "x-amz-date"));
        Ok(Self {
            body,
            date: header("x-amz-date")?,
            authorization,
        })
    }
}

async fn serve(listener: TcpListener) -> TestResult<Vec<SignedRequest>> {
    let mut bodies = Vec::new();
    for attempt in 0..2 {
        let (mut socket, _) = listener.accept().await?;
        let (headers, body) = read_request(&mut socket).await?;
        assert_eq!(headers.lines().next(), Some("POST / HTTP/1.1"));
        assert_eq!(
            headers
                .lines()
                .filter(|line| line.to_ascii_lowercase().starts_with("authorization:"))
                .count(),
            1
        );
        bodies.push(SignedRequest::parse(&headers, body)?);
        let (status, body) = if attempt == 0 {
            ("500 Internal Server Error", "not-json")
        } else {
            ("200 OK", PAGE)
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/x-amz-json-1.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await?;
        socket.shutdown().await?;
    }
    Ok(bodies)
}

async fn read_request(socket: &mut TcpStream) -> TestResult<(String, Vec<u8>)> {
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
        let length = headers
            .lines()
            .find_map(|line| {
                line.split_once(':')
                    .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                    .map(|(_, value)| value.trim().parse::<usize>())
            })
            .ok_or("content length absent")??;
        if body.len() == length {
            return Ok((headers.to_owned(), body.to_vec()));
        }
    }
    Err("request exceeds fixed read budget".into())
}
