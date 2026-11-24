#![cfg(feature = "sweep")]

#[path = "sweep_pagination_contract/catalog.rs"]
mod catalog;
#[path = "sweep_pagination_contract/peer.rs"]
mod peer;
#[path = "sweep_pagination_contract/state.rs"]
mod state;
#[path = "sweep_pagination_contract/stdout.rs"]
mod stdout;
use catalog::Fixture;
use peer::{Gate, Pages, Peer};
use state::{Summary, assert_queue, assert_resources, assert_summary, expire_leases};

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
            requests: 6,
            pages: 4,
            claims: 1,
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
            requests: 4,
            pages: 2,
            claims: 1,
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
            requests: 3,
            pages: 0,
            claims: 1,
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
            requests: 4,
            pages: 2,
            claims: 1,
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
