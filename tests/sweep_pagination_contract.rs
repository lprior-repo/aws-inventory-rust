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
use state::{
    Summary, assert_nested_resources, assert_queue, assert_resources, assert_summary, expire_leases,
};

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
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::Advancing, None).await?;
    let output = fixture.collect(&peer, false).await?;
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
            not_applicable: 0,
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
async fn sweep_flattens_ec2_reservations_keeps_owner_tags_and_continues() -> Result<()> {
    // Given: DescribeInstances returns multiple reservations, a foreign owner, and an empty reservation.
    let fixture = Fixture::create_ec2()?;
    let peer = Peer::start(fixture.root.path(), Pages::Ec2, None).await?;

    // When: the public sweep CLI traverses both DescribeInstances pages.
    let output = fixture.collect(&peer, false).await?;

    // Then: only owned instances are stored, each with its reservation owner and tags.
    assert_summary(
        &output,
        Summary {
            code: 0,
            writes: 3,
            rows: 3,
            complete: 1,
            not_applicable: 0,
            partial: 0,
            failed: 0,
            pending: 3,
            requests: 4,
            pages: 2,
            claims: 1,
        },
    )?;
    assert_nested_resources(&fixture.database, fixture.owner_field()?).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_reports_partial_and_stops_when_the_entire_continuation_tuple_repeats() -> Result<()>
{
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::Repeated, None).await?;
    let output = fixture.collect(&peer, false).await?;
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
            not_applicable: 0,
        },
    )?;
    assert_resources(&fixture.database, &["one", "two", "zero"]).await?;
    assert_queue(&fixture.database, 0, &[("partial", 1)], 1).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_rejects_a_truncated_page_missing_a_required_tuple_component_atomically() -> Result<()>
{
    let fixture = Fixture::create()?;
    let peer = Peer::start(fixture.root.path(), Pages::MissingRequired, None).await?;
    let output = fixture.collect(&peer, false).await?;
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
            not_applicable: 0,
        },
    )?;
    assert_resources(&fixture.database, &[]).await?;
    assert_queue(&fixture.database, 0, &[("failed", 1)], 0).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_resumes_the_durable_full_tuple_without_replaying_already_committed_scalar_pages()
-> Result<()> {
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
    let refused = fixture.collect(&peer, true).await?;
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("sweep has live leases"));
    assert_queue(&fixture.database, 1, &[], 2).await?;
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
            not_applicable: 0,
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
async fn sweep_classifies_a_sandbox_refusal_on_the_first_page_as_not_applicable()
-> Result<()> {
    let fixture = Fixture::create_for(&catalog::SESV2)?;
    let peer = Peer::start(fixture.root.path(), Pages::FirstPrerequisite, None).await?;
    let output = fixture.collect(&peer, false).await?;
    assert_summary(
        &output,
        Summary {
            code: 0,
            writes: 0,
            rows: 0,
            complete: 0,
            partial: 0,
            failed: 0,
            pending: 0,
            requests: 3,
            pages: 0,
            claims: 1,
            not_applicable: 1,
        },
    )?;
    assert_resources(&fixture.database, &[]).await?;
    assert_queue(&fixture.database, 0, &[("not_applicable", 1)], 0).await?;
    peer.stop().await
}

#[tokio::test]
async fn sweep_keeps_a_committed_prefix_partial_when_a_sandbox_refusal_follows()
-> Result<()> {
    let fixture = Fixture::create_for(&catalog::SESV2)?;
    let peer = Peer::start(fixture.root.path(), Pages::ContinuationPrerequisite, None).await?;
    let output = fixture.collect(&peer, false).await?;
    assert_summary(
        &output,
        Summary {
            code: 2,
            writes: 2,
            rows: 2,
            complete: 0,
            partial: 1,
            failed: 0,
            pending: 2,
            requests: 4,
            pages: 1,
            claims: 1,
            not_applicable: 0,
        },
    )?;
    assert_resources(&fixture.database, &["one", "zero"]).await?;
    assert_queue(&fixture.database, 0, &[("partial", 1)], 1).await?;
    peer.stop().await
}
