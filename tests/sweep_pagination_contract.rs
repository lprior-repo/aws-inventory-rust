#![cfg(feature = "sweep")]

use anyhow::{Context, Result, ensure};
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use cmdb_view::{Database, PageRequest, ResourceFilter};
use inventory_core::{
    catalog::runtime::{PlanPurpose, RuntimeCatalog},
    record::{FieldId, ValueRef, scalar_item},
};
use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Output, Stdio},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    process::{Child, Command},
    sync::oneshot,
    task::JoinHandle,
};
use tokio_rustls::TlsAcceptor;

const DEADLINE: Duration = Duration::from_secs(20);
const ACCOUNT: &str = "123456789012";
const REGION: &str = "us-east-1";
const KIND: &str = "AWS::ApiOnly::PaginationScalar";

// A small Smithy REST-JSON service, not a replacement collector: the production
// compiler, request encoder, signer, TLS transport, response decoder and SQLite
// writer all run unchanged. The peer serves fixed pages selected by wire cursors.
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n.pagination#Service":{"type":"service","version":"1","operations":[{"target":"n.pagination#GetCallerIdentity"},{"target":"n.pagination#DescribeRegions"},{"target":"n.pagination#ListItems"}],"traits":{"aws.api#service":{"sdkId":"Pagination","endpointPrefix":"pagination","arnNamespace":"pagination"},"aws.auth#sigv4":{"name":"pagination"},"aws.protocols#restJson1":{}}},
"n.pagination#GetCallerIdentity":{"type":"operation","output":{"target":"n.pagination#IdentityOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/identity"}}},
"n.pagination#IdentityOutput":{"type":"structure","members":{"Account":{"target":"smithy.api#String"}}},
"n.pagination#DescribeRegions":{"type":"operation","output":{"target":"n.pagination#RegionsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/regions"}}},
"n.pagination#RegionsOutput":{"type":"structure","members":{"Regions":{"target":"n.pagination#RegionList"}}},
"n.pagination#RegionList":{"type":"list","member":{"target":"n.pagination#Region"}},
"n.pagination#Region":{"type":"structure","members":{"RegionName":{"target":"smithy.api#String"}}},
"n.pagination#ListItems":{"type":"operation","input":{"target":"n.pagination#ListInput"},"output":{"target":"n.pagination#ListOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/items"}}},
"n.pagination#ListInput":{"type":"structure","members":{"StartName":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"name"}},"StartType":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"type"}},"StartIdentifier":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"identifier"}}}},
"n.pagination#ListOutput":{"type":"structure","members":{"Items":{"target":"n.pagination#Items"},"NextName":{"target":"smithy.api#String"},"NextType":{"target":"smithy.api#String"},"NextIdentifier":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean"}}},
"n.pagination#Items":{"type":"list","member":{"target":"smithy.api#String"}}
}}"#;

fn fixture_policy() -> Result<overlay::ServiceOverlay> {
    let plan = |scope, items, identity| -> Result<overlay::OperationOverlay> {
        let mut plan = overlay::OperationOverlay::default();
        plan.class = Some(overlay::OverlayClass::Enumerate);
        plan.scope = Some(scope);
        plan.items = Some(overlay::ItemSelector::parse(items)?);
        plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected(
            identity,
        )?]));
        plan.paginator = Some(overlay::Paginator::None);
        Ok(plan)
    };
    let mut items = plan(overlay::Scope::Region, "Items", "$")?;
    let pair = |input, output| -> Result<overlay::TokenPair> {
        Ok(overlay::TokenPair::new(
            overlay::MemberPath::new(input)?,
            overlay::MemberPath::new(output)?,
        ))
    };
    let mut optional = pair("StartIdentifier", "NextIdentifier")?;
    optional.optional = true;
    items.paginator = Some(overlay::Paginator::Tokens {
        pairs: vec![
            pair("StartName", "NextName")?,
            pair("StartType", "NextType")?,
            optional,
        ],
        truncated: Some(overlay::MemberPath::new("IsTruncated")?),
    });
    Ok(overlay::ServiceOverlay::new(
        "pagination",
        vec![
            (
                "GetCallerIdentity",
                plan(overlay::Scope::Account, "$", "Account")?,
            ),
            (
                "DescribeRegions",
                plan(overlay::Scope::Region, "Regions", "RegionName")?,
            ),
            ("ListItems", items),
        ],
    )?)
}

