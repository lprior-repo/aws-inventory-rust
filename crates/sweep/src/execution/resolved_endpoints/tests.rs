use super::*;
use crate::record::policy_dfgk_tests::fixture;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_plan_and_region_when_resolved_again_then_endpoint_storage_is_shared() -> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let region = RegionId::parse("us-east-1")?;
    let cache = ResolvedEndpoints::default();
    let first = cache.resolve(&catalog, plan, &region).await?;
    let again = cache.resolve(&catalog, plan, &region).await?;
    assert!(Arc::ptr_eq(&first, &again));
    assert_eq!(*first, super::super::routing::resolve(plan, &region)?);
    let other_region = RegionId::parse("us-west-2")?;
    let regional = cache.resolve(&catalog, plan, &other_region).await?;
    assert!(!Arc::ptr_eq(&first, &regional));
    assert_eq!(
        *regional,
        super::super::routing::resolve(plan, &other_region)?
    );
    let other_plan = fixture::plan(&catalog, "iam", "ListRoles")?;
    let other = cache.resolve(&catalog, other_plan, &region).await?;
    assert!(!Arc::ptr_eq(&first, &other));
    assert_eq!(
        *other,
        other_plan.endpoint(&region, &inventory_core::inventory::Region::Global)?
    );
    assert_eq!(cache.0.lock().await.len(), 3);
    let index = catalog
        .plan_index(plan.service_name(), plan.operation_name())
        .ok_or("plan index absent")?;
    assert!(std::ptr::eq(
        catalog.plans().get(index).ok_or("indexed plan absent")?,
        plan
    ));
    assert_eq!(catalog.plan_index("missing", "operation"), None);
    Ok(())
}

#[tokio::test]
async fn given_endpoint_cache_when_filled_past_capacity_then_exact_bound_and_hits_survive()
-> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let cache = ResolvedEndpoints::default();
    for ordinal in 0..MAX_ENDPOINTS {
        let region = RegionId::parse(&format!("us-east-{ordinal}"))?;
        cache.resolve(&catalog, plan, &region).await?;
    }
    assert_eq!(cache.0.lock().await.len(), 65_536);
    let overflow = RegionId::parse("us-east-65536")?;
    let miss = cache.resolve(&catalog, plan, &overflow).await?;
    let repeated = cache.resolve(&catalog, plan, &overflow).await?;
    assert_eq!(*miss, *repeated);
    assert!(!Arc::ptr_eq(&miss, &repeated));
    assert_eq!(*miss, super::super::routing::resolve(plan, &overflow)?);
    assert_eq!(cache.0.lock().await.len(), 65_536);
    let retained = RegionId::parse("us-east-0")?;
    let first = cache.resolve(&catalog, plan, &retained).await?;
    let hit = cache.resolve(&catalog, plan, &retained).await?;
    assert!(Arc::ptr_eq(&first, &hit));
    Ok(())
}

#[tokio::test]
async fn given_bootstrap_plans_when_cached_then_resolution_uses_regional_scope() -> TestResult {
    for (service, date, operation, operations) in [
        (
            "sts",
            "2011-06-15",
            "GetCallerIdentity",
            ["GetCallerIdentity"].as_slice(),
        ),
        (
            "ec2",
            "2016-11-15",
            "DescribeRegions",
            ["DescribeRegions"].as_slice(),
        ),
        (
            "timestream-write",
            "2018-11-01",
            "DescribeEndpoints",
            ["DescribeEndpoints", "ListDatabases"].as_slice(),
        ),
    ] {
        assert_bootstrap_scope(service, date, operation, operations).await?;
    }
    Ok(())
}

async fn assert_bootstrap_scope(
    service: &str,
    date: &str,
    operation: &str,
    operations: &[&str],
) -> TestResult {
    use inventory_core::{catalog::runtime::PlanPurpose, inventory::Region};
    let bytes =
        super::super::test_catalog::compile(service, date, operations, None, |name, policy| {
            if name == operation {
                policy.scope = Some(catalog_tool::overlay::Scope::Account);
            }
            Ok(())
        })?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(service, operation)
        .ok_or("bootstrap plan absent")?;
    let region = RegionId::parse("us-west-2")?;
    let scope = Region::Regional(region.clone());
    let cache = ResolvedEndpoints::default();
    assert!(matches!(
        plan.purpose(),
        PlanPurpose::AccountIdentity { .. }
            | PlanPurpose::RegionDiscovery { .. }
            | PlanPurpose::EndpointDiscovery
    ));
    let actual = match plan.purpose() {
        PlanPurpose::EndpointDiscovery => {
            super::super::discovery::resolve_endpoint(&catalog, plan, &region, &cache).await?
        }
        _ => super::super::bootstrap::resolve_endpoint(&catalog, plan, &region, &cache).await?,
    };
    assert_eq!(*actual, plan.endpoint(&region, &scope)?);
    assert_eq!(cache.0.lock().await.len(), 1);
    assert_ne!(
        plan.endpoint(&region, &scope)?,
        plan.endpoint(&region, &Region::Global)?
    );
    Ok(())
}

#[tokio::test]
async fn given_plan_absent_from_run_catalog_when_endpoint_resolved_then_exact_error_is_returned()
-> TestResult {
    let bytes = super::super::test_catalog::compile(
        "sts",
        "2011-06-15",
        &["GetCallerIdentity"],
        None,
        |_, _| Ok(()),
    )?;
    let foreign_catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = fixture::plan(&foreign_catalog, "sts", "GetCallerIdentity")?;
    let catalog = fixture::catalog()?;
    assert_eq!(catalog.plan_index("sts", "GetCallerIdentity"), None);
    let region = RegionId::parse("us-east-1")?;
    let cache = ResolvedEndpoints::default();
    let error = cache
        .resolve(&catalog, plan, &region)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "sweep operation failed: endpoint plan is absent from run catalog"
    );
    assert_eq!(cache.0.lock().await.len(), 0);
    Ok(())
}
