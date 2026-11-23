mod cases;
mod fixture;
mod safety;
mod scenario;
use crate::execution::page::cursor;

use crate::{
    execution::{
        page::{child, consumer_index::ConsumerIndex},
        routing,
    },
    input, record, request,
};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanScope, RuntimeCatalog},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::{Key, Record, Value, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use scenario::Scenario;
use serde_json::{Value as Json, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn resume_root(case: &cases::Case) -> TestResult {
    // Given an unfiltered pinned-model root and no preexisting/parent resources.
    let catalog = fixture::catalog()?;
    let mut scenario = Scenario::new(&catalog, case).await?;
    scenario.seed(&catalog, case).await?;
    let lease = scenario.claim(case, 0).await?;
    scenario
        .apply(&catalog, case, lease, 0, case.page(0)?, 0)
        .await?;
    scenario.assert_empty().await?;
    // When the empty first page has persisted continuation, and a worker crashes
    // while holding that continuation lease, open the actual store and reclaim it.
    let abandoned = scenario.claim(case, 1).await?;
    let token = abandoned
        .token()
        .ok_or("empty page lost continuation")?
        .to_vec();
    let bindings = abandoned.bindings().to_vec();
    drop(abandoned);
    scenario = scenario.reopen().await?;
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 2, 10)
            .await?
            .is_none(),
        "unexpired lease must not be stolen"
    );
    let reclaimed = scenario.claim(case, 12).await?;
    assert_eq!(reclaimed.token(), Some(token.as_slice()));
    assert_eq!(reclaimed.bindings(), bindings);
    scenario
        .apply(&catalog, case, reclaimed, 1, case.page(1)?, 12)
        .await?;
    scenario = scenario.reopen().await?;
    let lease = scenario.claim(case, 13).await?;
    scenario
        .apply(&catalog, case, lease, 2, case.page(2)?, 13)
        .await?;
    scenario = scenario.reopen().await?;
    // Then both pages preserve exact owned metadata, tags, SQLite rows and DAG nodes.
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 14, 10)
            .await?
            .is_none()
    );
    scenario.assert_resources(&catalog, case).await
}

#[tokio::test]
async fn given_ad_configurations_when_empty_continuation_is_reclaimed_after_store_open_then_both_metadata_nodes_survive()
-> TestResult {
    resume_root(cases::CASES.first().ok_or("ad configuration case absent")?).await
}

#[tokio::test]
async fn given_failover_slate_configurations_when_empty_continuation_is_reclaimed_after_store_open_then_both_metadata_nodes_survive()
-> TestResult {
    resume_root(cases::CASES.last().ok_or("failover slate case absent")?).await
}

#[tokio::test]
async fn given_valid_owned_row_before_hostile_arn_when_native_page_is_consumed_then_scope_failure_prevents_any_page_commit()
-> TestResult {
    // Given both actual compiled roots and strict request account/partition/region scope.
    let catalog = fixture::catalog()?;
    for case in cases::CASES {
        for forged in [
            "not-an-arn".to_owned(),
            format!("arn:aws:ivs:us-east-1:999999999999:{}/later", case.resource),
            format!(
                "arn:aws-cn:ivs:us-east-1:123456789012:{}/later",
                case.resource
            ),
            format!("arn:aws:ivs:us-west-2:123456789012:{}/later", case.resource),
        ] {
            let mut scenario = Scenario::new(&catalog, case).await?;
            scenario.seed(&catalog, case).await?;
            let lease = scenario.claim(case, 0).await?;
            let mut foreign = case.row("later");
            foreign
                .as_object_mut()
                .ok_or("IVS row is not an object")?
                .insert("arn".to_owned(), json!(forged));
            // When the native wire response mixes a valid row with an invalid one.
            let body = json!({case.items:[case.row("first"), foreign]});
            let error = scenario
                .apply(&catalog, case, lease, 0, body, 0)
                .await
                .err()
                .ok_or("hostile IVS ARN admitted")?;
            assert!(
                error.to_string().contains("modeled ARN"),
                "{}: {error}",
                case.operation
            );
            // Then neither the earlier row nor tags/edges/descendant work becomes durable.
            scenario = scenario.reopen().await?;
            scenario.assert_empty().await?;
            let reader = scenario.reader().await?;
            let queued: Vec<(String,)> = sqlx::query_as("SELECT sweep_id FROM work_queue")
                .fetch_all(&reader)
                .await?;
            assert_eq!(queued, [(case.operation.to_owned(),)]);
            reader.close().await;
            let retry = scenario.claim(case, 11).await?;
            assert_eq!(retry.token(), None);
        }
    }
    Ok(())
}

#[test]
fn given_promoted_metadata_roots_when_compiling_then_secret_content_mutations_stay_denied_and_stream_key_guard_survives()
-> TestResult {
    use inventory_core::catalog::runtime::{MissingSelected, RowPredicatePlan, RowValueType};
    // Given the production policy including the existing dependent stream-key list.
    let catalog = fixture::catalog()?;
    let allowed = [
        "ListAdConfigurations",
        "ListFailoverSlateConfigurations",
        "ListChannels",
        "ListStreamKeys",
    ];
    // When actual pinned plans are compiled, no detail/content/secret/mutation is callable.
    for operation in fixture::OPERATIONS {
        assert_eq!(
            catalog.lookup("ivs", operation).is_some(),
            allowed.contains(operation),
            "unexpected callable IVS operation {operation}"
        );
    }
    // Then the preexisting channel/stream-key request equality is not weakened by promotion.
    let stream_keys = catalog
        .lookup("ivs", "ListStreamKeys")
        .ok_or("stream-key metadata plan absent")?;
    let field = fixture::fields()?
        .id("channelArn")
        .ok_or("channel ARN field absent")?;
    assert_eq!(
        stream_keys.row_policy().assertions.as_ref(),
        [RowPredicatePlan::SelectedRequestEquals {
            selected: vec![field].into_boxed_slice(),
            request: vec![field].into_boxed_slice(),
            value_type: RowValueType::String,
            missing: MissingSelected::AllowAbsent,
        }]
    );
    Ok(())
}
