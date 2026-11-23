use super::*;
#[tokio::test]
async fn given_account_alias_observed_from_two_home_regions_when_collected_then_one_global_node_survives()
-> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iam", "ListAccountAliases")?;
    let mut scenario = Scenario::new(&catalog, "alias-east").await?;
    for (home, sweep) in [("us-east-1", "alias-east"), ("eu-west-1", "alias-west")] {
        scenario.home = RegionId::parse(home)?;
        scenario.sweep = inventory_engine::inventory::SweepId::new(sweep.to_owned())?;
        scenario.seed(plan).await?;
        let lease = scenario.claim("ListAccountAliases").await?;
        let (request, _) = scenario.request(plan, &lease)?;
        scenario
            .apply(
                &catalog,
                plan,
                lease,
                &request,
                &xml(
                    "ListAccountAliases",
                    "<AccountAliases/><IsTruncated>true</IsTruncated><Marker>resume2</Marker>",
                ),
            )
            .await?;
        scenario = scenario.reopen().await?;
        let lease = scenario.claim("ListAccountAliases").await?;
        let (request, wire) = scenario.request(plan, &lease)?;
        request_contains(&wire, "Marker=resume2")?;
        scenario.apply(&catalog, plan, lease, &request, &xml("ListAccountAliases",
            "<AccountAliases><member>example-corporation</member></AccountAliases><IsTruncated>false</IsTruncated>")).await?;
        assert!(
            scenario
                .store
                .claim(&scenario.sweep, 1000, 10000)
                .await?
                .is_none()
        );
    }
    let nodes = scenario
        .assert_nodes("AWS::ApiOnly::IAMAccountAlias", 1)
        .await?;
    assert_eq!(
        nodes,
        [(
            "123456789012".to_owned(),
            "global".to_owned(),
            "example-corporation".to_owned()
        )]
    );
    Ok(())
}

#[tokio::test]
async fn given_credential_only_for_another_user_when_all_users_pages_resume_then_only_metadata_and_user_edge_are_durable()
-> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iam", "ListServiceSpecificCredentials")?;
    let mut scenario = Scenario::new(&catalog, "service-credential-metadata").await?;
    scenario.seed(plan).await?;
    let lease = scenario.claim("ListServiceSpecificCredentials").await?;
    let (request, wire) = scenario.request(plan, &lease)?;
    request_contains(&wire, "AllUsers=true")?;
    let body = std::str::from_utf8(wire.body())?;
    assert!(!body.contains("UserName=") && !body.contains("ServiceName="));
    scenario.apply(&catalog, plan, lease, &request, &xml("ListServiceSpecificCredentials",
        "<ServiceSpecificCredentials/><IsTruncated>true</IsTruncated><Marker>resume2</Marker>")).await?;
    scenario = scenario.reopen().await?;
    let lease = scenario.claim("ListServiceSpecificCredentials").await?;
    let (request, wire) = scenario.request(plan, &lease)?;
    request_contains(&wire, "AllUsers=true")?;
    request_contains(&wire, "Marker=resume2")?;
    scenario.apply(&catalog, plan, lease, &request, &xml("ListServiceSpecificCredentials",
        "<ServiceSpecificCredentials><member><UserName>other-user</UserName><Status>Inactive</Status><ServiceUserName>other-user-at-123456789012</ServiceUserName><CreateDate>2026-01-01T00:00:00Z</CreateDate><ServiceSpecificCredentialId>ACCA1234567890EXAMPLE</ServiceSpecificCredentialId><ServiceName>codecommit.amazonaws.com</ServiceName></member></ServiceSpecificCredentials><IsTruncated>false</IsTruncated>")).await?;
    let nodes = scenario
        .assert_nodes("AWS::ApiOnly::IAMServiceSpecificCredential", 1)
        .await?;
    assert_eq!(
        nodes.first().ok_or("credential metadata absent")?.1,
        "global"
    );
    let reader = scenario.reader().await?;
    let references: Vec<(String, String, String)> = sqlx::query_as("SELECT target_kind,target_value,relation FROM unresolved_refs WHERE from_kind='AWS::ApiOnly::IAMServiceSpecificCredential'")
        .fetch_all(&reader).await?;
    assert_eq!(
        references,
        [(
            "AWS::IAM::User".to_owned(),
            "other-user".to_owned(),
            "HAS_USER".to_owned()
        )]
    );
    reader.close().await;
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .is_none()
    );
    Ok(())
}
