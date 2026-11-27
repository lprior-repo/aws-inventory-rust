use super::*;
#[tokio::test]
async fn given_mixed_knowledge_bases_when_completion_runs_then_only_quick_response_imports_are_collected()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-wisdom-completion").await?;
    scenario.seed(&catalog, "wisdom", &["ListKnowledgeBases"]).await?;
    let (scenario, seen) = scenario.drain(&catalog, "wisdom", |plan, request, wire, resumed| {
        assert_token(wire, "nextToken", resumed)?;
        if plan.operation_name() == "ListKnowledgeBases" {
            return Ok(json!({"knowledgeBaseSummaries":[{"knowledgeBaseId":"kb-one","knowledgeBaseArn":"arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-one","name":"quick responses","status":"ACTIVE","knowledgeBaseType":"QUICK_RESPONSES","tags":{}},{"knowledgeBaseId":"kb-two","knowledgeBaseArn":"arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-two","name":"custom","status":"ACTIVE","knowledgeBaseType":"CUSTOM","tags":{}}]}));
        }
        let kb = text(request, "knowledgeBaseId")?;
        assert_eq!(kb, "kb-one");
        let p = if resumed { "2" } else { "1" };
        assert_eq!(plan.operation_name(), "ListImportJobs");
        assert_eq!(wire.uri().path(), format!("/knowledgeBases/{kb}/importJobs"));
        Ok(page(
            "importJobSummaries",
            json!({"importJobId":format!("import-{p}"),"knowledgeBaseId":kb,"knowledgeBaseArn":format!("arn:aws:wisdom:us-east-1:123456789012:knowledge-base/{kb}"),"importJobType":"QUICK_RESPONSES","status":"COMPLETE"}),
            !resumed,
            "nextToken",
        ))
    }).await?;
    assert_eq!(seen.len(), 2);
    scenario
        .assert_ids(
            "AWS::Wisdom::KnowledgeBase",
            [
                "arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-one",
                "arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-two",
            ]
            .map(str::to_owned)
            .into(),
        )
        .await?;
    assert_eq!(seen.get("ListImportJobs"), Some(&2));
    assert!(!seen.contains_key("ListContents"));
    assert!(!seen.contains_key("ListQuickResponses"));
    scenario.assert_ids("AWS::Wisdom::Content", Vec::new()).await?;
    scenario.assert_ids("AWS::Wisdom::QuickResponse", Vec::new()).await?;
    let expected = ["1", "2"]
        .map(|p| correlated(&["kb-one", &format!("import-{p}")]))
        .into_iter()
        .collect();
    scenario
        .assert_ids("AWS::ApiOnly::WisdomImportJob", expected)
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WisdomImportJob",
            "IN_KNOWLEDGE_BASE",
            ["1", "2"]
                .map(|p| {
                    (
                        correlated(&["kb-one", &format!("import-{p}")]),
                        "AWS::Wisdom::KnowledgeBase".to_owned(),
                        "arn:aws:wisdom:us-east-1:123456789012:knowledge-base/kb-one"
                            .to_owned(),
                    )
                })
                .into_iter()
                .collect(),
        )
        .await?;
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
async fn given_voice_id_domains_when_swept_then_denied_directory_operations_are_never_called()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-voiceid-completion").await?;
    scenario
        .seed(&catalog, "voice-id", &["ListDomains"])
        .await?;
    let (scenario, seen) = scenario.drain(&catalog, "voice-id", |plan, _, _, _| {
        assert_eq!(plan.operation_name(), "ListDomains");
        Ok(json!({"DomainSummaries":[
            {"DomainId":"domain-one","Arn":"arn:aws:voiceid:us-east-1:123456789012:domain/domain-one","Name":"one"},
            {"DomainId":"domain-two","Arn":"arn:aws:voiceid:us-east-1:123456789012:domain/domain-two","Name":"two"}
        ]}))
    }).await?;
    assert_eq!(seen.get("ListDomains"), Some(&1));
    scenario.assert_ids(
        "AWS::VoiceID::Domain",
        ["domain-one", "domain-two"].map(|id| format!("arn:aws:voiceid:us-east-1:123456789012:domain/{id}")).into(),
    ).await?;
    for (operation, kind) in [
        ("ListWatchlists", "VoiceIDWatchlist"),
        ("ListSpeakers", "VoiceIDSpeaker"),
        ("ListFraudsters", "VoiceIDFraudster"),
        ("ListFraudsterRegistrationJobs", "VoiceIDFraudsterRegistrationJob"),
        ("ListSpeakerEnrollmentJobs", "VoiceIDSpeakerEnrollmentJob"),
    ] {
        assert!(catalog.lookup("voice-id", operation).is_none(), "{operation} is denied");
        scenario.assert_ids(&format!("AWS::ApiOnly::{kind}"), Vec::new()).await?;
    }
    Ok(())
}
