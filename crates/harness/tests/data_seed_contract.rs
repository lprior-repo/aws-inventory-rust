use aws_wire::sign::Credentials;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{InventoryError, Region},
    record::{Key, Value},
};
use inventory_harness::inventory_seed::{
    FieldPath, ScenarioSeedOracle, SeedConfig, SeedError, SeedIdentity, SeedIdentityEvidence,
    SeedInput, SeedInputValue, SeedNestedBinding, SeedRelationship, SeedRunReport, SeedScenario,
    SeedScope, SeedStep, SeedTemplate, SeedValuePart, StepId, StepOutcome, load, run_data_scenario,
    run_report,
};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, io, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::{TlsAcceptor, rustls};

type TestResult = Result<(), Box<dyn Error>>;
const REGION: &str = "us-west-2";
const ACCOUNT: &str = "123456789012";
const DEADLOCK_GUARD: Duration = Duration::from_secs(60);
const CALLER: &str = "<GetCallerIdentityResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\"><GetCallerIdentityResult><Account>123456789012</Account><Arn>arn:aws:iam::123456789012:user/seed-contract</Arn><UserId>observed-user</UserId></GetCallerIdentityResult></GetCallerIdentityResponse>";
fn text(field: &str, value: &str) -> Result<SeedInput, SeedError> {
    Ok(SeedInput::new(
        FieldPath::new(&[field])?,
        SeedInputValue::Text(value.into()),
    ))
}

fn group() -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![
        SeedStep::new(
            StepId::new(0),
            "iam".into(),
            "CreateGroup".into(),
            "AWS::IAM::Group".into(),
            FieldPath::new(&["Group", "Arn"])?,
            vec![text("GroupName", "requested-group")?],
        )
        .with_scope(SeedScope::Account),
    ])
}

fn alias(template: &str) -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![SeedStep::new(
        StepId::new(0),
        "kms".into(),
        "CreateAlias".into(),
        "AWS::KMS::Alias".into(),
        SeedIdentity::Request {
            input_field: FieldPath::new(&["AliasName"])?,
            template: Some(template.into()),
        },
        vec![
            text("AliasName", "alias/requested-contract")?,
            text("TargetKeyId", "observed-key-271")?,
        ],
    )])
}

fn dependencies() -> Result<SeedScenario, SeedError> {
    let attributes = SeedTemplate {
        value: Value::Map(BTreeMap::from([(
            Key::Name("DisplayName".into()),
            Value::Null,
        )])),
        bindings: vec![SeedNestedBinding {
            destination: vec![SeedValuePart::Field("DisplayName".into())],
            step: StepId::new(0),
            path: FieldPath::new(&["AvailabilityZones", "1", "ZoneName"])?,
        }],
    };
    let tags = SeedTemplate {
        value: Value::List(vec![Value::Map(BTreeMap::from([
            (Key::Name("Key".into()), Value::Text("origin".into())),
            (Key::Name("Value".into()), Value::Null),
        ]))]),
        bindings: vec![SeedNestedBinding {
            destination: vec![
                SeedValuePart::Index(0),
                SeedValuePart::Field("Value".into()),
            ],
            step: StepId::new(1),
            path: FieldPath::new(&["Attributes", "Owner"])?,
        }],
    };
    SeedScenario::new(vec![
        SeedStep::auxiliary(
            StepId::new(0),
            "ec2".into(),
            "DescribeAvailabilityZones".into(),
            vec![],
        ),
        SeedStep::auxiliary(
            StepId::new(1),
            "sns".into(),
            "GetTopicAttributes".into(),
            vec![text(
                "TopicArn",
                "arn:aws:sns:us-west-2:123456789012:parent-topic",
            )?],
        ),
        SeedStep::new(
            StepId::new(2),
            "sns".into(),
            "CreateTopic".into(),
            "AWS::SNS::Topic".into(),
            FieldPath::new(&["TopicArn"])?,
            vec![
                SeedInput::new(
                    FieldPath::new(&["Name"])?,
                    SeedInputValue::PreviousOutput {
                        step: StepId::new(1),
                        path: FieldPath::new(&["Attributes", "DisplayName"])?,
                    },
                ),
                SeedInput::new(
                    FieldPath::new(&["Attributes"])?,
                    SeedInputValue::Template(attributes),
                ),
                SeedInput::new(FieldPath::new(&["Tags"])?, SeedInputValue::Template(tags)),
            ],
        ),
    ])
}

