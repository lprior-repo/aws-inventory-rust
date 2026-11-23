use super::*;
mod populations;

#[tokio::test]
async fn given_custom_lenses_and_agent_profiles_when_native_pages_execute_then_operation_endpoint_auth_scope_and_goal_owners_are_preserved()
-> TestResult {
    use aws_wire::sign::{self, Body, Credentials, PathMode, PayloadHeader, Scope, SigningName};
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wellarchitected-completion").await?;
    scenario
        .seed(
            &catalog,
            "wellarchitected",
            &["ListLenses", "ListAgentProfiles"],
        )
        .await?;
    let (scenario, seen) = scenario.drain(&catalog, "wellarchitected", |plan, request, wire, resumed| {
        let p = if resumed { "2" } else { "1" };
        let endpoint = routing::resolve(plan, &RegionId::parse("us-east-1")?)?;
        assert_eq!(endpoint.signing_name, "wellarchitected");
        assert_eq!(endpoint.signing_region, "us-east-1");
        assert_eq!(plan.scope(), PlanScope::Region);
        assert_eq!(wire.method(), http::Method::GET);
        if plan.operation_name() == "ListLenses" {
            assert_eq!(wire.uri().host(), Some("wellarchitected.us-east-1.amazonaws.com"));
            assert_eq!(wire.uri().path(), "/lenses");
            assert!(wire.uri().query().unwrap_or_default().split('&').any(|pair| pair == "LensType=CUSTOM_SELF"));
            assert_token(wire, "NextToken", resumed)?;
            return Ok(page("LensSummaries", json!({"LensArn":format!("arn:aws:wellarchitected:us-east-1:123456789012:lens/custom-{p}"),"LensAlias":format!("custom-{p}"),"LensName":"owned","LensType":"CUSTOM_SELF","Owner":"123456789012","LensStatus":"PUBLISHED"}), !resumed, "NextToken"));
        }
        assert_eq!(wire.uri().host(), Some("wellarchitected-agent.us-east-1.amazonaws.com"));
        let scope = Scope { region: sign::Region::new(&endpoint.signing_region)?, name: SigningName::new(&endpoint.signing_name)? };
        let uri = wire.uri().to_string();
        let signed = sign::sign(&Credentials::new("A", "secret", None)?, &scope, std::time::SystemTime::UNIX_EPOCH,
            sign::Request { method: wire.method(), uri: &uri, headers: wire.headers(), body: Body::Bytes(wire.body()), payload_header: PayloadHeader::Omit }, PathMode::Standard)?;
        assert!(signed.get("authorization").ok_or("agent signing authorization absent")?.to_str()?.contains("/us-east-1/wellarchitected/aws4_request,"));
        assert_token(wire, "nextToken", resumed)?;
        match plan.operation_name() {
            "ListAgentProfiles" => {
                assert_eq!(wire.uri().path(), "/api/v1/agent-profiles");
                Ok(page("items", json!({"name":format!("profile-{p}"),"arn":format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{p}"),"executionRoleArn":"arn:aws:iam::123456789012:role/inventory","aggregationConfiguration":[{"accountId":"123456789012","regions":["us-east-1"],"accessRoleArn":"arn:aws:iam::123456789012:role/inventory"}],"tags":[{"key":"owner","value":"inventory"}],"createdBy":"inventory","createdAt":"2026-01-01T00:00:00Z"}), !resumed, "nextToken"))
            }
            "ListAgentGoals" => {
                let profile = text(request, "profileArn")?;
                assert!(profile.ends_with("/profile-1") || profile.ends_with("/profile-2"));
                assert!(wire.uri().path().starts_with("/api/v1/agent-profiles/arn"));
                assert!(wire.uri().path().ends_with(&format!("{}/goals", profile.rsplit('/').next().ok_or("profile ID absent")?)));
                Ok(page("items", json!({"id":format!("goal-{p}"),"profileArn":profile,"title":"safe optimization goal","createdBy":"inventory","createdAt":"2026-01-01T00:00:00Z"}), !resumed, "nextToken"))
            }
            other => Err(format!("unexpected WellArchitected operation {other}").into()),
        }
    }).await?;
    assert_eq!(
        seen,
        BTreeMap::from([
            ("ListLenses".to_owned(), 2),
            ("ListAgentProfiles".to_owned(), 2),
            ("ListAgentGoals".to_owned(), 4)
        ])
    );
    scenario
        .assert_ids(
            "AWS::WellArchitected::Lens",
            ["1", "2"]
                .map(|p| format!("arn:aws:wellarchitected:us-east-1:123456789012:lens/custom-{p}"))
                .to_vec(),
        )
        .await?;
    scenario
        .assert_ids(
            "AWS::WellArchitected::AgentProfile",
            ["1", "2"]
                .map(|p| {
                    format!(
                        "arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{p}"
                    )
                })
                .to_vec(),
        )
        .await?;
    let goals = ["1", "2"].into_iter().flat_map(|profile| ["1", "2"].map(move |goal| correlated(&[&format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{profile}"), &format!("goal-{goal}")]))).collect();
    scenario
        .assert_ids("AWS::WellArchitected::AgentGoal", goals)
        .await?;
    scenario.assert_edges("AWS::WellArchitected::AgentGoal","IN_PROFILE",
        ["1","2"].into_iter().flat_map(|profile| ["1","2"].map(move |goal| {
            let arn = format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{profile}");
            (correlated(&[&arn,&format!("goal-{goal}")]),"AWS::WellArchitected::AgentProfile".to_owned(),arn)
        })).collect()).await?;
    let reader = scenario.reader().await?;
    let scopes: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT region FROM resource_facets")
        .fetch_all(&reader)
        .await?;
    assert_eq!(scopes, [("us-east-1".to_owned(),)]);
    let tags: Vec<(String, String)> =
        sqlx::query_as("SELECT tag_key,tag_value FROM tags ORDER BY id")
            .fetch_all(&reader)
            .await?;
    assert_eq!(
        tags,
        [
            ("owner".to_owned(), "inventory".to_owned()),
            ("owner".to_owned(), "inventory".to_owned())
        ]
    );
    reader.close().await;
    Ok(())
}

#[tokio::test]
async fn given_agent_profile_arns_outside_actual_request_authority_when_native_rows_are_applied_then_no_resource_or_child_is_committed()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    for (index, arn) in [
        "arn:aws:wellarchitected:us-east-1:999999999999:agent-profile/foreign",
        "arn:aws:wellarchitected:us-west-2:123456789012:agent-profile/wrong-region",
        "arn:aws-cn:wellarchitected:us-east-1:123456789012:agent-profile/wrong-partition",
    ]
    .into_iter()
    .enumerate()
    {
        let scenario = Scenario::new(&catalog, &format!("tz-agent-authority-{index}")).await?;
        scenario
            .seed(&catalog, "wellarchitected", &["ListAgentProfiles"])
            .await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("profile work absent")?;
        let plan = catalog
            .lookup("wellarchitected", "ListAgentProfiles")
            .ok_or("profile plan absent")?;
        let (request, _) = scenario.request(plan, &lease)?;
        let error = scenario.apply(&catalog, plan, lease, &request,
            json!({"items":[{"name":"foreign","arn":arn,"executionRoleArn":"arn:aws:iam::123456789012:role/inventory","aggregationConfiguration":[],"createdBy":"inventory","createdAt":"2026-01-01T00:00:00Z"}]}))
            .await.err().ok_or("out-of-authority profile did not fail closed")?;
        assert!(
            error
                .to_string()
                .contains("modeled ARN does not match trusted request scope"),
            "{error}"
        );
        let reader = scenario.reader().await?;
        let (resources,): (i64,) = sqlx::query_as("SELECT count(*) FROM resource_facets")
            .fetch_one(&reader)
            .await?;
        assert_eq!(resources, 0);
        reader.close().await;
    }
    Ok(())
}
