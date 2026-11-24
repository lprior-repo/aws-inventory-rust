use super::*;
use crate::{
    execution::test_transport::{Resources, lease},
    record::policy_dfgk_tests::fixture,
};
use inventory_core::{
    inventory::{Kind, Region, ResourceId, ResourceKey},
    record::Record,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_non_home_workspace_arn_when_page_assembled_then_whole_key_uses_request_scope()
-> TestResult {
    // Given a us-west-2 ARN and lease, while bootstrap home remains us-east-1.
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let record = fixture::decode_as_sweep(plan, br#"{"workspaceSummaries":[{"workspaceId":"one","arn":"arn:aws:iottwinmaker:us-west-2:123456789012:workspace/one","creationDateTime":0,"updateDateTime":1}]}"#)?;
    // When the production page assembly validates identities and moves observations.
    let (parts, request_region) = prepare_page(
        &catalog,
        plan,
        record,
        Region::Regional(RegionId::parse("us-west-2")?),
    )
    .await?;
    assert_eq!(request_region, RegionId::parse("us-west-2")?);
    // Then every key component is exact, including the non-home region.
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        parts
            .observations
            .first()
            .ok_or("workspace observation absent")?
            .key(),
        &ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::Regional(RegionId::parse("us-west-2")?),
            kind: Kind::parse("AWS::IoTTwinMaker::Workspace")?,
            id: ResourceId::parse("arn:aws:iottwinmaker:us-west-2:123456789012:workspace/one")?,
        }
    );
    Ok(())
}

async fn prepare_page(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    record: Record,
    scope: Region,
) -> TestResult<(PageParts, RegionId)> {
    let directory = tempfile::tempdir()?;
    let (store, lease) = lease(catalog, plan, directory.path(), scope).await?;
    let resources = Resources::new("us-east-1")?;
    let progress = crate::control::Progress::new(None);
    let runtime = resources.runtime(catalog, &store, &progress)?;
    assert_eq!(runtime.bootstrap.home.as_str(), "us-east-1");
    let prepared =
        super::super::preparation::prepare(&runtime, &lease, plan, &cursor::Cursor::default())
            .await?;
    let region = prepared.region;
    let context = Context {
        catalog,
        account: &runtime.account,
        home: runtime.bootstrap.home,
        executor: runtime.bootstrap.executor,
        consumers: &runtime.consumers,
        max_pages: 10,
    };
    let parts = prepare(
        &context,
        &lease,
        plan,
        cursor::Cursor::default(),
        call::DecodedResponse {
            record,
            headers: http::HeaderMap::new(),
            http_version: http::Version::HTTP_11,
        },
        &PageRequest {
            record: prepared.base,
            region: region.clone(),
        },
        &DiscoveryContext::empty(),
    )
    .await?;
    Ok((parts, region))
}

#[tokio::test]
async fn given_global_role_lease_when_page_assembled_then_whole_key_remains_global() -> TestResult {
    // Given account-global IAM work with a regional request endpoint.
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iam", "ListRoles")?;
    let record = fixture::decode_as_sweep(plan, br#"<ListRolesResponse><ListRolesResult><Roles><member><Path>/</Path><RoleName>worker</RoleName><RoleId>AROATEST</RoleId><Arn>arn:aws:iam::123456789012:role/worker</Arn><CreateDate>2024-01-01T00:00:00Z</CreateDate></member></Roles><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>"#)?;
    // When production preparation chooses a regional request and assembly emits the role.
    let (parts, request_region) = prepare_page(&catalog, plan, record, Region::Global).await?;
    // Then the stored resource key retains its global scope, not the request region.
    assert_eq!(request_region, RegionId::parse("us-east-1")?);
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        parts.observations.first().ok_or("role absent")?.key(),
        &ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::Global,
            kind: Kind::parse("AWS::IAM::Role")?,
            id: ResourceId::parse("arn:aws:iam::123456789012:role/worker")?,
        }
    );
    Ok(())
}

#[tokio::test]
async fn given_regional_lease_with_fixed_request_route_when_assembled_then_key_keeps_lease_region()
-> TestResult {
    // Given supported fixed request routing independent of a regional enumeration scope.
    let bytes = crate::execution::test_catalog::compile(
        "cognito-identity-provider",
        "2016-04-18",
        &["ListUserPools"],
        None,
        |_, policy| {
            policy.routing = catalog_tool::overlay::RequestRouting::FixedRegion {
                region: "us-west-2".to_owned(),
            };
            Ok(())
        },
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = fixture::plan(&catalog, "cognito-identity-provider", "ListUserPools")?;
    assert_eq!(
        plan.scope(),
        inventory_core::catalog::runtime::PlanScope::Region
    );
    let record = fixture::decode_as_sweep(
        plan,
        br#"{"UserPools":[{"Id":"us-west-2_pool","Name":"customer"}]}"#,
    )?;
    // When production preparation routes east-scoped work to the fixed west endpoint.
    let (parts, request_region) = prepare_page(
        &catalog,
        plan,
        record,
        Region::Regional(RegionId::parse("us-east-1")?),
    )
    .await?;
    // Then the resource belongs to its lease scope, not its request destination.
    assert_eq!(request_region, RegionId::parse("us-west-2")?);
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        parts.observations.first().ok_or("pool absent")?.key(),
        &ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::Regional(RegionId::parse("us-east-1")?),
            kind: Kind::parse("AWS::Cognito::UserPool")?,
            id: ResourceId::parse("us-west-2_pool")?,
        }
    );
    Ok(())
}