struct Fixture {
    listener: TcpListener,
    acceptor: TlsAcceptor,
    config: SeedConfig,
    directory: tempfile::TempDir,
}

impl Fixture {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let ca = directory.path().join("ca.pem");
        let rcgen::CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec![String::from("127.0.0.1")])?;
        std::fs::write(&ca, cert.pem())?;
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(signing_key.serialize_der()));
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let mut tls = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(vec![cert.der().clone()], key)?;
        tls.alpn_protocols = vec![b"http/1.1".to_vec()];
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let config = SeedConfig::new(&format!("https://{}", listener.local_addr()?), &ca, REGION)?;
        Ok(Self {
            listener,
            acceptor: TlsAcceptor::from(Arc::new(tls)),
            config,
            directory,
        })
    }
}

async fn read_request(
    socket: &mut tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> io::Result<Vec<u8>> {
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
    let text = std::str::from_utf8(&head).map_err(io::Error::other)?;
    let length = text
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()
        .map_err(io::Error::other)?;
    let length = match length {
        Some(length) => length,
        None if text.starts_with("GET ") => 0,
        None => return Err(io::Error::other("missing content-length")),
    };
    if length > 65_536 {
        return Err(io::Error::other("request body exceeds bound"));
    }
    let mut body = vec![0; length];
    socket.read_exact(&mut body).await?;
    head.extend(body);
    Ok(head)
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    replies: Vec<&str>,
    requests: &mut Vec<Vec<u8>>,
) -> io::Result<()> {
    for body in replies {
        let (socket, _) = listener.accept().await?;
        let mut socket = acceptor.accept(socket).await?;
        requests.push(read_request(&mut socket).await?);
        let content_type = if body.starts_with('{') {
            "application/x-amz-json-1.1"
        } else {
            "text/xml"
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await?;
        socket.shutdown().await?;
    }
    Ok(())
}

async fn execute(
    scenario: SeedScenario,
    replies: Vec<&str>,
) -> Result<(Result<ScenarioSeedOracle, SeedError>, Vec<Vec<u8>>), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let _ca_directory = fixture.directory;
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let credentials = Credentials::new("SEEDCONTRACT", "contract-secret", Some("contract-token"))?;
    let mut requests = Vec::with_capacity(replies.len());
    let result = tokio::time::timeout(DEADLOCK_GUARD, async {
        let seed = run_data_scenario(&catalog, fixture.config, &credentials, scenario);
        tokio::pin!(seed);
        tokio::select! {
            biased;
            served = serve(fixture.listener, fixture.acceptor, replies, &mut requests) => {
                served?;
                Ok::<_, io::Error>(seed.await)
            }
            result = &mut seed => Ok(result),
        }
    })
    .await??;
    Ok((result, requests))
}

fn parts(request: &[u8]) -> Result<(&str, &str), Box<dyn Error>> {
    std::str::from_utf8(request)?
        .split_once("\r\n\r\n")
        .ok_or_else(|| "missing HTTP delimiter".into())
}

fn hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let key = if key.len() > 64 {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    let padded: Vec<_> = key
        .into_iter()
        .chain(std::iter::repeat(0))
        .take(64)
        .collect();
    let inner: Vec<_> = padded
        .iter()
        .map(|byte| byte ^ 0x36)
        .chain(message.iter().copied())
        .collect();
    let outer: Vec<_> = padded
        .iter()
        .map(|byte| byte ^ 0x5c)
        .chain(Sha256::digest(inner).iter().copied())
        .collect();
    Sha256::digest(outer).to_vec()
}

fn signed(request: &[u8], service: &str) -> TestResult {
    let (head, body) = parts(request)?;
    let headers: BTreeMap<_, _> = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim()))
        .collect();
    let auth = headers
        .get("authorization")
        .ok_or("missing authorization")?;
    let date = headers.get("x-amz-date").ok_or("missing signing date")?;
    let day = date.get(..8).ok_or("short signing date")?;
    let credential =
        format!("AWS4-HMAC-SHA256 Credential=SEEDCONTRACT/{day}/{REGION}/{service}/aws4_request, ");
    assert!(auth.strip_prefix(&credential).is_some(), "{auth}");
    let signature = auth.split("Signature=").nth(1).ok_or("missing signature")?;
    let names = auth
        .split("SignedHeaders=")
        .nth(1)
        .and_then(|value| value.split_once(", "))
        .map(|(names, _)| names)
        .ok_or("missing signed header names")?;
    let canonical_headers = names
        .split(';')
        .map(|name| {
            headers
                .get(name)
                .map(|value| format!("{name}:{value}\n"))
                .ok_or("signed header missing")
        })
        .collect::<Result<String, _>>()?;
    assert_eq!(head.lines().next(), Some("POST / HTTP/1.1"));
    let payload = inventory_harness::provenance::sha256_hex(body.as_bytes());
    assert_eq!(headers.get("x-amz-security-token"), Some(&"contract-token"));
    assert_eq!(
        headers.get("x-amz-content-sha256").copied(),
        Some(payload.as_str())
    );
    let canonical = format!("POST\n/\n\n{canonical_headers}\n{names}\n{payload}");
    let scope = format!("{day}/{REGION}/{service}/aws4_request");
    let to_sign = format!(
        "AWS4-HMAC-SHA256\n{date}\n{scope}\n{}",
        inventory_harness::provenance::sha256_hex(canonical.as_bytes())
    );
    let key = hmac(b"AWS4contract-secret", day.as_bytes());
    let key = hmac(&key, REGION.as_bytes());
    let key = hmac(&key, service.as_bytes());
    let key = hmac(&key, b"aws4_request");
    let expected = hmac(&key, to_sign.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(signature, expected);
    Ok(())
}

