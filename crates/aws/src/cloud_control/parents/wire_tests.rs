use std::time::Duration;

use aws_config::{BehaviorVersion, retry::RetryConfig};
use aws_credential_types::Credentials;
use aws_types::region::Region;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use super::*;

type TestError = Box<dyn std::error::Error>;
type TestResult<T = ()> = Result<T, TestError>;
const EXPECTED_REQUESTS: usize = 10;
const MAX_HTTP_BYTES: usize = 65_536;

#[derive(Debug)]
struct Request {
    target: String,
    body: Value,
}

async fn read_request(stream: &mut TcpStream) -> TestResult<Request> {
    let mut bytes = Vec::new();
    for _ in 0..256 {
        let mut chunk = [0_u8; 4_096];
        let count = stream.read(&mut chunk).await?;
        if count == 0 {
            return Err("HTTP request ended before its content length".into());
        }
        bytes.extend_from_slice(chunk.get(..count).ok_or("invalid read length")?);
        if bytes.len() > MAX_HTTP_BYTES {
            return Err("HTTP fixture request exceeds budget".into());
        }
        let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let body_start = end.checked_add(4).ok_or("header offset overflow")?;
        let headers = std::str::from_utf8(bytes.get(..end).ok_or("missing header bytes")?)?;
        let fields: BTreeMap<_, _> = headers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_owned()))
            .collect();
        let length: usize = fields
            .get("content-length")
            .ok_or("missing Content-Length")?
            .parse()?;
        let finish = body_start
            .checked_add(length)
            .ok_or("content length overflow")?;
        if finish > MAX_HTTP_BYTES {
            return Err("HTTP fixture content length exceeds budget".into());
        }
        if bytes.len() >= finish {
            return Ok(Request {
                target: fields
                    .get("x-amz-target")
                    .ok_or("missing X-Amz-Target")?
                    .clone(),
                body: serde_json::from_slice(
                    bytes.get(body_start..finish).ok_or("missing JSON body")?,
                )?,
            });
        }
    }
    Err("HTTP fixture read count exceeded".into())
}

fn text<'a>(body: &'a Value, key: &str) -> TestResult<&'a str> {
    body.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("request lacks string {key}").into())
}

fn resource(identifier: &str, properties: Value) -> Value {
    json!({"Identifier":identifier,"Properties":properties.to_string()})
}

fn respond(request: &Request, served: &mut BTreeSet<String>) -> TestResult<Value> {
    let type_name = text(&request.body, "TypeName")?;
    if request.target.ends_with(".ListResources") {
        if type_name == "AWS::EKS::Cluster" {
            if request.body.get("ResourceModel").is_some()
                || request.body.get("NextToken").is_some()
            {
                return Err("parent list unexpectedly carries a model or page token".into());
            }
            return Ok(
                json!({"TypeName":type_name,"ResourceDescriptions":[resource("alpha",json!({})),resource("beta",json!({}))]}),
            );
        }
        if type_name != "AWS::EKS::Addon" {
            return Err("unexpected child type".into());
        }
        let model: Value = serde_json::from_str(text(&request.body, "ResourceModel")?)?;
        let parent = text(&model, "ClusterName")?;
        if model.as_object().map(|model| model.len()) != Some(1) || !served.contains(parent) {
            return Err("child model was not derived from a completed parent read".into());
        }
        match request.body.get("NextToken").and_then(Value::as_str) {
            None => Ok(
                json!({"TypeName":type_name,"ResourceDescriptions":[],"NextToken":format!("next-{parent}")}),
            ),
            Some(token) if token == format!("next-{parent}") => Ok(json!({"TypeName":type_name,
                "ResourceDescriptions":[resource(&format!("{parent}|coredns"),json!({"ClusterName":parent}))]})),
            _ => Err("child pagination token leaked between parent scopes".into()),
        }
    } else if request.target.ends_with(".GetResource") {
        if request.body.get("ResourceModel").is_some() {
            return Err("get unexpectedly carries ResourceModel".into());
        }
        let id = text(&request.body, "Identifier")?;
        let properties = if type_name == "AWS::EKS::Cluster" && ["alpha", "beta"].contains(&id) {
            served.insert(id.to_owned());
            json!({"Name":id,"Arn":format!("arn:aws:eks:us-east-1:111111111111:cluster/{id}")})
        } else if type_name == "AWS::EKS::Addon" {
            let (parent, name) = id.split_once('|').ok_or("child identifier lacks parent")?;
            if !served.contains(parent) || name != "coredns" {
                return Err("unrecognized child identifier".into());
            }
            json!({"ClusterName":parent,"AddonName":name,
                "Arn":format!("arn:aws:eks:us-east-1:111111111111:addon/{parent}/{name}/01234567")})
        } else {
            return Err("unexpected read request".into());
        };
        Ok(json!({"TypeName":type_name,"ResourceDescription":resource(id,properties)}))
    } else {
        Err("unexpected CloudControl operation".into())
    }
}

