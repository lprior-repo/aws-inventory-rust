//! Behavior: a plain identifier only binds within the source resource's own
//! account and region; same-named resources observed elsewhere never bind.
use super::{ACCOUNT, REGION, SCAN, TestResult, resource, store_with};
use inventory_core::Resource;
use inventory_engine::graph::reference::{self, ReferenceConfig};
use serde_json::json;

const FOREIGN_ACCOUNT: &str = "222222222222";
const FOREIGN_REGION: &str = "eu-west-1";
const SHARED_IDENTIFIER: &str = "shared-identifier";

fn target(account: &str, region: &str) -> Resource {
    let id = format!("arn:aws:service:{region}:{account}:widget/{SHARED_IDENTIFIER}");
    let mut target = resource(
        &id,
        "aws_service_widget",
        json!({"resource_identifier": SHARED_IDENTIFIER}),
    );
    target.account_id = account.to_owned();
    target.region = region.to_owned();
    target
}

fn consumer() -> Resource {
    resource(
        &format!("arn:aws:service:{REGION}:{ACCOUNT}:widget/consumer-77"),
        "aws_service_consumer",
        json!({"linked": SHARED_IDENTIFIER}),
    )
}

#[tokio::test]
async fn plain_identifier_never_binds_a_target_in_another_account() -> TestResult {
    let store = store_with(vec![target(FOREIGN_ACCOUNT, REGION), consumer()]).await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    assert_eq!(result.summary.ambiguous, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn plain_identifier_never_binds_a_target_in_another_region() -> TestResult {
    let store = store_with(vec![target(ACCOUNT, FOREIGN_REGION), consumer()]).await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    assert_eq!(result.summary.ambiguous, 0);
    store.close().await;
    Ok(())
}
