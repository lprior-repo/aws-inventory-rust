use super::*;

// Pin: quicksight/service/2018-04-01/quicksight-2018-04-01.json.
// These summary closures contain identifiers/configuration, not definitions, credentials or rows.
// Topic and TopicV2 have separate create/read contracts; preserve both canonical kinds.
pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    [
        ("ListActionConnectors", "ActionConnectorSummaries", "Arn", "AwsAccountId", "NextToken"),
        ("ListAgents", "AgentSummaries", "Arn", "AwsAccountId", "NextToken"),
        ("ListApps", "AppSummaryList", "Arn", "AwsAccountId", "NextToken"),
        ("ListBrands", "Brands", "Arn", "AwsAccountId", "NextToken"),
        ("ListDlpSettings", "DlpSettingSummaries", "Arn", "AwsAccountId", "NextToken"),
        ("ListFlows", "FlowSummaryList", "Arn", "AwsAccountId", "NextToken"),
        ("ListLimitsProfiles", "profiles", "arn", "accountId", "nextToken"),
        ("ListOAuthClientApplications", "OAuthClientApplications", "Arn", "AwsAccountId", "NextToken"),
        ("ListSpaces", "SpaceSummaries", "spaceArn", "AwsAccountId", "NextToken"),
        ("ListTemplates", "TemplateSummaryList", "Arn", "AwsAccountId", "NextToken"),
        ("ListTopics", "TopicsSummaries", "Arn", "AwsAccountId", "NextToken"),
        ("ListTopicsV2", "TopicSummaryList", "Arn", "AwsAccountId", "NextToken"),
    ].into_iter().try_for_each(|(operation, items, field, account, token)| {
        let mut plan = arn(items, field, Some((token, token)))?;
        plan.bind.insert(path(account)?, Binding::Account);
        put(service, operation, plan)
    })?;
    groups(service)?;
    schedules(service)
}

fn groups(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = arn("GroupList", "Arn", Some(("NextToken", "NextToken")))?;
    parent(&mut plan, "Namespace", "ListNamespaces", "Name")?;
    plan.bind.insert(path("AwsAccountId")?, Binding::Account);
    put(service, "ListGroups", plan)
}

fn schedules(service: &mut ServiceOverlay) -> Result<(), NameError> {
    // Arn is optional; ScheduleId is local to its dataset. No pagination exists in this model.
    let mut plan = qualified("RefreshSchedules", &["DataSetId"], &["ScheduleId"], None)?;
    parent(&mut plan, "DataSetId", "ListDataSets", "DataSetId")?;
    plan.bind.insert(path("AwsAccountId")?, Binding::Account);
    put(service, "ListRefreshSchedules", plan)
}
