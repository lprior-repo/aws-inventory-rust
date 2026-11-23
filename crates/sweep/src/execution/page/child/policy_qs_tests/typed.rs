use super::*;
mod atomic;

const ASSISTANTS: [&str; 2] = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
];
const PROMPTS: [&str; 2] = [
    "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
];

#[tokio::test]
async fn given_empty_and_multicomponent_namespaces_in_two_buckets_when_pages_execute_then_sequences_and_bucket_provenance_do_not_collide()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "table-namespaces-including-empty", "us-east-1").await?;
    scenario
        .seed(fixture::plan(&catalog, "s3tables", "ListTableBuckets")?)
        .await?;
    scenario.run(&catalog,5,|plan,lease,wire| {
        if plan.operation_name()=="ListTableBuckets" {
            return bytes(json!({"tableBuckets":[{"arn":bucket("a"),"name":"a","ownerAccountId":"123456789012","createdAt":"1970-01-01T00:00:00Z"},{"arn":bucket("b"),"name":"b","ownerAccountId":"123456789012","createdAt":"1970-01-01T00:00:00Z"}]}));
        }
        assert!(wire.uri().path().starts_with("/namespaces/arn%3Aaws%3As3tables%3Aus-east-1%3A123456789012%3Abucket%2F"));
        assert!(!wire.uri().query().is_some_and(|query|query.contains("prefix=")));
        let second=lease.token().is_some();
        let names=if second {vec![vec!["empty"]]} else {vec![vec!["a","b_c"],vec!["a_b","c"]]};
        paginated(json!({"namespaces":names.into_iter().map(|namespace|json!({"namespace":namespace,"createdAt":"1970-01-01T00:00:00Z","createdBy":"123456789012","ownerAccountId":"123456789012"})).collect::<Vec<_>>()}),"continuationToken",second)
    }).await?;
    let saved = scenario.saved().await?;
    assert_eq!(count(&saved, "AWS::S3Tables::Namespace"), 6);
    assert_eq!(
        saved
            .iter()
            .filter(|(kind, _, _)| kind == "AWS::S3Tables::Namespace")
            .map(|(_, id, _)| id)
            .collect::<BTreeSet<_>>()
            .len(),
        6
    );
    assert_eq!(
        scenario
            .edges()
            .await?
            .iter()
            .filter(
                |(kind, _, _, _, relation)| kind == "AWS::S3Tables::Namespace"
                    && relation == "IN_TABLE_BUCKET"
            )
            .count(),
        6
    );
    Ok(())
}