fn decode_form(value: &str) -> Result<String, Box<dyn Error>> {
    let mut decoded = Vec::new();
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        decoded.push(match byte {
            b'+' => b' ',
            b'%' => {
                let pair = [
                    bytes.next().ok_or("short escape")?,
                    bytes.next().ok_or("short escape")?,
                ];
                u8::from_str_radix(std::str::from_utf8(&pair)?, 16)?
            }
            byte => byte,
        });
    }
    Ok(String::from_utf8(decoded)?)
}

fn form(request: &[u8]) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    parts(request)?
        .1
        .split('&')
        .map(|pair| {
            let (key, value) = pair.split_once('=').ok_or("missing form value")?;
            Ok((decode_form(key)?, decode_form(value)?))
        })
        .collect()
}

const FIRST_TOPIC: &str = "<CreateTopicResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\"><CreateTopicResult><TopicArn>arn:aws:sns:us-west-2:123456789012:first</TopicArn></CreateTopicResult></CreateTopicResponse>";
const LATER_TOPIC: &str = "<CreateTopicResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\"><CreateTopicResult><TopicArn>arn:aws:sns:us-west-2:123456789012:later</TopicArn></CreateTopicResult></CreateTopicResponse>";
const TOPIC_ATTRIBUTES: &str = "<GetTopicAttributesResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\"><GetTopicAttributesResult><Attributes><entry><key>Owner</key><value>123456789012</value></entry></Attributes></GetTopicAttributesResult></GetTopicAttributesResponse>";

