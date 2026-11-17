use super::*;

#[tokio::test]
async fn given_serverless_policy_type_domains_when_every_branch_resumes_then_same_named_types_remain_distinct() -> TestResult {
    let operations = [
        ("ListAccessPolicies", "accessPolicySummaries", "AWS::OpenSearchServerless::AccessPolicy", &["data"][..]),
        ("ListLifecyclePolicies", "lifecyclePolicySummaries", "AWS::OpenSearchServerless::LifecyclePolicy", &["retention"][..]),
        ("ListSecurityPolicies", "securityPolicySummaries", "AWS::OpenSearchServerless::SecurityPolicy", &["encryption", "network"][..]),
        ("ListSecurityConfigs", "securityConfigSummaries", "AWS::OpenSearchServerless::SecurityConfig", &["saml", "iamidentitycenter", "iamfederation"][..]),
    ];
    let names: Vec<_> = operations.iter().map(|entry| entry.0).collect();
    let bytes = compiled("opensearchserverless", "2021-11-01", &names, operations.first().ok_or("empty cases")?.2)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "serverless-all-policy-types").await?;
    for operation in &names { scenario.seed(plan(&catalog, "opensearchserverless", operation)?).await?; }
    let mut seen = BTreeSet::new();
    for _ in 0..14 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let case = operations.iter().find(|case| case.0 == operation).ok_or("unexpected serverless work")?;
        let callable = plan(&catalog, "opensearchserverless", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        let kind = text(&native, "type")?;
        assert!(case.3.contains(&kind.as_str()));
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), kind.clone(), second)));
        let label = if second { "second" } else { "shared" };
        // Policy rows intentionally omit type: identity must use this branch's real request.
        let row = if operation == "ListSecurityConfigs" { json!({"id":format!("{kind}-{label}"),"type":kind}) } else { json!({"name":label}) };
        scenario.apply(&catalog, callable, lease, &request, page(json!({(case.1):[row]}), "nextToken", second)?).await?;
    }
    for (operation, _, kind, types) in operations {
        let mut expected = BTreeSet::new();
        for variant in types { for label in ["shared", "second"] {
            assert!(seen.contains(&(operation.to_owned(), (*variant).to_owned(), label == "second")));
            expected.insert(if operation == "ListSecurityConfigs" { format!("{variant}-{label}") }
                else { format!("{}:{}|{}:{}", variant.len(), variant, label.len(), label) });
        } }
        assert_eq!(scenario.stored(kind).await?, (expected, BTreeSet::new()));
    }
    Ok(())
}

#[tokio::test]
async fn given_mediaconnect_owned_roots_when_unfiltered_pages_resume_then_endpoints_and_gateway_installations_persist() -> TestResult {
    let cases = [
        ("ListGatewayInstances", "instances", "gatewayInstanceArn", "AWS::ApiOnly::MediaConnectGatewayInstance", "gateway-instance"),
        ("ListSourceEndpoints", "sourceEndpoints", "arn", "AWS::MediaConnect::SourceEndpoint", "source-endpoint"),
        ("ListDestinationEndpoints", "destinationEndpoints", "arn", "AWS::MediaConnect::DestinationEndpoint", "destination-endpoint"),
    ];
    let names = ["ListGateways", "ListGatewayInstances", "ListSourceEndpoints", "ListDestinationEndpoints"];
    let bytes = compiled("mediaconnect", "2018-11-14", &names, "AWS::MediaConnect::Gateway")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "mediaconnect-owned-roots").await?;
    for operation in names { scenario.seed(plan(&catalog, "mediaconnect", operation)?).await?; }
    let gateway = "arn:aws:mediaconnect:us-east-1:123456789012:gateway:one";
    let mut seen = BTreeSet::new();
    for _ in 0..7 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(&catalog, "mediaconnect", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), second)));
        let body = if operation == "ListGateways" { json!({"gateways":[{"gatewayArn":gateway}]}) } else {
            let case = cases.iter().find(|case| case.0 == operation).ok_or("unexpected MediaConnect work")?;
            let native = native_input(&wire)?;
            assert!(!native.contains("gatewayArn="));
            let arn = format!("arn:aws:mediaconnect:us-east-1:123456789012:{}:{}", case.4, if second { "two" } else { "one" });
            let row = if operation == "ListGatewayInstances" { json!({(case.2):arn,"gatewayArn":gateway,"instanceId":"i-12345678"}) }
                else { json!({(case.2):arn,"resourceShareOwnership":"SELF"}) };
            page(json!({(case.1):[row]}), "nextToken", second)?
        };
        scenario.apply(&catalog, callable, lease, &request, body).await?;
    }
    for (operation, _, _, kind, resource) in cases {
        let expected: BTreeSet<_> = ["one", "two"].into_iter().map(|suffix| format!("arn:aws:mediaconnect:us-east-1:123456789012:{resource}:{suffix}")).collect();
        let edges = if operation == "ListGatewayInstances" { expected.iter().map(|id| (id.clone(), gateway.to_owned(), "IN_GATEWAY".to_owned())).collect() } else { BTreeSet::new() };
        assert_eq!(scenario.stored(kind).await?, (expected, edges));
        assert!(seen.contains(&(operation.to_owned(), true)));
    }
    Ok(())
}

