use super::*;

#[tokio::test]
async fn given_custom_lenses_and_agent_profiles_when_native_pages_execute_then_operation_endpoint_auth_scope_and_goal_owners_are_preserved() -> TestResult {
    use aws_wire::sign::{self, Body, Credentials, PathMode, PayloadHeader, Scope, SigningName};
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wellarchitected-completion").await?;
    scenario.seed(&catalog, "wellarchitected", &["ListLenses", "ListAgentProfiles"]).await?;
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
    assert_eq!(seen, BTreeMap::from([("ListLenses".to_owned(),2),("ListAgentProfiles".to_owned(),2),("ListAgentGoals".to_owned(),4)]));
    scenario.assert_ids("AWS::WellArchitected::Lens", ["1", "2"].map(|p| format!("arn:aws:wellarchitected:us-east-1:123456789012:lens/custom-{p}")).to_vec()).await?;
    scenario.assert_ids("AWS::WellArchitected::AgentProfile", ["1", "2"].map(|p| format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{p}")).to_vec()).await?;
    let goals = ["1", "2"].into_iter().flat_map(|profile| ["1", "2"].map(move |goal| correlated(&[&format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{profile}"), &format!("goal-{goal}")]))).collect();
    scenario.assert_ids("AWS::WellArchitected::AgentGoal", goals).await?;
    scenario.assert_edges("AWS::WellArchitected::AgentGoal","IN_PROFILE",
        ["1","2"].into_iter().flat_map(|profile| ["1","2"].map(move |goal| {
            let arn = format!("arn:aws:wellarchitected:us-east-1:123456789012:agent-profile/profile-{profile}");
            (correlated(&[&arn,&format!("goal-{goal}")]),"AWS::WellArchitected::AgentProfile".to_owned(),arn)
        })).collect()).await?;
    let reader = scenario.reader().await?;
    let scopes: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT region FROM resource_facets").fetch_all(&reader).await?;
    assert_eq!(scopes, [("us-east-1".to_owned(),)]);
    let tags: Vec<(String,String)> = sqlx::query_as("SELECT tag_key,tag_value FROM tags ORDER BY id").fetch_all(&reader).await?;
    assert_eq!(tags, [("owner".to_owned(),"inventory".to_owned()),("owner".to_owned(),"inventory".to_owned())]);
    reader.close().await;
    Ok(())
}

#[tokio::test]
async fn given_two_knowledge_bases_when_summary_collections_execute_then_content_quickresponses_and_imports_have_distinct_durable_owners() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wisdom-completion").await?;
    scenario.seed(&catalog, "wisdom", &["ListKnowledgeBases"]).await?;
    let (scenario, seen) = scenario.drain(&catalog, "wisdom", |plan, request, wire, resumed| {
        assert_token(wire, "nextToken", resumed)?;
        let p = if resumed { "2" } else { "1" };
        if plan.operation_name() == "ListKnowledgeBases" {
            return Ok(json!({"knowledgeBaseSummaries":[{"knowledgeBaseId":"kb-one","knowledgeBaseArn":"arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-one","name":"one","status":"ACTIVE","knowledgeBaseType":"CUSTOM","tags":{}},{"knowledgeBaseId":"kb-two","knowledgeBaseArn":"arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-two","name":"two","status":"ACTIVE","knowledgeBaseType":"CUSTOM","tags":{}}]}));
        }
        let kb = text(request, "knowledgeBaseId")?;
        assert!(matches!(kb, "kb-one" | "kb-two"));
        let (items, row, suffix) = match plan.operation_name() {
            "ListContents" => ("contentSummaries", json!({"knowledgeBaseId":kb,"knowledgeBaseArn":format!("arn:aws:wisdom:us-east-1:123456789012:knowledge-base/{kb}"),"contentId":format!("content-{p}"),"contentArn":format!("arn:aws:wisdom:us-east-1:123456789012:content/{kb}/content-{p}"),"name":"metadata-only","contentType":"text/plain","status":"ACTIVE","tags":{"owner":"inventory"}}), "contents"),
            "ListQuickResponses" => ("quickResponseSummaries", json!({"quickResponseId":format!("response-{p}"),"quickResponseArn":format!("arn:aws:wisdom:us-east-1:123456789012:quick-response/{kb}/response-{p}"),"knowledgeBaseId":kb,"name":"summary-only","contentType":"text/plain","status":"PUBLISHED","tags":{"owner":"inventory"}}), "quickResponses"),
            "ListImportJobs" => ("importJobSummaries", json!({"importJobId":format!("import-{p}"),"knowledgeBaseId":kb,"knowledgeBaseArn":format!("arn:aws:wisdom:us-east-1:123456789012:knowledge-base/{kb}"),"importJobType":"QUICK_RESPONSES","status":"COMPLETE"}), "importJobs"),
            other => return Err(format!("unexpected Wisdom operation {other}").into()),
        };
        assert_eq!(wire.uri().path(), format!("/knowledgeBases/{kb}/{suffix}"));
        Ok(page(items, row, !resumed, "nextToken"))
    }).await?;
    assert_eq!(seen.len(),4);
    for (kind, resource, prefix) in [("AWS::Wisdom::Content","content","content"),("AWS::Wisdom::QuickResponse","quick-response","response")] {
        let expected = ["kb-one","kb-two"].into_iter().flat_map(|kb| ["1","2"].map(move |p| format!("arn:aws:wisdom:us-east-1:123456789012:{resource}/{kb}/{prefix}-{p}"))).collect();
        scenario.assert_ids(kind, expected).await?;
    }
    let expected = ["kb-one","kb-two"].into_iter().flat_map(|kb| ["1","2"].map(move |p| correlated(&[kb, &format!("import-{p}")]))).collect();
    scenario.assert_ids("AWS::ApiOnly::WisdomImportJob", expected).await?;
    for (kind,resource,prefix) in [
        ("AWS::Wisdom::Content","content","content"),
        ("AWS::Wisdom::QuickResponse","quick-response","response"),
        ("AWS::ApiOnly::WisdomImportJob","","import"),
    ] {
        scenario.assert_edges(kind,"IN_KNOWLEDGE_BASE",
            ["kb-one","kb-two"].into_iter().flat_map(|kb| ["1","2"].map(move |p| {
                let from = if resource.is_empty() { correlated(&[kb,&format!("{prefix}-{p}")]) }
                    else { format!("arn:aws:wisdom:us-east-1:123456789012:{resource}/{kb}/{prefix}-{p}") };
                (from,"AWS::Wisdom::KnowledgeBase".to_owned(),format!("arn:aws:wisdom:us-east-1:123456789012:knowledge-base/{kb}"))
            })).collect()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_two_wickr_networks_when_bot_summary_pages_execute_then_challenge_boolean_and_parent_scoped_bot_ids_persist() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wickr-completion").await?;
    scenario.seed(&catalog, "wickr", &["ListNetworks"]).await?;
    let (scenario, _) = scenario.drain(&catalog, "wickr", |plan, request, wire, resumed| {
        assert_token(wire, "nextToken", resumed)?;
        if plan.operation_name() == "ListNetworks" {
            return Ok(json!({"networks":[{"networkId":"11111111","networkArn":"arn:aws:wickr:us-east-1:123456789012:network/11111111","networkName":"one"},{"networkId":"22222222","networkArn":"arn:aws:wickr:us-east-1:123456789012:network/22222222","networkName":"two"}]}));
        }
        assert_eq!(plan.operation_name(), "ListBots");
        let network = text(request, "networkId")?;
        assert!(matches!(network, "11111111" | "22222222"));
        assert!(!wire.uri().query().unwrap_or_default().contains("status="));
        assert_eq!(wire.uri().path(), format!("/networks/{network}/bots"));
        let p = if resumed { "2" } else { "1" };
        Ok(page("bots", json!({"botId":format!("bot-{p}"),"displayName":"metadata bot","username":"metadata","hasChallenge":true,"suspended":false}), !resumed, "nextToken"))
    }).await?;
    let expected = ["11111111","22222222"].into_iter().flat_map(|network| ["1","2"].map(move |p| correlated(&[network,&format!("bot-{p}")]))).collect();
    scenario.assert_ids("AWS::ApiOnly::WickrBot", expected).await?;
    scenario.assert_edges("AWS::ApiOnly::WickrBot","IN_NETWORK",
        ["11111111","22222222"].into_iter().flat_map(|network| ["1","2"].map(move |p|
            (correlated(&[network,&format!("bot-{p}")]),"AWS::Wickr::Network".to_owned(),
                format!("arn:aws:wickr:us-east-1:123456789012:network/{network}")))).collect()).await?;
    Ok(())
}

#[tokio::test]
async fn given_two_voiceid_domains_when_summary_jobs_and_directory_pages_execute_then_all_five_account_owned_populations_persist_without_cross_domain_collapse() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-voiceid-completion").await?;
    scenario.seed(&catalog, "voice-id", &["ListDomains"]).await?;
    let (scenario, seen) = scenario.drain(&catalog, "voice-id", |plan, request, wire, resumed| {
        let native: Json = serde_json::from_slice(wire.body())?;
        assert_token(wire, "NextToken", resumed)?;
        if plan.operation_name() == "ListDomains" {
            return Ok(json!({"DomainSummaries":[{"DomainId":"domain-one","Arn":"arn:aws:voiceid:us-east-1:123456789012:domain/domain-one","Name":"one"},{"DomainId":"domain-two","Arn":"arn:aws:voiceid:us-east-1:123456789012:domain/domain-two","Name":"two"}]}));
        }
        let domain = text(request, "DomainId")?;
        assert!(matches!(domain, "domain-one" | "domain-two"));
        assert_eq!(native.get("DomainId").and_then(Json::as_str), Some(domain));
        for filter in ["JobStatus", "WatchlistId"] { assert_eq!(native.get(filter), None, "{filter} must not narrow domain inventory"); }
        let p = if resumed { "2" } else { "1" };
        let (items, row) = match plan.operation_name() {
            "ListWatchlists" => ("WatchlistSummaries", json!({"DomainId":domain,"WatchlistId":format!("watchlist-{p}"),"Name":"watchlist"})),
            "ListSpeakers" => ("SpeakerSummaries", json!({"DomainId":domain,"GeneratedSpeakerId":format!("speaker-{p}"),"Status":"ENROLLED"})),
            "ListFraudsters" => ("FraudsterSummaries", json!({"DomainId":domain,"GeneratedFraudsterId":format!("fraudster-{p}")})),
            "ListFraudsterRegistrationJobs" => ("JobSummaries", json!({"DomainId":domain,"JobId":format!("job-{p}"),"JobStatus":"COMPLETED"})),
            "ListSpeakerEnrollmentJobs" => ("JobSummaries", json!({"DomainId":domain,"JobId":format!("job-{p}"),"JobStatus":"COMPLETED"})),
            other => return Err(format!("unexpected VoiceID operation {other}").into()),
        };
        Ok(page(items, row, !resumed, "NextToken"))
    }).await?;
    assert_eq!(seen.len(), 6);
    for (kind,prefix) in [("VoiceIDWatchlist","watchlist"),("VoiceIDSpeaker","speaker"),("VoiceIDFraudster","fraudster"),("VoiceIDFraudsterRegistrationJob","job"),("VoiceIDSpeakerEnrollmentJob","job")] {
        let expected = ["domain-one","domain-two"].into_iter().flat_map(|domain| ["1","2"].map(move |p| correlated(&[domain,&format!("{prefix}-{p}")]))).collect();
        scenario.assert_ids(&format!("AWS::ApiOnly::{kind}"), expected).await?;
        scenario.assert_edges(&format!("AWS::ApiOnly::{kind}"),"IN_DOMAIN",
            ["domain-one","domain-two"].into_iter().flat_map(|domain| ["1","2"].map(move |p|
                (correlated(&[domain,&format!("{prefix}-{p}")]),"AWS::VoiceID::Domain".to_owned(),
                    format!("arn:aws:voiceid:us-east-1:123456789012:domain/{domain}")))).collect()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_agent_profile_arns_outside_actual_request_authority_when_native_rows_are_applied_then_no_resource_or_child_is_committed() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    for (index, arn) in [
        "arn:aws:wellarchitected:us-east-1:999999999999:agent-profile/foreign",
        "arn:aws:wellarchitected:us-west-2:123456789012:agent-profile/wrong-region",
        "arn:aws-cn:wellarchitected:us-east-1:123456789012:agent-profile/wrong-partition",
    ].into_iter().enumerate() {
        let scenario = Scenario::new(&catalog, &format!("tz-agent-authority-{index}")).await?;
        scenario.seed(&catalog, "wellarchitected", &["ListAgentProfiles"]).await?;
        let lease = scenario.store.claim(&scenario.sweep, 1000, 10000).await?.ok_or("profile work absent")?;
        let plan = catalog.lookup("wellarchitected", "ListAgentProfiles").ok_or("profile plan absent")?;
        let (request, _) = scenario.request(plan, &lease)?;
        let error = scenario.apply(&catalog, plan, lease, &request,
            json!({"items":[{"name":"foreign","arn":arn,"executionRoleArn":"arn:aws:iam::123456789012:role/inventory","aggregationConfiguration":[],"createdBy":"inventory","createdAt":"2026-01-01T00:00:00Z"}]}))
            .await.err().ok_or("out-of-authority profile did not fail closed")?;
        assert!(error.to_string().contains("modeled ARN does not match trusted request scope"), "{error}");
        let reader = scenario.reader().await?;
        let (resources,): (i64,) = sqlx::query_as("SELECT count(*) FROM resource_facets").fetch_one(&reader).await?;
        assert_eq!(resources, 0);
        reader.close().await;
    }
    Ok(())
}
