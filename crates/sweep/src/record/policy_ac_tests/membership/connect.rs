use super::*;

const SPEC: [(&str, &str, &str, &str, &str); 5] = [
    (
        "ListAgentStatuses",
        "AWS::Connect::AgentStatus",
        "AgentStatusSummaryList",
        "agent-status",
        "agent-state",
    ),
    (
        "ListQuickConnects",
        "AWS::Connect::QuickConnect",
        "QuickConnectSummaryList",
        "quick-connects",
        "transfer-destination",
    ),
    (
        "ListUserHierarchyGroups",
        "AWS::Connect::UserHierarchyGroup",
        "UserHierarchyGroupSummaryList",
        "user-hierarchy-groups-summary",
        "agent-group",
    ),
    (
        "ListUsers",
        "AWS::Connect::User",
        "UserSummaryList",
        "users-summary",
        "agent",
    ),
    (
        "ListViews",
        "AWS::Connect::View",
        "ViewsSummaryList",
        "views",
        "view",
    ),
];

fn instance(owner: &str) -> Json {
    json!({"Id":owner,"Arn":format!("arn:aws:connect:us-east-1:123456789012:instance/{owner}"),
        "InstanceAlias":"same","IdentityManagementType":"CONNECT_MANAGED","InstanceStatus":"ACTIVE",
        "CreatedTime":1767225600,"InboundCallsEnabled":true,"OutboundCallsEnabled":false})
}

fn case(
    spec: (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
) -> TestResult<Case> {
    let (operation, kind, items, endpoint, resource) = spec;
    let owner = |owner: &str| -> TestResult<Owner> {
        let page = |later: bool| -> TestResult<Page> {
            let id = if later {
                "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
            } else {
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
            };
            let arn =
                format!("arn:aws:connect:us-east-1:123456789012:instance/{owner}/{resource}/{id}");
            let mut row = json!({"Id":id,"Arn":arn});
            let object = row.as_object_mut().ok_or("Connect summary object absent")?;
            if operation == "ListUsers" {
                object.insert("Username".to_owned(), json!("same"));
            } else {
                object.insert("Name".to_owned(), json!("same"));
            }
            match operation {
                "ListAgentStatuses" => {
                    object.insert("Type".to_owned(), json!("CUSTOM"));
                }
                "ListQuickConnects" => {
                    object.insert("QuickConnectType".to_owned(), json!("USER"));
                }
                "ListViews" => {
                    object.insert("Type".to_owned(), json!("CUSTOMER_MANAGED"));
                    object.insert("Status".to_owned(), json!("PUBLISHED"));
                    object.insert("Description".to_owned(), json!("metadata only"));
                }
                _ => {}
            }
            if operation != "ListViews" {
                object.insert("LastModifiedTime".to_owned(), json!(1767312000));
                object.insert("LastModifiedRegion".to_owned(), json!("us-east-1"));
            }
            let mut query = match operation {
                "ListAgentStatuses" => vec!["AgentStatusTypes=CUSTOM"],
                "ListViews" => vec!["type=CUSTOMER_MANAGED"],
                _ => Vec::new(),
            };
            if later {
                query.push("nextToken=later");
            }
            Ok(Page {
                body: response(items, row, "NextToken", later),
                id: arn,
                wire: NativeRequest::get(format!("/{endpoint}/{owner}"), &query),
            })
        };
        Ok(Owner {
            value: owner.to_owned(),
            pages: [page(false)?, page(true)?],
        })
    };
    Ok(Case {
        operation,
        kind,
        parent_field: "InstanceId",
        requires_request_identity: false,
        owners: [owner(OWNERS[0])?, owner(OWNERS[1])?],
    })
}

#[tokio::test]
async fn given_two_instances_when_custom_statuses_owned_views_and_other_summary_pages_resume_then_native_ownership_filters_and_exact_rows_survive_sqlite()
-> TestResult {
    let cases = SPEC.into_iter().map(case).collect::<TestResult<Vec<_>>>()?;
    let excluded = ["DescribeView", "DescribeUser", "CreateView"];
    let operations = std::iter::once("ListInstances")
        .chain(SPEC.iter().map(|spec| spec.0))
        .chain(excluded)
        .collect::<Vec<_>>();
    // Exact native query assertions reject removing CUSTOM/CUSTOMER_MANAGED or replacing them with provider types.
    run(
        ("connect", "2017-08-08"),
        "ListInstances",
        "Id",
        json!({"InstanceSummaryList":OWNERS.map(instance)}),
        &operations,
        &excluded,
        cases,
    )
    .await
}
