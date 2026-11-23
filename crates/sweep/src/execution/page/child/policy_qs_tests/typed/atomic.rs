use super::*;
#[tokio::test]
async fn given_a_prompt_version_page_with_an_invalid_base_prompt_arn_when_native_projection_runs_then_the_entire_page_is_rejected_before_sqlite()
-> TestResult {
    let catalog = fixture::catalog()?;
    let base = format!(
        "arn:aws:wisdom:us-east-1:123456789012:ai-prompt/{}/{}",
        ASSISTANTS[0], PROMPTS[0]
    );
    for invalid_arn in [
        format!("{base}:1"),
        base.replacen(":wisdom:", ":s3:", 1),
        base.replacen(":ai-prompt/", ":assistant/", 1),
        base.replacen(ASSISTANTS[0], "invalid-assistant-uuid", 1),
        base.replacen(PROMPTS[0], "invalid-prompt-uuid", 1),
    ] {
        let scenario = Scenario::new(&catalog, "invalid-prompt-base-arn", "us-east-1").await?;
        scenario
            .seed(fixture::plan(&catalog, "qconnect", "ListAssistants")?)
            .await?;
        let mut respond = |plan: &CallablePlan<'_>, _: &WorkLease, _: &Wire| match plan
            .operation_name()
        {
            "ListAssistants" => bytes(json!({"assistantSummaries":[assistant(ASSISTANTS[0])]})),
            "ListAIPrompts" => {
                bytes(json!({"aiPromptSummaries":[prompt(ASSISTANTS[0],PROMPTS[0])]}))
            }
            "ListAIPromptVersions" => {
                let mut invalid = prompt(ASSISTANTS[0], PROMPTS[0]);
                invalid
                    .as_object_mut()
                    .ok_or("native prompt fixture is not an object")?
                    .insert("aiPromptArn".to_owned(), json!(invalid_arn));
                bytes(json!({"aiPromptVersionSummaries":[
                        {"aiPromptSummary":prompt(ASSISTANTS[0],PROMPTS[0]),"versionNumber":1},
                        {"aiPromptSummary":invalid,"versionNumber":2}],"nextToken":"uncommitted-page-two"}))
            }
            _ => Err("unreviewed prompt ARN scenario operation".into()),
        };
        scenario.step(&catalog, &mut respond).await?;
        scenario.step(&catalog, &mut respond).await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .ok_or("native prompt version work is missing")?;
        let checkpoint = prompt_atomic_snapshot(&scenario).await?;
        let resources = scenario.saved().await?;
        let edges = scenario.edges().await?;
        let result = scenario.step_lease(&catalog, lease, &mut respond).await;
        assert!(
            result.is_err(),
            "invalid modeled base-prompt ARN accepted: {invalid_arn}"
        );
        let scenario = scenario.reopen(&catalog).await?;
        assert_eq!(
            prompt_atomic_snapshot(&scenario).await?,
            checkpoint,
            "failed version page changed its claimed checkpoint or references"
        );
        assert_eq!(scenario.saved().await?, resources);
        assert_eq!(scenario.edges().await?, edges);
        let saved = scenario.saved().await?;
        assert_eq!(count(&saved, "AWS::Wisdom::AIPrompt"), 1);
        assert_eq!(count(&saved, "AWS::Wisdom::AIPromptVersion"), 0);
        assert!(
            !scenario
                .edges()
                .await?
                .iter()
                .any(|(_, _, _, _, relation)| relation == "VERSION_OF")
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_a_customer_prompt_page_with_an_invalid_prompt_arn_when_native_projection_runs_then_no_prompt_or_descendant_checkpoint_is_committed()
-> TestResult {
    let catalog = fixture::catalog()?;
    let base = format!(
        "arn:aws:wisdom:us-east-1:123456789012:ai-prompt/{}/{}",
        ASSISTANTS[0], PROMPTS[1]
    );
    for invalid_arn in [
        format!("{base}:1"),
        base.replacen(":wisdom:", ":s3:", 1),
        base.replacen(":ai-prompt/", ":assistant/", 1),
        base.replacen(ASSISTANTS[0], "invalid-assistant-uuid", 1),
        base.replacen(PROMPTS[1], "invalid-prompt-uuid", 1),
    ] {
        let scenario = Scenario::new(&catalog, "invalid-customer-prompt-arn", "us-east-1").await?;
        scenario
            .seed(fixture::plan(&catalog, "qconnect", "ListAssistants")?)
            .await?;
        scenario
            .step(&catalog, &mut |plan, _, _| {
                assert_eq!(plan.operation_name(), "ListAssistants");
                bytes(json!({"assistantSummaries":[assistant(ASSISTANTS[0])]}))
            })
            .await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .ok_or("native customer prompt work is missing")?;
        let checkpoint = prompt_atomic_snapshot(&scenario).await?;
        let resources = scenario.saved().await?;
        let edges = scenario.edges().await?;
        let result=scenario.step_lease(&catalog,lease,&mut |plan,_,wire| {
            assert_eq!(plan.operation_name(),"ListAIPrompts");
            assert!(wire.uri().query().is_some_and(|query|query.contains("origin=CUSTOMER")));
            let mut invalid=prompt(ASSISTANTS[0],PROMPTS[1]);
            invalid.as_object_mut().ok_or("native prompt fixture is not an object")?.insert("aiPromptArn".to_owned(),json!(invalid_arn));
            bytes(json!({"aiPromptSummaries":[prompt(ASSISTANTS[0],PROMPTS[0]),invalid],"nextToken":"uncommitted-page-two"}))
        }).await;
        assert!(
            result.is_err(),
            "invalid modeled prompt ARN accepted: {invalid_arn}"
        );
        let scenario = scenario.reopen(&catalog).await?;
        assert_eq!(
            prompt_atomic_snapshot(&scenario).await?,
            checkpoint,
            "failed prompt page changed checkpoint, descendants or references"
        );
        assert_eq!(scenario.saved().await?, resources);
        assert_eq!(scenario.edges().await?, edges);
        let saved = scenario.saved().await?;
        assert_eq!(count(&saved, "AWS::Wisdom::AIPrompt"), 0);
        assert_eq!(count(&saved, "AWS::Wisdom::AIPromptVersion"), 0);
    }
    Ok(())
}

async fn prompt_atomic_snapshot(
    scenario: &Scenario<'_, '_>,
) -> TestResult<(Vec<String>, Vec<String>)> {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(scenario.directory.path().join("inventory.db"))
                .read_only(true),
        )
        .await?;
    let work=sqlx::query_scalar(
        "SELECT quote(sweep_id)||quote(scope_key)||quote(state)||quote(generation)||quote(lease_until)||quote(token)||quote(bindings)||quote(context) FROM work_queue ORDER BY sweep_id,scope_key")
        .fetch_all(&reader).await?;
    let references=sqlx::query_scalar(
        "SELECT quote(from_account)||quote(from_region)||quote(from_kind)||quote(from_id)||quote(target_scope)||quote(target_kind)||quote(target_path)||quote(target_value)||quote(relation)||quote(source)||quote(resolution) FROM unresolved_refs ORDER BY from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source")
        .fetch_all(&reader).await?;
    reader.close().await;
    Ok((work, references))
}