async fn serve(listener: TcpListener) -> TestResult<Vec<Request>> {
    let mut requests = Vec::new();
    let mut served = BTreeSet::new();
    for ordinal in 0..EXPECTED_REQUESTS {
        let (mut stream, _) = listener.accept().await?;
        let request = read_request(&mut stream).await?;
        let (status, body) = if ordinal == 0 {
            if !request.target.ends_with(".ListResources")
                || text(&request.body, "TypeName")? != "AWS::EKS::Cluster"
            {
                return Err("first request is not root discovery".into());
            }
            (
                "429 Too Many Requests",
                json!({"__type":"ThrottlingException","Message":"fixture throttle"}),
            )
        } else {
            ("200 OK", respond(&request, &mut served)?)
        };
        let body = body.to_string();
        let response = format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/x-amz-json-1.0\r\ncontent-length: {}\r\nconnection: close\r\nx-amzn-requestid: inventory-parent-wire\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await?;
        stream.shutdown().await?;
        requests.push(request);
    }
    Ok(requests)
}

#[tokio::test]
async fn production_sdk_serializes_parent_models_keeps_cursors_isolated_and_executor_owns_retry()
-> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::new(
            "AKIDEXAMPLE",
            "fixture-only-secret",
            None,
            None,
            "parent-wire-fixture",
        ))
        .endpoint_url(endpoint)
        .retry_config(RetryConfig::standard().with_max_attempts(1))
        .load()
        .await;
    let executor = Arc::new(ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 1_000,
            max_backoff_ms: 10,
            attempt_timeout_ms: 5_000,
            ..ExecutorConfig::default()
        },
        6,
    )?);
    let (sender, mut receiver) = mpsc::channel(128);
    let ctx = Context {
        config,
        executor: Arc::clone(&executor),
        sender,
        account: "111111111111".into(),
        partition: "aws".into(),
        region: "us-east-1".into(),
        max_pages: 3,
        page_size: 100,
    };
    let selection = Selection::Types(vec![inventory_core::ResourceType::new("AWS::EKS::Addon")]);
    let types = validate_selection(&selection)?;
    let work = async {
        tokio::try_join!(
            async {
                super::super::collect(&ctx, &types, &selection)
                    .await
                    .map_err(|error| -> TestError { Box::new(error) })
            },
            serve(listener)
        )
    };
    let ((), requests) = tokio::time::timeout(Duration::from_secs(20), work)
        .await
        .map_err(|_| "parent SDK wire fixture exceeded its 20-second deadline")??;
    assert_eq!(requests.len(), EXPECTED_REQUESTS);
    let stats = executor.stats();
    assert_eq!(stats.attempts, 10);
    assert_eq!(stats.retries, 1);
    assert_eq!(stats.throttles, 1);
    assert_eq!(stats.successes, 9);
    assert_eq!(stats.failures, 0);
    assert_eq!(stats.in_flight, 0);
    let mut resources = Vec::new();
    let mut edges = Vec::new();
    for _ in 0..128 {
        match receiver.try_recv() {
            Ok(CollectionEvent::Page(page)) => {
                resources.extend(page.resources);
                edges.extend(page.edges);
            }
            Ok(CollectionEvent::Failure(failure)) => {
                return Err(format!("unexpected collection failure {failure:?}").into());
            }
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                break;
            }
        }
    }
    assert_eq!(resources.len(), 4);
    let parents: BTreeSet<_> = edges
        .iter()
        .filter(|edge| edge.relation == "CHILD_OF")
        .map(|edge| edge.to_id.as_str())
        .collect();
    assert_eq!(
        parents,
        BTreeSet::from([
            "arn:aws:eks:us-east-1:111111111111:cluster/alpha",
            "arn:aws:eks:us-east-1:111111111111:cluster/beta"
        ])
    );
    Ok(())
}
