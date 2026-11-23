use super::*;
#[tokio::test]
async fn given_two_knowledge_bases_when_summary_collections_execute_then_content_quickresponses_and_imports_have_distinct_durable_owners()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wisdom-completion").await?;
    scenario
        .seed(&catalog, "wisdom", &["ListKnowledgeBases"])
        .await?;
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
    assert_eq!(seen.len(), 4);
    for (kind, resource, prefix) in [
        ("AWS::Wisdom::Content", "content", "content"),
        ("AWS::Wisdom::QuickResponse", "quick-response", "response"),
    ] {
        let expected = ["kb-one", "kb-two"]
            .into_iter()
            .flat_map(|kb| {
                ["1", "2"].map(move |p| {
                    format!("arn:aws:wisdom:us-east-1:123456789012:{resource}/{kb}/{prefix}-{p}")
                })
            })
            .collect();
        scenario.assert_ids(kind, expected).await?;
    }
    let expected = ["kb-one", "kb-two"]
        .into_iter()
        .flat_map(|kb| ["1", "2"].map(move |p| correlated(&[kb, &format!("import-{p}")])))
        .collect();
    scenario
        .assert_ids("AWS::ApiOnly::WisdomImportJob", expected)
        .await?;
    for (kind, resource, prefix) in [
        ("AWS::Wisdom::Content", "content", "content"),
        ("AWS::Wisdom::QuickResponse", "quick-response", "response"),
        ("AWS::ApiOnly::WisdomImportJob", "", "import"),
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
async fn given_two_wickr_networks_when_bot_summary_pages_execute_then_challenge_boolean_and_parent_scoped_bot_ids_persist()
-> TestResult {
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
    let expected = ["11111111", "22222222"]
        .into_iter()
        .flat_map(|network| ["1", "2"].map(move |p| correlated(&[network, &format!("bot-{p}")])))
        .collect();
    scenario
        .assert_ids("AWS::ApiOnly::WickrBot", expected)
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WickrBot",
            "IN_NETWORK",
            ["11111111", "22222222"]
                .into_iter()
                .flat_map(|network| {
                    ["1", "2"].map(move |p| {
                        (
                            correlated(&[network, &format!("bot-{p}")]),
                            "AWS::Wickr::Network".to_owned(),
                            format!("arn:aws:wickr:us-east-1:123456789012:network/{network}"),
                        )
                    })
                })
                .collect(),
        )
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_two_voiceid_domains_when_summary_jobs_and_directory_pages_execute_then_all_five_account_owned_populations_persist_without_cross_domain_collapse()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-voiceid-completion").await?;
    scenario
        .seed(&catalog, "voice-id", &["ListDomains"])
        .await?;
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
    for (kind, prefix) in [
        ("VoiceIDWatchlist", "watchlist"),
        ("VoiceIDSpeaker", "speaker"),
        ("VoiceIDFraudster", "fraudster"),
        ("VoiceIDFraudsterRegistrationJob", "job"),
        ("VoiceIDSpeakerEnrollmentJob", "job"),
    ] {
        let expected = ["domain-one", "domain-two"]
            .into_iter()
            .flat_map(|domain| {
                ["1", "2"].map(move |p| correlated(&[domain, &format!("{prefix}-{p}")]))
            })
            .collect();
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), expected)
            .await?;
        scenario
            .assert_edges(
                &format!("AWS::ApiOnly::{kind}"),
                "IN_DOMAIN",
                ["domain-one", "domain-two"]
                    .into_iter()
                    .flat_map(|domain| {
                        ["1", "2"].map(move |p| {
                            (
                                correlated(&[domain, &format!("{prefix}-{p}")]),
                                "AWS::VoiceID::Domain".to_owned(),
                                format!("arn:aws:voiceid:us-east-1:123456789012:domain/{domain}"),
                            )
                        })
                    })
                    .collect(),
            )
            .await?;
    }
    Ok(())
}