#[tokio::test]
async fn given_two_assistants_when_customer_prompt_versions_page_then_inherited_assistant_and_long_version_identity_survive_sqlite()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "customer-prompt-versions", "us-east-1").await?;
    scenario
        .seed(fixture::plan(&catalog, "qconnect", "ListAssistants")?)
        .await?;
    let mut requests = BTreeSet::new();
    scenario.run(&catalog,13,|plan,lease,wire| {
        let operation=plan.operation_name();
        if operation=="ListAssistants" {return bytes(json!({"assistantSummaries":[assistant(ASSISTANTS[0]),assistant(ASSISTANTS[1])]}));}
        let path=wire.uri().path();
        let assistant=if path.contains(ASSISTANTS[0]) {ASSISTANTS[0]} else {ASSISTANTS[1]};
        assert!(path.starts_with(&format!("/assistants/{assistant}/aiprompts")));
        assert!(wire.uri().query().is_some_and(|query|query.contains("origin=CUSTOMER")));
        let second=lease.token().is_some();
        assert!(requests.insert((operation.to_owned(),path.to_owned(),second)));
        let value=if operation=="ListAIPrompts" {
            json!({"aiPromptSummaries":[prompt(assistant,if second {PROMPTS[1]} else {PROMPTS[0]})]})
        } else {
            assert_eq!(operation,"ListAIPromptVersions");
            let id=if path.contains(&format!("{}/",PROMPTS[0])) {PROMPTS[0]} else {PROMPTS[1]};
            json!({"aiPromptVersionSummaries":[{"aiPromptSummary":prompt(assistant,id),"versionNumber":if second {2147483648_i64} else {1_i64}}]})
        };
        paginated(value,"nextToken",second)
    }).await?;
    assert_eq!(requests.len(), 12);
    let scenario = scenario.reopen(&catalog).await?;
    let saved = scenario.saved().await?;
    assert_eq!(count(&saved, "AWS::Wisdom::AIPrompt"), 4);
    assert_eq!(count(&saved, "AWS::Wisdom::AIPromptVersion"), 8);
    assert_eq!(
        saved
            .iter()
            .filter(
                |(kind, id, _)| kind == "AWS::Wisdom::AIPromptVersion" && id.contains("2147483648")
            )
            .count(),
        4
    );
    let edges = scenario.edges().await?;
    assert_eq!(
        edges
            .iter()
            .filter(
                |(kind, _, _, _, relation)| kind == "AWS::Wisdom::AIPromptVersion"
                    && relation == "VERSION_OF"
            )
            .count(),
        8
    );
    for assistant in ASSISTANTS {
        for prompt in PROMPTS {
            let expected =
                format!("arn:aws:wisdom:us-east-1:123456789012:ai-prompt/{assistant}/{prompt}");
            assert!(
                saved
                    .iter()
                    .any(|(kind, id, _)| kind == "AWS::Wisdom::AIPrompt" && id == &expected)
            );
            let mut matched = 0;
            for (_, source, target_kind, target, relation) in &edges {
                if relation == "VERSION_OF" && source.contains(&expected) {
                    assert_eq!(target_kind, "AWS::Wisdom::AIPrompt");
                    assert_eq!(target, &expected);
                    matched += 1;
                }
            }
            assert_eq!(matched, 2);
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_two_region_lakes_when_singleton_region_queries_execute_then_each_arn_is_stored_only_under_its_actual_region()
-> TestResult {
    let catalog = fixture::catalog()?;
    for region in ["us-east-1", "eu-west-1"] {
        let scenario = Scenario::new(&catalog, &format!("securitylake-{region}"), region).await?;
        scenario
            .seed(fixture::plan(&catalog, "securitylake", "ListDataLakes")?)
            .await?;
        scenario.run(&catalog,1,|_,_,wire| {
            assert_eq!(wire.uri().query(),Some(format!("regions={region}").as_str()));
            bytes(json!({"dataLakes":[{"dataLakeArn":format!("arn:aws:securitylake:{region}:123456789012:data-lake/default"),"region":region,"s3BucketArn":"arn:aws:s3:::owned-security-lake"}]}))
        }).await?;
        let saved = scenario.saved().await?;
        assert_eq!(
            saved,
            vec![(
                "AWS::SecurityLake::DataLake".to_owned(),
                format!("arn:aws:securitylake:{region}:123456789012:data-lake/default"),
                region.to_owned()
            )]
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_a_foreign_region_or_account_lake_when_native_rows_arrive_then_the_entire_page_is_rejected_before_sqlite()
-> TestResult {
    let catalog = fixture::catalog()?;
    for arn in [
        "arn:aws:securitylake:eu-west-1:123456789012:data-lake/default",
        "arn:aws:securitylake:us-east-1:999999999999:data-lake/default",
        "arn:aws-cn:securitylake:us-east-1:123456789012:data-lake/default",
    ] {
        let scenario = Scenario::new(&catalog, "invalid-securitylake-scope", "us-east-1").await?;
        scenario
            .seed(fixture::plan(&catalog, "securitylake", "ListDataLakes")?)
            .await?;
        let result=scenario.step(&catalog,&mut |_,_,_|bytes(json!({"dataLakes":[
            {"dataLakeArn":"arn:aws:securitylake:us-east-1:123456789012:data-lake/valid","region":"us-east-1"},
            {"dataLakeArn":arn,"region":"us-east-1"}]}))).await;
        assert!(result.is_err(), "untrusted ARN accepted: {arn}");
        assert!(scenario.saved().await?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn given_two_portfolios_with_iam_and_pattern_principals_when_all_constraint_types_page_then_associations_and_five_canonical_subtypes_survive()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(
        &catalog,
        "portfolio-constraints-and-principals",
        "us-east-1",
    )
    .await?;
    scenario
        .seed(fixture::plan(
            &catalog,
            "service-catalog",
            "ListPortfolios",
        )?)
        .await?;
    scenario.run(&catalog,9,|plan,lease,wire| {
        if plan.operation_name()=="ListPortfolios" {return bytes(json!({"PortfolioDetails":[{"Id":"port-a","ARN":"arn:aws:catalog:us-east-1:123456789012:portfolio/port-a"},{"Id":"port-b","ARN":"arn:aws:catalog:us-east-1:123456789012:portfolio/port-b"}]}));}
        let request=body(wire)?;
        let portfolio=text(&request,"PortfolioId")?;
        assert!(matches!(text(&request,"PortfolioId")?,"port-a"|"port-b"));
        assert!(request.get("ProductId").is_none());
        let second=lease.token().is_some();
        assert_eq!(request.get("PageToken").and_then(Json::as_str),second.then_some("page-two"));
        let value=if plan.operation_name()=="ListPrincipalsForPortfolio" {
            json!({"Principals":[{"PrincipalARN":if second {"arn:aws:iam:::role/team-*"} else {"arn:aws:iam::123456789012:role/inventory"},"PrincipalType":if second {"IAM_PATTERN"} else {"IAM"}}]})
        } else {
            assert_eq!(plan.operation_name(),"ListConstraintsForPortfolio");
            let kinds:&[&str]=if second {&["STACKSET","TEMPLATE"]} else {&["LAUNCH","NOTIFICATION","RESOURCE_UPDATE"]};
            json!({"ConstraintDetails":kinds.iter().map(|kind|json!({"ConstraintId":format!("same-{kind}"),"Type":kind,"Owner":"123456789012","PortfolioId":portfolio})).collect::<Vec<_>>()})
        };
        paginated(value,"NextPageToken",second)
    }).await?;
    let saved = scenario.saved().await?;
    assert_eq!(
        count(&saved, "AWS::ServiceCatalog::PortfolioPrincipalAssociation"),
        4
    );
    for kind in [
        "LaunchRoleConstraint",
        "LaunchNotificationConstraint",
        "ResourceUpdateConstraint",
        "StackSetConstraint",
        "LaunchTemplateConstraint",
    ] {
        assert_eq!(count(&saved, &format!("AWS::ServiceCatalog::{kind}")), 2);
    }
    assert_eq!(
        scenario
            .edges()
            .await?
            .iter()
            .filter(|(_, _, _, _, relation)| relation == "FOR_PORTFOLIO")
            .count(),
        14
    );
    assert!(
        !saved
            .iter()
            .any(|(kind, _, _)| kind.starts_with("AWS::IAM::"))
    );
    Ok(())
}

fn bucket(id: &str) -> String {
    format!("arn:aws:s3tables:us-east-1:123456789012:bucket/{id}")
}
fn assistant(id: &str) -> Json {
    json!({"assistantId":id,"assistantArn":format!("arn:aws:wisdom:us-east-1:123456789012:assistant/{id}"),"name":id,"type":"AGENT","status":"ACTIVE"})
}
fn prompt(assistant: &str, id: &str) -> Json {
    json!({"name":id,"assistantId":assistant,"assistantArn":format!("arn:aws:wisdom:us-east-1:123456789012:assistant/{assistant}"),"aiPromptId":id,"aiPromptArn":format!("arn:aws:wisdom:us-east-1:123456789012:ai-prompt/{assistant}/{id}"),"type":"ANSWER_GENERATION","templateType":"TEXT","modelId":"anthropic.claude-3-haiku-20240307-v1:0","apiFormat":"ANTHROPIC_CLAUDE_MESSAGES","visibilityStatus":"PUBLISHED","origin":"CUSTOMER","tags":{}})
}