fn assert_fixture_metadata(bytes: &[u8]) -> Result<()> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    assert_eq!(catalog.plans().len(), 3);
    let account = catalog
        .lookup("pagination", "GetCallerIdentity")
        .context("account bootstrap absent")?;
    assert!(matches!(account.purpose(),
        PlanPurpose::AccountIdentity { account_path } if account_path.as_ref() == [FieldId::new(1)?]));
    let regions = catalog
        .lookup("pagination", "DescribeRegions")
        .context("region bootstrap absent")?;
    assert!(matches!(regions.purpose(),
        PlanPurpose::RegionDiscovery { regions_path, region_name_path, opt_in_status_path }
        if regions_path.as_ref() == [FieldId::new(8)?]
            && region_name_path.as_ref() == [FieldId::new(7)?] && opt_in_status_path.is_none()));
    let items = catalog
        .lookup("pagination", "ListItems")
        .context("scalar plan absent")?;
    assert_eq!(items.purpose(), &PlanPurpose::Enumeration);
    let pagination = items.pagination();
    assert_eq!(pagination.tokens.len(), 3);
    pagination
        .tokens
        .iter()
        .zip([(10, 5, false), (11, 6, false), (9, 4, true)])
        .try_for_each(|(token, (input, output, optional))| -> Result<()> {
            assert_eq!(token.request_path.as_ref(), [FieldId::new(input)?]);
            assert_eq!(token.response_path.as_ref(), [FieldId::new(output)?]);
            assert_eq!(token.optional, optional);
            Ok(())
        })?;
    assert_eq!(
        pagination.truncated_path.as_deref(),
        Some([FieldId::new(2)?].as_slice())
    );
    Ok(())
}

const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Pagination fixture",
    default_unmapped_reason: None,
    cfn: &[],
    relations: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    relation_targets: &[],
    api_only: &[
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationAccount",
            reason: "Bootstrap account",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationRegion",
            reason: "Bootstrap region",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationScalar",
            reason: "Scalar items",
        },
    ],
    operations: &[
        kind_map::OperationMapping {
            service: "pagination",
            operation: "GetCallerIdentity",
            kind: "AWS::ApiOnly::PaginationAccount",
        },
        kind_map::OperationMapping {
            service: "pagination",
            operation: "DescribeRegions",
            kind: "AWS::ApiOnly::PaginationRegion",
        },
        kind_map::OperationMapping {
            service: "pagination",
            operation: "ListItems",
            kind: "AWS::ApiOnly::PaginationScalar",
        },
    ],
    bootstraps: &[
        kind_map::BootstrapMapping::AccountIdentity {
            service: "pagination",
            operation: "GetCallerIdentity",
            account_path: "Account",
        },
        kind_map::BootstrapMapping::RegionDiscovery {
            service: "pagination",
            operation: "DescribeRegions",
            regions_path: "Regions",
            region_name_path: "RegionName",
            opt_in_status_path: None,
        },
    ],
};