fn topic_step(id: u16, name: &str) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id),
        "sns".into(),
        "CreateTopic".into(),
        "AWS::SNS::Topic".into(),
        FieldPath::new(&["TopicArn"])?,
        vec![text("Name", name)?],
    ))
}

async fn run_report_with_topic_binding(
    dependency: StepId,
    replies: Vec<&str>,
) -> Result<(SeedRunReport, Vec<Vec<u8>>), Box<dyn Error>> {
    let scenario = SeedScenario::new(vec![
        topic_step(0, "first")?,
        topic_step(1, "duplicate")?,
        topic_step(2, "later")?,
        SeedStep::auxiliary(
            StepId::new(3),
            "sns".into(),
            "GetTopicAttributes".into(),
            vec![SeedInput::new(
                FieldPath::new(&["TopicArn"])?,
                SeedInputValue::PreviousOutput {
                    step: dependency,
                    path: FieldPath::new(&["TopicArn"])?,
                },
            )],
        ),
    ])?;
    let fixture = Fixture::new().await?;
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let credentials = Credentials::new("SEEDCONTRACT", "contract-secret", Some("contract-token"))?;
    let mut requests = Vec::with_capacity(replies.len());
    let (report, served) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::join!(
            run_report(&catalog, &fixture.config, &credentials, &scenario),
            serve(fixture.listener, fixture.acceptor, replies, &mut requests)
        )
    })
    .await?;
    served?;
    Ok((report?, requests))
}

#[tokio::test]
async fn given_rejection_before_accepted_output_when_bound_then_the_later_value_is_sent()
-> TestResult {
    let replies = vec![
        CALLER,
        FIRST_TOPIC,
        FIRST_TOPIC,
        LATER_TOPIC,
        TOPIC_ATTRIBUTES,
    ];
    let (report, requests) = run_report_with_topic_binding(StepId::new(2), replies).await?;
    assert_eq!(
        report.outcomes,
        [
            StepOutcome::Accepted,
            StepOutcome::Rejected {
                code: "SeedError".into(),
                message: "invalid emulator seed configuration: seed operation returned a duplicate resource identity".into(),
            },
            StepOutcome::Accepted,
            StepOutcome::Accepted,
        ]
    );
    assert_eq!(requests.len(), 5);
    let request = requests
        .get(4)
        .ok_or("missing bound GetTopicAttributes request")?;
    let fields = form(request)?;
    assert_eq!(
        fields.get("Action").map(String::as_str),
        Some("GetTopicAttributes")
    );
    assert_eq!(
        fields.get("TopicArn").map(String::as_str),
        Some("arn:aws:sns:us-west-2:123456789012:later")
    );
    Ok(())
}

#[tokio::test]
async fn given_rejected_output_when_bound_then_dependency_is_named_and_no_request_is_sent()
-> TestResult {
    let rejected = StepId::new(1);
    let replies = vec![CALLER, FIRST_TOPIC, FIRST_TOPIC, LATER_TOPIC];
    let (report, requests) = run_report_with_topic_binding(rejected, replies).await?;
    assert_eq!(
        report.outcomes,
        [
            StepOutcome::Accepted,
            StepOutcome::Rejected {
                code: "SeedError".into(),
                message: "invalid emulator seed configuration: seed operation returned a duplicate resource identity".into(),
            },
            StepOutcome::Accepted,
            StepOutcome::Skipped { dependency: rejected },
        ]
    );
    assert_eq!(requests.len(), 4);
    Ok(())
}

