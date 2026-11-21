mod fixture;
mod scenario;
use crate::sweep_runtime::execution::page::cursor;

use crate::sweep_runtime::{
    execution::{
        page::{child, consumer_index::ConsumerIndex},
        routing,
    },
    input, record, request,
};
use cursor::Cursor;
use inventory_core::{
    catalog::runtime::{PlanScope, RuntimeCatalog},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::{KeyRef, Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const KIND: &str = "AWS::GameLift::ContainerFleet";

#[tokio::test]
async fn given_owned_container_fleets_when_empty_continuation_reopens_then_distinct_native_rows_and_graph_nodes_survive_without_detail_calls()
-> TestResult {
    // Given the actual pinned model, embedded production policy and canonical CFN kind map.
    let catalog = fixture::catalog()?;
    let mut scenario = scenario::Scenario::new(&catalog, "container-fleets-restart").await?;
    scenario.page(&catalog, 0, 0, fixture::response(0)?).await?;
    assert_eq!(
        scenario.store.projection_resources(None, 10).await?.len(),
        0
    );
    // When the BinaryStore itself is reopened, claim must recover the persisted empty-page cursor.
    drop(scenario.store);
    scenario.store = BinaryStore::open(&scenario.path).await?;
    scenario.page(&catalog, 1, 1, fixture::response(1)?).await?;
    drop(scenario.store);
    scenario.store = BinaryStore::open(&scenario.path).await?;
    scenario.page(&catalog, 2, 2, fixture::response(2)?).await?;
    drop(scenario.store);
    scenario.store = BinaryStore::open(&scenario.path).await?;
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 3, 10_000)
            .await?
            .is_none(),
    );
    // Then query the real SQLite facets, not a second mock/projection of the response.
    let reader = scenario.reader().await?;
    let saved: Vec<(String, String, Vec<u8>)> =
        sqlx::query_as("SELECT kind,id,record FROM resource_facets ORDER BY id")
            .fetch_all(&reader)
            .await?;
    assert_eq!(saved.len(), 2);
    let expected_ids = [fixture::arn("first"), fixture::arn("later")];
    assert_eq!(
        saved
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        expected_ids
            .iter()
            .map(|id| (KIND, id.as_str()))
            .collect::<Vec<_>>()
    );
    let fields = fixture::fields()?;
    for ((_, _, bytes), ordinal) in saved.into_iter().zip(["first", "later"]) {
        let persisted = Record::from_bytes(bytes)?;
        assert_eq!(
            fixture::json_value(ValueRef::Map(persisted.root()), &fields)?,
            fixture::row(ordinal)
        );
    }
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs")
        .fetch_one(&reader)
        .await?;
    assert_eq!(pending, 0);
    reader.close().await;
    let nodes = scenario.store.projection_resources(None, 10).await?;
    assert_eq!(
        nodes
            .iter()
            .map(|node| (node.key.kind.as_str(), node.key.id.as_str()))
            .collect::<BTreeSet<_>>(),
        expected_ids
            .iter()
            .map(|id| (KIND, id.as_str()))
            .collect::<BTreeSet<_>>()
    );
    // IAM/container-definition/log references are metadata, not independently owned observations.
    assert_eq!(
        scenario.store.projection_edges(None, 10).await?.edges.len(),
        0
    );
    Ok(())
}

#[test]
fn given_reviewed_container_metadata_when_embedded_plan_compiles_then_detail_content_credentials_and_mutations_remain_uncallable()
-> TestResult {
    // Given ListContainerFleets returns configuration references, never container definitions/log bytes.
    let catalog = fixture::catalog()?;
    assert_eq!(scenario::plan(&catalog)?.kind().as_str(), KIND);
    // Then neither a public reference nor a nested configuration authorizes more API calls.
    for operation in fixture::FORBIDDEN {
        assert!(
            catalog.lookup("gamelift", operation).is_none(),
            "forbidden {operation} became callable"
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_actual_request_scope_when_a_fleet_arn_is_hostile_then_the_whole_page_has_no_sqlite_or_graph_observation()
-> TestResult {
    let catalog = fixture::catalog()?;
    for (ordinal, arn, message) in [
        ("malformed", "not-an-arn", "modeled ARN is malformed"),
        (
            "foreign-account",
            "arn:aws:gamelift:us-east-1:999999999999:containerfleet/containerfleet-bad",
            "modeled ARN does not match trusted request scope",
        ),
        (
            "foreign-partition",
            "arn:aws-cn:gamelift:us-east-1:123456789012:containerfleet/containerfleet-bad",
            "modeled ARN does not match trusted request scope",
        ),
        // us-west-2 is the runner home region; only actual request region us-east-1 is authority.
        (
            "foreign-request-region",
            "arn:aws:gamelift:us-west-2:123456789012:containerfleet/containerfleet-bad",
            "modeled ARN does not match trusted request scope",
        ),
    ] {
        // Given a genuine durable empty page followed by one good and one hostile native row.
        let scenario = scenario::Scenario::new(&catalog, ordinal).await?;
        scenario.page(&catalog, 0, 0, fixture::response(0)?).await?;
        let mut hostile = fixture::row("bad");
        hostile
            .as_object_mut()
            .ok_or("hostile row is not an object")?
            .insert("FleetArn".to_owned(), json!(arn));
        // When the real row gate processes the mixed page, no partial observation can commit.
        let error = scenario
            .page(
                &catalog,
                1,
                1,
                json!({"ContainerFleets":[fixture::row("first"),hostile]}),
            )
            .await
            .err()
            .ok_or("hostile fleet ARN was accepted")?;
        assert_eq!(
            error.to_string(),
            format!("sweep operation failed: {message}"),
            "{ordinal}"
        );
        let reader = scenario.reader().await?;
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets")
            .fetch_one(&reader)
            .await?;
        let references: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs")
            .fetch_one(&reader)
            .await?;
        assert_eq!((rows, references), (0, 0), "{ordinal}");
        reader.close().await;
        assert_eq!(
            scenario.store.projection_resources(None, 10).await?.len(),
            0,
            "{ordinal}"
        );
        assert_eq!(
            scenario.store.projection_edges(None, 10).await?.edges.len(),
            0,
            "{ordinal}"
        );
    }
    Ok(())
}