#[tokio::test]
async fn sweep_persists_every_scalar_when_full_tuple_advances_and_optional_token_disappears()
-> Result<()> {
    // Given four immutable pages: name stays constant, identifier disappears,
    // then type changes. Comparing only the first token would stop too early.
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::Advancing, None).await?;
    // When the public CLI collects through its actual signed TLS transport.
    let output = fixture.collect(&peer, false).await?;
    // Then both the invocation summary and independently reopened durable data
    // contain all six scalars, with no replayed pages or lost scalar identities.
    assert_summary(
        &output,
        Summary {
            code: 0,
            writes: 6,
            rows: 6,
            complete: 1,
            partial: 0,
            failed: 0,
            pending: 6,
        },
    )?;
    assert_resources(
        &fixture.database,
        &["five", "four", "one", "three", "two", "zero"],
    )
    .await?;
    assert_queue(&fixture.database, 0, &[("complete", 1)], 3).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_reports_partial_and_stops_when_the_entire_continuation_tuple_repeats() -> Result<()>
{
    // Given a service repeating exactly the same name/type/identifier tuple.
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::Repeated, None).await?;
    // When two real responses have been decoded and committed.
    let output = fixture.collect(&peer, false).await?;
    // Then repetition is explicitly partial, not success or an endless replay;
    // observations on the final received page remain durably visible.
    assert_summary(
        &output,
        Summary {
            code: 2,
            writes: 3,
            rows: 3,
            complete: 0,
            partial: 1,
            failed: 0,
            pending: 3,
        },
    )?;
    assert_resources(&fixture.database, &["one", "two", "zero"]).await?;
    assert_queue(&fixture.database, 0, &[("partial", 1)], 1).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_rejects_a_truncated_page_missing_a_required_tuple_component_atomically() -> Result<()>
{
    // Given IsTruncated=true with a name but no required type token.
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::MissingRequired, None).await?;
    // When request assembly, transport and decoding feed that page to the runtime.
    let output = fixture.collect(&peer, false).await?;
    // Then its scalar is not committed and the scope is failed, not complete.
    assert_summary(
        &output,
        Summary {
            code: 2,
            writes: 0,
            rows: 0,
            complete: 0,
            partial: 0,
            failed: 1,
            pending: 0,
        },
    )?;
    assert_resources(&fixture.database, &[]).await?;
    assert_queue(&fixture.database, 0, &[("failed", 1)], 0).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_resumes_the_durable_full_tuple_without_replaying_already_committed_scalar_pages()
-> Result<()> {
    // Given a gate at page three, after two committed pages and after the
    // optional identifier disappeared. Receiving this request proves the prior
    // durable checkpoint was committed, without timing guesses or sleeps.
    let fixture = Fixture::create()?;
    let (arrived_tx, arrived_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let gate = Gate {
        arrived: arrived_tx,
        release: release_rx,
    };
    let peer = Peer::start(fixture.root.path(), Pages::Advancing, Some(gate)).await?;
    let mut interrupted = fixture.spawn(&peer, false)?;
    tokio::time::timeout(DEADLINE, arrived_rx).await??;
    interrupted.kill().await?;
    let interrupted_output =
        tokio::time::timeout(DEADLINE, interrupted.wait_with_output()).await??;
    assert!(!interrupted_output.status.success());
    assert_resources(&fixture.database, &["one", "two", "zero"]).await?;
    assert_queue(&fixture.database, 1, &[], 2).await?;
    release_tx
        .send(())
        .map_err(|_| anyhow::anyhow!("checkpoint gate stopped before release"))?;
    // Then a new process cannot reclaim the killed process's lease while it is
    // still live (ES-02). The refused attempt must leave the durable queue intact.
    let refused = fixture.collect(&peer, true).await?;
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("sweep has live leases"));
    assert_queue(&fixture.database, 1, &[], 2).await?;
    // The lease lapses only after its full budget (about an hour at concurrency 1).
    // Lapse it directly; resume's live-lease predicate is `lease_until > now`.
    expire_leases(&fixture.database).await?;
    let output = fixture.collect(&peer, true).await?;
    assert_summary(
        &output,
        Summary {
            code: 0,
            writes: 3,
            rows: 6,
            complete: 1,
            partial: 0,
            failed: 0,
            pending: 6,
        },
    )?;
    assert_resources(
        &fixture.database,
        &["five", "four", "one", "three", "two", "zero"],
    )
    .await?;
    assert_queue(&fixture.database, 0, &[("complete", 1)], 3).await?;
    peer.stop().await
}