#[tokio::test]
async fn given_named_map_and_list_outputs_when_executed_then_nested_requests_use_observed_values()
-> TestResult {
    let replies = vec![
        CALLER,
        "<DescribeAvailabilityZonesResponse xmlns=\"http://ec2.amazonaws.com/doc/2016-11-15/\"><availabilityZoneInfo><item><zoneName>us-west-2a</zoneName></item><item><zoneName>us-west-2c</zoneName></item></availabilityZoneInfo></DescribeAvailabilityZonesResponse>",
        "<GetTopicAttributesResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\"><GetTopicAttributesResult><Attributes><entry><key>DisplayName</key><value>observed-topic-name</value></entry><entry><key>Owner</key><value>observed-owner-419</value></entry></Attributes></GetTopicAttributesResult></GetTopicAttributesResponse>",
        "<CreateTopicResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\"><CreateTopicResult><TopicArn>arn:aws:sns:us-west-2:123456789012:server-selected-topic-id</TopicArn></CreateTopicResult></CreateTopicResponse>",
    ];
    let (result, requests) = execute(dependencies()?, replies).await?;
    let oracle = result?;
    assert_eq!(requests.len(), 4);
    requests
        .iter()
        .zip(["sts", "ec2", "sns", "sns"])
        .try_for_each(|(request, service)| signed(request, service))?;
    let fields = form(requests.get(3).ok_or("missing CreateTopic request")?)?;
    assert_eq!(
        fields,
        BTreeMap::from([
            ("Action".into(), "CreateTopic".into()),
            ("Version".into(), "2010-03-31".into()),
            ("Name".into(), "observed-topic-name".into()),
            ("Attributes.entry.1.key".into(), "DisplayName".into()),
            ("Attributes.entry.1.value".into(), "us-west-2c".into()),
            ("Tags.member.1.Key".into(), "origin".into()),
            ("Tags.member.1.Value".into(), "observed-owner-419".into()),
        ])
    );
    assert_eq!(oracle.resources.len(), 1);
    assert_eq!(oracle.edges.len(), 0);
    let resource = oracle.resources.first().ok_or("missing topic")?;
    assert_eq!(resource.step, StepId::new(2));
    assert_eq!(resource.kind.as_str(), "AWS::SNS::Topic");
    assert_eq!(
        resource.id.as_str(),
        "arn:aws:sns:us-west-2:123456789012:server-selected-topic-id"
    );
    assert_eq!(resource.key.id, resource.id);
    assert_eq!(resource.evidence, SeedIdentityEvidence::Response);
    Ok(())
}

#[tokio::test]
async fn given_account_scope_when_executed_then_oracle_is_global_but_signing_uses_transport_region()
-> TestResult {
    let reply = "<CreateGroupResponse xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><CreateGroupResult><Group><Arn>arn:aws:iam::123456789012:group/observed-group-983</Arn></Group></CreateGroupResult></CreateGroupResponse>";
    let (result, requests) = execute(group()?, vec![CALLER, reply]).await?;
    let oracle = result?;
    assert_eq!(requests.len(), 2);
    signed(requests.first().ok_or("missing caller request")?, "sts")?;
    let request = requests.get(1).ok_or("missing CreateGroup request")?;
    signed(request, "iam")?;
    assert_eq!(
        form(request)?.get("GroupName").map(String::as_str),
        Some("requested-group")
    );
    assert_eq!(oracle.account_id.as_str(), ACCOUNT);
    assert_eq!(oracle.region.as_str(), REGION);
    assert_eq!(oracle.resources.len(), 1);
    let resource = oracle.resources.first().ok_or("missing group")?;
    assert_eq!(resource.key.region, Region::Global);
    assert_eq!(resource.key.account.as_str(), ACCOUNT);
    assert_eq!(
        resource.id.as_str(),
        "arn:aws:iam::123456789012:group/observed-group-983"
    );
    assert_eq!(resource.evidence, SeedIdentityEvidence::Response);
    Ok(())
}