#[tokio::test]
async fn given_macie_current_and_former_members_when_all_metadata_pages_resume_then_false_filter_is_native_and_all_owned_rows_persist() -> TestResult {
    let cases = [
        ("ListMembers", "members", "arn", "AWS::Macie::Member"),
        ("ListClassificationScopes", "classificationScopes", "id", "AWS::ApiOnly::MacieClassificationScope"),
        ("ListSensitivityInspectionTemplates", "sensitivityInspectionTemplates", "id", "AWS::ApiOnly::MacieSensitivityInspectionTemplate"),
    ];
    let names: Vec<_> = cases.iter().map(|case| case.0).collect();
    let bytes = compiled("macie2", "2020-01-01", &names, "AWS::Macie::Member")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "macie-member-and-configuration-pages").await?;
    for operation in names { scenario.seed(plan(&catalog, "macie2", operation)?).await?; }
    let mut seen = BTreeSet::new();
    for _ in 0..6 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let case = cases.iter().find(|case| case.0 == operation).ok_or("unexpected Macie work")?;
        let callable = plan(&catalog, "macie2", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), second)));
        let label = if second { "former" } else { "current" };
        let row = if operation == "ListMembers" {
            assert!(wire.uri().query().ok_or("member query absent")?.split('&').any(|pair| pair == "onlyAssociated=false"));
            json!({"arn":format!("arn:aws:macie2:us-east-1:123456789012:member/{label}"),"accountId":if second { "222222222222" } else { "111111111111" },"relationshipStatus":if second { "Removed" } else { "Enabled" }})
        } else { json!({"id":label,"name":label}) };
        scenario.apply(&catalog, callable, lease, &request, page(json!({(case.1):[row]}), "nextToken", second)?).await?;
    }
    for (operation, _, _, kind) in cases {
        let expected = ["current", "former"].into_iter().map(|label| if operation == "ListMembers" {
            format!("arn:aws:macie2:us-east-1:123456789012:member/{label}")
        } else { label.to_owned() }).collect();
        assert_eq!(scenario.stored(kind).await?, (expected, BTreeSet::new()));
        assert!(seen.contains(&(operation.to_owned(), true)));
    }
    Ok(())
}

#[tokio::test]
async fn given_both_proton_connection_roles_when_native_pages_overlap_then_independent_role_cursors_preserve_union() -> TestResult {
    let operation = "ListEnvironmentAccountConnections";
    let kind = "AWS::Proton::EnvironmentAccountConnection";
    let bytes = compiled("proton", "2020-07-20", &[operation], kind)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let callable = plan(&catalog, "proton", operation)?;
    let scenario = Scenario::new(&catalog, "proton-role-union").await?;
    scenario.seed(callable).await?;
    let mut seen = BTreeSet::new();
    for _ in 0..4 {
        let lease = scenario.claim().await?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        let role = text(&native, "requestedBy")?;
        assert!(["MANAGEMENT_ACCOUNT", "ENVIRONMENT_ACCOUNT"].contains(&role.as_str()));
        assert_cursor(&wire, second)?;
        assert!(seen.insert((role.clone(), second)));
        let label = if second { role.as_str() } else { "shared" };
        let body = json!({"environmentAccountConnections":[{"arn":format!("arn:aws:proton:us-east-1:123456789012:environment-account-connection/{label}")}]});
        scenario.apply(&catalog, callable, lease, &request, page(body, "nextToken", second)?).await?;
    }
    assert_eq!(seen.len(), 4);
    let expected = ["shared", "MANAGEMENT_ACCOUNT", "ENVIRONMENT_ACCOUNT"].into_iter().map(|id| format!("arn:aws:proton:us-east-1:123456789012:environment-account-connection/{id}")).collect();
    assert_eq!(scenario.stored(kind).await?, (expected, BTreeSet::new()));
    Ok(())
}