struct Fixture {
    root: tempfile::TempDir,
    catalog: PathBuf,
    database: PathBuf,
    credentials: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn create() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("source");
        let models = source.join("models/pagination/service/2026-01-01");
        std::fs::create_dir_all(&models)?;
        std::fs::write(models.join("model.json"), MODEL)?;
        let checked = overlay::check_policies(
            &source,
            &[fixture_policy()?],
            &catalog_tool::classes::DenyList::builtin(),
        )?;
        let registry = smithy::Registry::parse(
            "Account\nIsTruncated\nItems\nNextIdentifier\nNextName\nNextType\nRegionName\nRegions\nStartIdentifier\nStartName\nStartType\n",
        )?;
        let base = smithy::build_catalog(&source, &registry, "pagination-consumer-fixture")?;
        let kinds = KINDS.compile(&names::Exceptions::none())?;
        let endpoints = BTreeMap::from([("pagination".to_owned(), verified_endpoint()?)]);
        let catalog = root.path().join("fixture.catalog");
        let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
        assert_fixture_metadata(&bytes)?;
        std::fs::write(&catalog, bytes)?;
        let credentials = root.path().join("credentials");
        let config = root.path().join("config");
        std::fs::write(
            &credentials,
            "[pagination]\naws_access_key_id=AKIAPAGINATIONCONTRACT\naws_secret_access_key=local-only-pagination-secret\n",
        )?;
        std::fs::write(&config, "[profile pagination]\nregion=us-east-1\n")?;
        let database = root.path().join("inventory.sqlite");
        Ok(Self {
            root,
            catalog,
            database,
            credentials,
            config,
        })
    }

    fn spawn(&self, peer: &Peer, resume: bool) -> Result<Child> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aws-inventory"));
        command
            .env_clear()
            .env("HOME", self.root.path())
            .env("AWS_CONFIG_FILE", &self.config)
            .env("AWS_SHARED_CREDENTIALS_FILE", &self.credentials)
            .env("AWS_EC2_METADATA_DISABLED", "true")
            .args(["sweep", "--database"])
            .arg(&self.database)
            .arg("--catalog")
            .arg(&self.catalog)
            .args(["--profile", "pagination", "--region", REGION, "--endpoint"])
            .arg(&peer.endpoint)
            .arg("--ca-bundle")
            .arg(&peer.ca)
            .args(["--max-pages", "20", "--concurrency", "1"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if resume {
            command.arg("--resume");
        }
        command.spawn().context("launching public sweep binary")
    }

    async fn collect(&self, peer: &Peer, resume: bool) -> Result<Output> {
        let mut child = self.spawn(peer, resume)?;
        match tokio::time::timeout(DEADLINE, child.wait()).await {
            Ok(status) => {
                status?;
            }
            Err(deadline) => {
                child
                    .kill()
                    .await
                    .context("killing and reaping timed-out sweep")?;
                return Err(deadline.into());
            }
        }
        Ok(tokio::time::timeout(DEADLINE, child.wait_with_output()).await??)
    }
}