#[tokio::test]
async fn given_unit_response_when_request_identity_is_explicit_then_template_and_receipt_are_request_sourced()
-> TestResult {
    let (result, requests) = execute(
        alias("arn:{partition}:kms:{region}:{account}:{value}")?,
        vec![CALLER, "{}"],
    )
    .await?;
    let oracle = result?;
    assert_eq!(requests.len(), 2);
    let request = requests.get(1).ok_or("missing CreateAlias request")?;
    signed(request, "kms")?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(parts(request)?.1)?,
        serde_json::json!({"AliasName":"alias/requested-contract", "TargetKeyId":"observed-key-271"})
    );
    assert_eq!(oracle.resources.len(), 1);
    let resource = oracle.resources.first().ok_or("missing alias")?;
    assert_eq!(
        resource.id.as_str(),
        "arn:aws:kms:us-west-2:123456789012:alias/requested-contract"
    );
    assert_eq!(resource.evidence, SeedIdentityEvidence::Request);
    assert_eq!(resource.key.region.as_str(), REGION);
    Ok(())
}

#[tokio::test]
async fn given_missing_or_empty_response_identity_when_executed_then_exact_errors_replace_no_facts()
-> TestResult {
    for (field, message) in [
        (
            "",
            "invalid emulator seed configuration: modeled output field or element is missing",
        ),
        (
            "<Arn></Arn>",
            "resource identity is empty, contains controls, or exceeds 4096 bytes",
        ),
    ] {
        let reply = format!(
            "<CreateGroupResponse><CreateGroupResult><Group>{field}</Group></CreateGroupResult></CreateGroupResponse>"
        );
        let (result, requests) = execute(group()?, vec![CALLER, &reply]).await?;
        assert_eq!(requests.len(), 2);
        signed(requests.get(1).ok_or("missing CreateGroup request")?, "iam")?;
        let error = result.err().ok_or("missing identity was accepted")?;
        assert_eq!(error.to_string(), message);
        match field {
            "" => assert!(matches!(
                error,
                SeedError::Invalid("modeled output field or element is missing")
            )),
            _ => assert!(matches!(
                error,
                SeedError::Inventory(InventoryError::Resource)
            )),
        }
    }
    Ok(())
}

#[test]
fn given_unsafe_identity_templates_when_constructed_then_exact_rejections_prevent_request_identity_fabrication()
-> TestResult {
    for (template, message) in [
        (
            "fixed-identity",
            "identity template must contain {value} and fit its bound",
        ),
        (
            "{value}:{response}",
            "identity template contains an unknown token",
        ),
        (
            "{value}:{account",
            "identity template has an unclosed token",
        ),
        ("{value}}", "identity template has an unmatched brace"),
    ] {
        let error = alias(template)
            .err()
            .ok_or("invalid identity template was accepted")?;
        assert_eq!(
            error.to_string(),
            format!("invalid emulator seed configuration: {message}")
        );
        assert!(matches!(error, SeedError::Invalid(actual) if actual == message));
    }
    Ok(())
}

#[tokio::test]
async fn given_nontext_request_identity_when_executed_then_modeled_type_rejects_it_before_network()
-> TestResult {
    let scenario = SeedScenario::new(vec![SeedStep::new(
        StepId::new(0),
        "sqs".into(),
        "CreateQueue".into(),
        "AWS::SQS::Queue".into(),
        SeedIdentity::Request {
            input_field: FieldPath::new(&["Attributes"])?,
            template: None,
        },
        vec![
            text("QueueName", "seed-contract")?,
            SeedInput::new(
                FieldPath::new(&["Attributes"])?,
                SeedInputValue::Literal(Value::Map(BTreeMap::from([(
                    Key::Name("VisibilityTimeout".into()),
                    Value::Text("30".into()),
                )]))),
            ),
        ],
    )])?;
    let (result, requests) = execute(scenario, vec![]).await?;
    assert_eq!(requests, Vec::<Vec<u8>>::new());
    let error = result
        .err()
        .ok_or("nontext request identity was accepted")?;
    assert!(matches!(
        error,
        SeedError::Invalid("seed identity path must select modeled text")
    ));
    assert_eq!(
        error.to_string(),
        "invalid emulator seed configuration: seed identity path must select modeled text"
    );
    Ok(())
}

#[path = "data_seed_contract/connect.rs"]
mod connect;