#[tokio::test]
async fn given_code_signing_configs_when_scalar_function_arn_pages_resume_then_lambdas_are_native_arn_resources() -> TestResult {
    let bytes = compiled("lambda", "2015-03-31", &["ListCodeSigningConfigs", "ListFunctionsByCodeSigningConfig"], "AWS::Lambda::CodeSigningConfig")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "lambda-code-signing-scalar-functions").await?;
    scenario.seed(plan(&catalog, "lambda", "ListCodeSigningConfigs")?).await?;
    let mut expected = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for _ in 0..5 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(&catalog, "lambda", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        let body = if operation == "ListCodeSigningConfigs" {
            json!({"CodeSigningConfigs":[{"CodeSigningConfigId":"csc-11111111111111111","CodeSigningConfigArn":"arn:aws:lambda:us-east-1:123456789012:code-signing-config:csc-11111111111111111","AllowedPublishers":{"SigningProfileVersionArns":[]},"CodeSigningPolicies":{"UntrustedArtifactOnDeployment":"Warn"},"LastModified":"2024-01-01T00:00:00.000+0000"},{"CodeSigningConfigId":"csc-22222222222222222","CodeSigningConfigArn":"arn:aws:lambda:us-east-1:123456789012:code-signing-config:csc-22222222222222222","AllowedPublishers":{"SigningProfileVersionArns":[]},"CodeSigningPolicies":{"UntrustedArtifactOnDeployment":"Warn"},"LastModified":"2024-01-01T00:00:00.000+0000"}]})
        } else if operation == "ListFunctionsByCodeSigningConfig" {
            let parent = bound(callable, &request)?;
            assert_parent(&wire, &parent)?;
            assert!(seen.insert((parent.clone(), second)));
            let label = if parent.ends_with('1') { "one" } else { "two" };
            let arn = format!("arn:aws:lambda:us-east-1:123456789012:function:{label}-{}", if second { "second" } else { "first" });
            expected.insert(arn.clone());
            page(json!({"FunctionArns":[arn]}), "NextMarker", second)?
        } else { return Err("unexpected Lambda work".into()) };
        scenario.apply(&catalog, callable, lease, &request, body).await?;
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(scenario.stored("AWS::Lambda::Function").await?, (expected, BTreeSet::new()));
    Ok(())
}

#[tokio::test]
async fn given_delegated_administrators_sharing_services_when_parent_bound_pages_resume_then_global_service_bindings_do_not_collapse() -> TestResult {
    let bytes = compiled("organizations", "2016-11-28", &["ListDelegatedAdministrators", "ListDelegatedServicesForAccount"], "AWS::ApiOnly::OrganizationsDelegatedAdministrator")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "organization-delegated-services").await?;
    scenario.seed(plan(&catalog, "organizations", "ListDelegatedAdministrators")?).await?;
    let mut expected = BTreeSet::new();
    let mut edges = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for _ in 0..5 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(&catalog, "organizations", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        let body = if operation == "ListDelegatedAdministrators" {
            json!({"DelegatedAdministrators":[
                {"Id":"111111111111","Arn":"arn:aws:organizations::123456789012:account/o-exampleorgid/111111111111"},
                {"Id":"222222222222","Arn":"arn:aws:organizations::123456789012:account/o-exampleorgid/222222222222"}
            ]})
        } else if operation == "ListDelegatedServicesForAccount" {
            let native: Json = serde_json::from_slice(wire.body())?;
            let account = text(&native, "AccountId")?;
            assert!(["111111111111", "222222222222"].contains(&account.as_str()));
            assert!(seen.insert((account.clone(), second)));
            let service = if second { "config.amazonaws.com" } else { "securityhub.amazonaws.com" };
            let id = format!("{}:{}|{}:{}", account.len(), account, service.len(), service);
            expected.insert(id.clone());
            edges.insert((id, format!("arn:aws:organizations::123456789012:account/o-exampleorgid/{account}"), "DELEGATED_FOR_ACCOUNT".to_owned()));
            page(json!({"DelegatedServices":[{"ServicePrincipal":service}]}), "NextToken", second)?
        } else { return Err("unexpected Organizations work".into()) };
        scenario.apply(&catalog, callable, lease, &request, body).await?;
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(scenario.stored("AWS::ApiOnly::OrganizationsDelegatedService").await?, (expected, edges));
    Ok(())
}