fn verified_endpoint() -> Result<endpoints::VerifiedEndpoint> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"pagination","signingRegion":REGION}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://pagination.invalid","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("pagination-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(
        json!([{"params":{"Region":REGION},"expect":{"endpoint":{"url":"https://pagination.invalid","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("pagination"))
        .map_err(|error| anyhow::anyhow!("fixture endpoint derivation failed: {error:?}"))
}

struct Summary {
    code: i32,
    writes: u64,
    rows: u64,
    complete: u64,
    partial: u64,
    failed: u64,
    pending: u64,
}

fn assert_summary(output: &Output, expected: Summary) -> Result<()> {
    let Summary {
        code,
        writes,
        rows,
        complete,
        partial,
        failed,
        pending,
    } = expected;
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // No slice budget is configured, so no work is left for a later slice.
    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        format!(
            "resource_writes_scope=invocation resource_writes={writes} rows={rows} scopes=1 complete={complete} partial={partial} inaccessible=0 unsupported=0 failed={failed} unobserved=0 graph_pending={pending} work_remaining=0 unattributed_resources=0\n"
        )
    );
    Ok(())
}

async fn assert_resources(path: &Path, ids: &[&str]) -> Result<()> {
    let database = Database::open(path).await?;
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 100,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, u64::try_from(ids.len())?);
    assert_eq!(page.next, None);
    let actual = page
        .resources
        .iter()
        .map(|resource| {
            let scalar = match scalar_item(resource.record.root())? {
                Some(ValueRef::Text(value)) => value,
                _ => anyhow::bail!("persisted scalar item was not a text value"),
            };
            Ok((
                resource.key.account.as_str(),
                resource.key.region.as_str(),
                resource.key.kind.as_str(),
                resource.key.id.as_str(),
                scalar,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let expected = ids
        .iter()
        .map(|id| (ACCOUNT, REGION, KIND, *id, *id))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    Ok(())
}

async fn expire_leases(path: &Path) -> Result<()> {
    let options = SqliteConnectOptions::new().filename(path);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::query("UPDATE work_queue SET lease_until=0 WHERE state=1")
        .execute(&pool)
        .await?;
    Ok(())
}

async fn assert_queue(
    path: &Path,
    active: i64,
    outcomes: &[(&str, i64)],
    digests: i64,
) -> Result<()> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .pragma("query_only", "ON");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state IN(0,1)")
        .fetch_one(&pool)
        .await?;
    let cursor_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM work_queue WHERE state IN(0,1) AND token IS NOT NULL",
    )
    .fetch_one(&pool)
    .await?;
    let stored = sqlx::query_as::<_, (String, i64)>(
        "SELECT outcome,COUNT(*) FROM scope_outcomes GROUP BY outcome ORDER BY outcome",
    )
    .fetch_all(&pool)
    .await?;
    let token_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_tokens")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(
        (pending, cursor_count, token_count),
        (active, active, digests)
    );
    assert_eq!(
        stored,
        outcomes
            .iter()
            .map(|(outcome, count)| ((*outcome).to_owned(), *count))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[derive(Clone, Copy)]
enum Pages {
    Advancing,
    Repeated,
    MissingRequired,
}

struct Gate {
    arrived: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

struct Peer {
    endpoint: String,
    ca: PathBuf,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}

impl Peer {
    async fn start(directory: &Path, pages: Pages, gate: Option<Gate>) -> Result<Self> {
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

    async fn stop(mut self) -> Result<()> {
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
    let tuple = query_tuple(&uri)?;
    if uri.path() == "/items"
        && tuple == Some(("same.example.", "A", None))
        && let Some(checkpoint) = gate.take()
    {
        checkpoint
            .arrived
            .send(())
            .map_err(|_| anyhow::anyhow!("checkpoint observer dropped"))?;
        checkpoint.release.await?;
        return Ok(()); // The test killed and reaped this request's process.
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
    let body = match (uri.path(), tuple) {
        ("/identity", None) => json!({"Account": ACCOUNT}),
        ("/regions", None) => json!({"Regions": [{"RegionName": REGION}]}),
        ("/items", None) if matches!(pages, Pages::MissingRequired) => {
            json!({"Items":["must-not-commit"],"IsTruncated":true,"NextName":"same.example."})
        }
        ("/items", None) => {
            json!({"Items":["zero","one"],"IsTruncated":true,"NextName":"same.example.","NextType":"A","NextIdentifier":"blue"})
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
