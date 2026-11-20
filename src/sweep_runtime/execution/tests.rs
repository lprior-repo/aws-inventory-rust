mod admission;
mod lease_budget;
mod shutdown;
mod slice_budget;
mod support;

use super::*;

#[test]
fn account_scope_is_scheduled_once_at_home_region() -> Result<(), Box<dyn std::error::Error>> {
    let home = RegionId::parse("us-east-1")?;
    let regions = vec![
        Region::Regional(home.clone()),
        Region::Regional(RegionId::parse("us-west-2")?),
    ];
    let planned = plan_regions(PlanScope::Account, &regions, &home);
    assert_eq!(planned.len(), 1);
    let only = planned.first().ok_or("account scope missing")?;
    assert_eq!(only.scope, Region::Global);
    assert_eq!(only.request, Region::Regional(home));
    Ok(())
}

#[tokio::test]
async fn cancelled_executor_rejects_unstarted_request() -> Result<(), Box<dyn std::error::Error>> {
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    executor.cancel();
    let result = executor
        .execute("test", || async {
            Ok::<(), inventory_engine::limiter::ApiError>(())
        })
        .await;
    assert!(matches!(
        result,
        Err(inventory_engine::limiter::ApiError::Cancelled)
    ));
    Ok(())
}
