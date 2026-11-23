use super::*;

const DOMAIN: &str = "arn:aws:lightsail:global:123456789012:Domain/seed";

#[tokio::test]
async fn given_global_lightsail_domains_when_fake_transport_sweeps_then_account_rows_persist()
-> TestResult {
    let compiled = crate::consumer_fixture::compile("lightsail", "2016-11-28", &["GetDomains"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let scenario = Scenario::new(&catalog, "global-domains").await?;
    scenario.seed(&catalog, "lightsail", &["GetDomains"]).await?;
    let (scenario, seen) = scenario
        .drain(&catalog, "lightsail", |plan, _, wire, resumed| {
            assert_eq!(plan.operation_name(), "GetDomains");
            assert_eq!(wire.method(), http::Method::POST);
            assert!(!resumed);
            Ok(json!({"domains":[{"arn":DOMAIN,"name":"seed.example.com"}]}))
        })
        .await?;
    assert_eq!(seen.get("GetDomains"), Some(&1));
    scenario
        .assert_ids_at("AWS::Lightsail::Domain", "global", vec![DOMAIN.to_owned()])
        .await?;
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.failed_scopes, 0);
    Ok(())
}
