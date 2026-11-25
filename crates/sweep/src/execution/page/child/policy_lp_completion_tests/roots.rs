use super::*;
mod accounts;

#[tokio::test]
async fn given_serverless_policy_type_domains_when_every_branch_resumes_then_same_named_types_remain_distinct()
-> TestResult {
    let operations = [
        (
            "ListAccessPolicies",
            "accessPolicySummaries",
            "AWS::OpenSearchServerless::AccessPolicy",
            &["data"][..],
        ),
        (
            "ListLifecyclePolicies",
            "lifecyclePolicySummaries",
            "AWS::OpenSearchServerless::LifecyclePolicy",
            &["retention"][..],
        ),
        (
            "ListSecurityPolicies",
            "securityPolicySummaries",
            "AWS::OpenSearchServerless::SecurityPolicy",
            &["encryption", "network"][..],
        ),
        (
            "ListSecurityConfigs",
            "securityConfigSummaries",
            "AWS::OpenSearchServerless::SecurityConfig",
            &["saml", "iamidentitycenter", "iamfederation"][..],
        ),
    ];
    let names: Vec<_> = operations.iter().map(|entry| entry.0).collect();
    let bytes = compiled(
        "opensearchserverless",
        "2021-11-01",
        &names,
        operations.first().ok_or("empty cases")?.2,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "serverless-all-policy-types").await?;
    for operation in &names {
        scenario
            .seed(plan(&catalog, "opensearchserverless", operation)?)
            .await?;
    }
    let mut seen = BTreeSet::new();
    for _ in 0..14 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let case = operations
            .iter()
            .find(|case| case.0 == operation)
            .ok_or("unexpected serverless work")?;
        let callable = plan(&catalog, "opensearchserverless", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        let kind = text(&native, "type")?;
        assert!(case.3.contains(&kind.as_str()));
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), kind.clone(), second)));
        let label = if second { "second" } else { "shared" };
        let row = if operation == "ListSecurityConfigs" {
            json!({"id":format!("{kind}-{label}"),"type":kind})
        } else {
            json!({"name":label})
        };
        scenario
            .apply(
                &catalog,
                callable,
                lease,
                &request,
                page(json!({(case.1):[row]}), "nextToken", second)?,
            )
            .await?;
    }
    for (operation, _, kind, types) in operations {
        let mut expected = BTreeSet::new();
        for variant in types {
            for label in ["shared", "second"] {
                assert!(seen.contains(&(
                    operation.to_owned(),
                    (*variant).to_owned(),
                    label == "second"
                )));
                expected.insert(if operation == "ListSecurityConfigs" {
                    format!("{variant}-{label}")
                } else {
                    format!("{}:{}|{}:{}", variant.len(), variant, label.len(), label)
                });
            }
        }
        assert_eq!(scenario.stored(kind).await?, (expected, BTreeSet::new()));
    }
    Ok(())
}

type MediaConnectCase = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

const MEDIACONNECT_CASES: [MediaConnectCase; 3] = [
    (
        "ListGatewayInstances",
        "instances",
        "gatewayInstanceArn",
        "AWS::ApiOnly::MediaConnectGatewayInstance",
        "gateway-instance",
    ),
    (
        "ListSourceEndpoints",
        "sourceEndpoints",
        "arn",
        "AWS::MediaConnect::SourceEndpoint",
        "source-endpoint",
    ),
    (
        "ListDestinationEndpoints",
        "destinationEndpoints",
        "arn",
        "AWS::MediaConnect::DestinationEndpoint",
        "destination-endpoint",
    ),
];

#[tokio::test]
async fn given_mediaconnect_owned_roots_when_unfiltered_pages_resume_then_endpoints_and_gateway_installations_persist()
-> TestResult {
    let cases = &MEDIACONNECT_CASES;
    let names = [
        "ListGateways",
        "ListGatewayInstances",
        "ListSourceEndpoints",
        "ListDestinationEndpoints",
    ];
    let bytes = compiled(
        "mediaconnect",
        "2018-11-14",
        &names,
        "AWS::MediaConnect::Gateway",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "mediaconnect-owned-roots").await?;
    for operation in names {
        scenario
            .seed(plan(&catalog, "mediaconnect", operation)?)
            .await?;
    }
    let gateway = "arn:aws:mediaconnect:us-east-1:123456789012:gateway:one";
    let seen = mediaconnect_pages(&scenario, &catalog, cases, gateway).await?;
    for &(operation, _, _, kind, resource) in cases {
        let expected: BTreeSet<_> = ["one", "two"]
            .into_iter()
            .map(|suffix| {
                format!("arn:aws:mediaconnect:us-east-1:123456789012:{resource}:{suffix}")
            })
            .collect();
        let edges = if operation == "ListGatewayInstances" {
            expected
                .iter()
                .map(|id| (id.clone(), gateway.to_owned(), "IN_GATEWAY".to_owned()))
                .collect()
        } else {
            expected
                .iter()
                .map(|id| (id.clone(), id.clone(), "REFERENCES".to_owned()))
                .collect()
        };
        assert_eq!(scenario.stored(kind).await?, (expected, edges));
        assert!(seen.contains(&(operation.to_owned(), true)));
    }
    Ok(())
}

async fn mediaconnect_pages(
    scenario: &Scenario<'_, '_>,
    catalog: &RuntimeCatalog<'_>,
    cases: &[MediaConnectCase],
    gateway: &str,
) -> TestResult<BTreeSet<(String, bool)>> {
    let mut seen = BTreeSet::new();
    for _ in 0..7 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(catalog, "mediaconnect", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), second)));
        let body = if operation == "ListGateways" {
            json!({"gateways":[{"gatewayArn":gateway}]})
        } else {
            let case = cases
                .iter()
                .find(|case| case.0 == operation)
                .ok_or("unexpected MediaConnect work")?;
            let native = native_input(&wire)?;
            assert!(!native.contains("gatewayArn="));
            let arn = format!(
                "arn:aws:mediaconnect:us-east-1:123456789012:{}:{}",
                case.4,
                if second { "two" } else { "one" }
            );
            let row = if operation == "ListGatewayInstances" {
                json!({(case.2):arn,"gatewayArn":gateway,"instanceId":"i-12345678"})
            } else {
                json!({(case.2):arn,"resourceShareOwnership":"SELF"})
            };
            page(json!({(case.1):[row]}), "nextToken", second)?
        };
        scenario
            .apply(catalog, callable, lease, &request, body)
            .await?;
    }
    Ok(seen)
}
