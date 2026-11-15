use super::*;

const LISTS: &[(&str, &str, &str, &str, &str)] = &[
    ("ListActionConnectors", "ActionConnectorSummaries", "Arn", "actionconnector", "AWS::QuickSight::ActionConnector"),
    ("ListAgents", "AgentSummaries", "Arn", "agent", "AWS::QuickSight::Agent"),
    ("ListApps", "AppSummaryList", "Arn", "app", "AWS::ApiOnly::QuickSightApp"),
    ("ListBrands", "Brands", "Arn", "brand", "AWS::QuickSight::Brand"),
    ("ListDlpSettings", "DlpSettingSummaries", "Arn", "dlpsetting", "AWS::QuickSight::DLPSetting"),
    ("ListFlows", "FlowSummaryList", "Arn", "flow", "AWS::QuickSight::Flow"),
    ("ListLimitsProfiles", "profiles", "arn", "limitsprofile", "AWS::QuickSight::LimitsProfile"),
    ("ListOAuthClientApplications", "OAuthClientApplications", "Arn", "oauthclientapplication", "AWS::QuickSight::OAuthClientApplication"),
    ("ListSpaces", "SpaceSummaries", "spaceArn", "space", "AWS::QuickSight::Space"),
    ("ListTemplates", "TemplateSummaryList", "Arn", "template", "AWS::QuickSight::Template"),
    ("ListTopics", "TopicsSummaries", "Arn", "topic", "AWS::QuickSight::Topic"),
    ("ListTopicsV2", "TopicSummaryList", "Arn", "topic", "AWS::QuickSight::TopicV2"),
    ("ListApprovalPolicies", "Policies", "PolicyArn", "approvalpolicy", "AWS::QuickSight::ApprovalPolicy"),
];

#[tokio::test]
async fn given_owned_quicksight_metadata_and_colliding_child_ids_when_native_pages_resume_then_all_canonical_resources_persist() -> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "qs-account-metadata", "us-east-1").await?;
    for (operation, _, _, _, _) in LISTS { scenario.seed(fixture::plan(&catalog, "quicksight", operation)?).await?; }
    for operation in ["ListNamespaces", "ListDataSets"] { scenario.seed(fixture::plan(&catalog, "quicksight", operation)?).await?; }
    let mut seen = BTreeSet::new();
    scenario.run(&catalog, 34, |plan, lease, wire| {
        let operation = plan.operation_name();
        let second = lease.token().is_some();
        let uri = wire.uri().to_string();
        if operation == "ListApprovalPolicies" {
            assert!(wire.uri().path().ends_with("/governance/approvalworkflows/policies"));
            assert!(!uri.contains("123456789012"));
        } else { assert!(wire.uri().path().contains("/123456789012/")); }
        if second { assert!(uri.contains("page-two")); }
        assert!(seen.insert((operation.to_owned(), wire.uri().path().to_owned(), second)));
        response(operation, second, wire)
    }).await?;
    let saved = scenario.saved().await?;
    for (_, _, _, resource, kind) in LISTS {
        assert_eq!(count(&saved, kind), 2, "missing {kind}");
        assert!(saved.iter().filter(|(saved, _, _)| saved == kind).all(|(_, id, region)|
            id.starts_with(&format!("arn:aws:quicksight:us-east-1:123456789012:{resource}/")) && region == "us-east-1"));
    }
    assert_eq!(count(&saved, "AWS::QuickSight::Group"), 4);
    assert_eq!(count(&saved, "AWS::QuickSight::RefreshSchedule"), 10);
    assert_eq!(count(&saved, "AWS::ApiOnly::QuickSightApprovalPolicy"), 0);
    assert_eq!(saved.iter().filter(|(kind, _, _)| kind == "AWS::QuickSight::RefreshSchedule").map(|(_, id, _)| id).collect::<BTreeSet<_>>().len(), 10);
    Ok(())
}

fn response(operation: &str, second: bool, wire: &Wire) -> TestResult<Vec<u8>> {
    let page = if second { "b" } else { "a" };
    match operation {
        "ListNamespaces" => bytes(json!({"Namespaces":[{"Name":"team-a","Arn":arn("namespace", "team-a")},{"Name":"team-b","Arn":arn("namespace", "team-b")}]})),
        "ListDataSets" => bytes(json!({"DataSetSummaries":[{"DataSetId":"data-a","Arn":arn("dataset", "data-a")},{"DataSetId":"data-b","Arn":arn("dataset", "data-b")}]})),
        "ListGroups" => {
            let namespace = if wire.uri().path().contains("team-a") { "team-a" } else { "team-b" };
            paginated(json!({"GroupList":[{"GroupName":page,"Arn":arn("group", &format!("{namespace}/{page}")),"PrincipalId":format!("{namespace}-{page}")}]}), "NextToken", second)
        }
        "ListRefreshSchedules" => {
            assert!(wire.uri().query().is_none());
            bytes(json!({"RefreshSchedules":(0..5).map(|id| json!({"ScheduleId":format!("same-{id}"),"ScheduleFrequency":{"Interval":"DAILY","TimeOfTheDay":"01:00","Timezone":"UTC"},"StartAfterDateTime":0,"RefreshType":"FULL_REFRESH"})).collect::<Vec<_>>()}))
        }
        _ => {
            let (_, items, field, resource, _) = LISTS.iter().find(|(name, _, _, _, _)| *name == operation).ok_or("unreviewed QuickSight call")?;
            let mut row = json!({(*field):arn(resource, page)});
            match operation {
                "ListDlpSettings" => row = json!({"DlpSettingId":page,"Name":page,"Arn":arn(resource,page),"Status":"ACTIVE","ProviderType":"MICROSOFT_PURVIEW","CreatedAt":0,"UpdatedAt":0}),
                "ListLimitsProfiles" => row = json!({"profileId":page,"arn":arn(resource,page),"accountId":"123456789012","profileName":page,"resourceLimits":{},"createdAt":0,"updatedAt":0}),
                "ListApprovalPolicies" => row = json!({"PolicyId":page,"PolicyArn":arn(resource,page),"Name":page,"Actions":[],"AssetTypes":[],"ApplicableTo":{},"ApprovalGroups":[],"CreatedAt":0,"UpdatedAt":0}),
                _ => {}
            }
            // An empty nonterminal page must still schedule its continuation.
            let rows = if operation == "ListAgents" && !second { vec![] }
                else if operation == "ListAgents" { vec![json!({"Arn":arn(resource,"a")}), row] } else { vec![row] };
            paginated(json!({(*items):rows}), if operation == "ListLimitsProfiles" { "nextToken" } else { "NextToken" }, second)
        }
    }
}
fn arn(resource: &str, id: &str) -> String { format!("arn:aws:quicksight:us-east-1:123456789012:{resource}/{id}") }
