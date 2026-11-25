use super::*;
use crate::execution::{
    test_catalog,
    test_transport::{Resources, TestResult, lease, serve},
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{Region, RegionId},
};
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn given_scope_region_differs_from_home_when_page_prepared_then_wire_url_and_signature_use_scope()
-> TestResult {
    // Given a regional scope whose request region differs from bootstrap home.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let bytes = test_catalog::compile(
        "cognito-identity-provider",
        "2016-04-18",
        &["ListUserPools"],
        Some(test_catalog::loopback_endpoint(
            "cognito-identity-provider",
            address,
        )?),
        |_, _| Ok(()),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("cognito-identity-provider", "ListUserPools")
        .ok_or("user pool plan absent")?;
    let directory = tempfile::tempdir()?;
    let (store, lease) = lease(
        &catalog,
        plan,
        directory.path(),
        Region::Regional(RegionId::parse("us-west-2")?),
    )
    .await?;
    let resources = Resources::new("us-east-1")?;
    let progress = crate::control::Progress::new(None);
    let context = resources.runtime(&catalog, &store, &progress)?;
    assert_eq!(context.bootstrap.home.as_str(), "us-east-1");
    // When production preparation resolves its cached endpoint and sends the page.
    let prepared = prepare(&context, &lease, plan, &cursor::Cursor::default()).await?;
    let again = prepare(&context, &lease, plan, &cursor::Cursor::default()).await?;
    assert!(std::sync::Arc::ptr_eq(
        &prepared.endpoint.modeled,
        &again.endpoint.modeled
    ));
    let (response, captured) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(
            prepared.send(&context, plan),
            serve(
                listener,
                r#"{"UserPools":[{"Id":"us-west-2_pool","Name":"customer"}]}"#
            )
        )
    })
    .await?;
    let response = response?;
    // Then both wire URL and signing scope use the lease region, never home.
    let captured = captured?;
    captured.assert_route(address, "us-west-2", "cognito-identity-provider")?;
    captured.assert_operation("AWSCognitoIdentityProviderService", "ListUserPools")?;
    assert_eq!(prepared.region, RegionId::parse("us-west-2")?);
    assert_regional_observation(&context, &lease, plan, prepared, response).await?;
    Ok(())
}

#[tokio::test]
async fn given_discovered_scope_differs_from_home_when_prepared_then_discovery_wire_request_uses_scope()
-> TestResult {
    // Given an account-scoped discovery operation for a west-region inventory page.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let bytes = test_catalog::compile(
        "timestream-write",
        "2018-11-01",
        &["DescribeEndpoints", "ListDatabases"],
        Some(test_catalog::loopback_endpoint(
            "timestream-write",
            address,
        )?),
        |name, policy| {
            if name == "DescribeEndpoints" {
                policy.scope = Some(catalog_tool::overlay::Scope::Account);
            }
            Ok(())
        },
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("timestream-write", "ListDatabases")
        .ok_or("database plan absent")?;
    let directory = tempfile::tempdir()?;
    let scope = Region::Regional(RegionId::parse("us-west-2")?);
    let (store, lease) = lease(&catalog, plan, directory.path(), scope).await?;
    let resources = Resources::new("us-east-1")?;
    let progress = crate::control::Progress::new(None);
    let context = resources.runtime(&catalog, &store, &progress)?;
    assert_eq!(context.bootstrap.home.as_str(), "us-east-1");
    let body = r#"{"Endpoints":[{"Address":"ingest-cell.timestream.us-west-2.amazonaws.com","CachePeriodInMinutes":60}]}"#;
    // When preparation drives the real endpoint discovery and caches its lease.
    let cursor = cursor::Cursor::default();
    let (prepared, captured) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(
            prepare(&context, &lease, plan, &cursor),
            serve(listener, body)
        )
    })
    .await?;
    let prepared = prepared?;
    assert_discovery_cache_entries(&catalog, context.bootstrap.resolved_endpoints).await?;
    // Then the sent discovery URL and signing scope use page region, not home or global scope.
    captured?.assert_route(address, "us-west-2", "timestream-write")?;
    assert_eq!(prepared.region, RegionId::parse("us-west-2")?);
    assert_eq!(
        prepared.endpoint.modeled.url,
        format!("http://{address}/us-west-2")
    );
    assert_eq!(
        prepared.endpoint.url(),
        "https://ingest-cell.timestream.us-west-2.amazonaws.com"
    );
    Ok(())
}

async fn assert_regional_observation(
    context: &crate::execution::RuntimeContext<
        '_,
        '_,
        crate::credentials::TestCredentials,
        inventory_engine::inventory::BinaryStore,
    >,
    lease: &inventory_engine::inventory::WorkLease,
    plan: &CallablePlan<'_>,
    prepared: Prepared,
    response: crate::call::DecodedResponse,
) -> TestResult {
    use inventory_core::inventory::{
        AccountId, DiscoveryContext, Kind, Region, ResourceId, ResourceKey,
    };
    let context = super::super::assembly::Context {
        catalog: context.bootstrap.catalog,
        account: &context.account,
        home: context.bootstrap.home,
        executor: context.bootstrap.executor,
        consumers: &context.consumers,
        max_pages: 10,
    };
    let parts = super::super::assembly::prepare(
        &context,
        lease,
        plan,
        cursor::Cursor::default(),
        response,
        &super::super::PageRequest {
            record: prepared.base,
            region: prepared.region,
        },
        &DiscoveryContext::empty(),
    )
    .await?;
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        parts
            .observations
            .first()
            .ok_or("user pool observation absent")?
            .key(),
        &ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::Regional(RegionId::parse("us-west-2")?),
            kind: Kind::parse("AWS::Cognito::UserPool")?,
            id: ResourceId::parse("us-west-2_pool")?,
        }
    );
    Ok(())
}

async fn assert_discovery_cache_entries(
    catalog: &RuntimeCatalog<'_>,
    cache: &crate::execution::resolved_endpoints::ResolvedEndpoints,
) -> TestResult {
    let mut expected = ["DescribeEndpoints", "ListDatabases"]
        .into_iter()
        .map(|operation| {
            Ok((
                catalog
                    .plan_index("timestream-write", operation)
                    .ok_or("Timestream plan index absent")?,
                RegionId::parse("us-west-2")?,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    expected.sort();
    assert_eq!(cache.keys().await, expected);
    Ok(())
}
