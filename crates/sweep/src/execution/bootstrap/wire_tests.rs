use super::*;
use crate::execution::{
    test_catalog,
    test_transport::{Resources, TestResult, serve},
};
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn given_bootstrap_home_differs_from_config_when_executed_then_wire_url_and_signature_use_home()
-> TestResult {
    // Given an account-scoped bootstrap whose home differs from configured and credential regions.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let bytes = test_catalog::compile(
        "sts",
        "2011-06-15",
        &["GetCallerIdentity"],
        Some(test_catalog::loopback_endpoint("sts", address)?),
        |_, policy| {
            policy.scope = Some(catalog_tool::overlay::Scope::Account);
            Ok(())
        },
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("sts", "GetCallerIdentity")
        .ok_or("identity plan absent")?;
    assert_eq!(plan.scope(), PlanScope::Account);
    let resources = Resources::new("us-west-2")?;
    let context = resources.bootstrap(&catalog);
    assert_eq!(context.config.region.as_deref(), Some("us-east-1"));
    assert_eq!(context.credentials.region(), "us-east-1");
    let body = "<GetCallerIdentityResponse><GetCallerIdentityResult><Account>123456789012</Account><Arn>arn:aws:iam::123456789012:user/test</Arn><UserId>user</UserId></GetCallerIdentityResult></GetCallerIdentityResponse>";
    // When the production bootstrap caller resolves, signs, and sends its request.
    let (response, captured) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(execute(&context, plan, None), serve(listener, body))
    })
    .await?;
    let response = response?;
    assert_eq!(
        context.resolved_endpoints.keys().await,
        vec![(
            catalog
                .plan_index("sts", "GetCallerIdentity")
                .ok_or("identity index absent")?,
            RegionId::parse("us-west-2")?,
        )],
    );
    // Then the actual wire target and credential scope use home, not the configured region.
    let captured = captured?;
    captured.assert_route(address, "us-west-2", "sts")?;
    captured.assert_operation("sts", "GetCallerIdentity")?;
    let PlanPurpose::AccountIdentity { account_path } = plan.purpose() else {
        return Err("account identity purpose absent".into());
    };
    assert_eq!(
        account_id(&response.record, account_path)?.as_str(),
        "123456789012"
    );
    Ok(())
}
