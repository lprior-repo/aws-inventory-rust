use super::*;

fn cases_domain(owner: &str) -> Json {
    json!({"domainId":owner,"domainArn":format!("arn:aws:cases:us-east-1:123456789012:domain/{owner}"),"name":"same"})
}

fn cases_case(
    operation: &'static str,
    kind: &'static str,
    items: &'static str,
    resource: &'static str,
) -> Case {
    let owner = |owner: &str| {
        let page = |later: bool| {
            let id = if later {
                "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
            } else {
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
            };
            let arn =
                format!("arn:aws:cases:us-east-1:123456789012:domain/{owner}/{resource}/{id}");
            let row = if operation == "ListLayouts" {
                json!({"layoutId":id,"layoutArn":arn,"name":"same"})
            } else {
                json!({"templateId":id,"templateArn":arn,"name":"same","status":"ACTIVE",
                    "tagPropagationConfigurations":[{"resourceType":"Cases","tagMap":{"environment":"test"}}]})
            };
            let query = if later {
                BTreeSet::from(["nextToken=later".to_owned()])
            } else {
                BTreeSet::new()
            };
            Page {
                body: response(items, row, "nextToken", later),
                id: arn,
                wire: NativeRequest {
                    method: "POST",
                    path: format!("/domains/{owner}/{resource}s-list"),
                    query,
                    body: Json::Null,
                },
            }
        };
        Owner {
            value: owner.to_owned(),
            pages: [page(false), page(true)],
        }
    };
    Case {
        operation,
        kind,
        parent_field: "domainId",
        requires_request_identity: false,
        owners: OWNERS.map(owner),
    }
}

#[tokio::test]
async fn given_two_cases_domains_when_condensed_layout_and_template_pages_resume_then_readonly_posts_preserve_domain_and_sqlite_rows_without_content()
-> TestResult {
    let excluded = ["GetLayout", "GetCase", "CreateTemplate"];
    let operations = [
        "ListDomains",
        "ListLayouts",
        "ListTemplates",
        "GetLayout",
        "GetCase",
        "CreateTemplate",
    ];
    let cases = vec![
        cases_case("ListLayouts", "AWS::Cases::Layout", "layouts", "layout"),
        cases_case(
            "ListTemplates",
            "AWS::Cases::Template",
            "templates",
            "template",
        ),
    ];
    run(
        "connectcases",
        "2022-10-03",
        "ListDomains",
        "domainId",
        json!({"domains":OWNERS.map(cases_domain)}),
        &operations,
        &excluded,
        cases,
    )
    .await
}

#[tokio::test]
async fn given_inactive_and_active_health_subscriptions_when_domain_pages_resume_then_both_existing_rows_persist_without_activation_or_patient_calls()
-> TestResult {
    let domain_ids = ["dom-ownera", "dom-ownerb"];
    let domains = domain_ids.map(|owner| json!({"domainId":owner,
        "arn":format!("arn:aws:health-agent:us-east-1:123456789012:domain/{owner}"),"name":"same","status":"ACTIVE","createdAt":1767225600}));
    let owner = |owner: &str| {
        let page = |later: bool| {
            let id = if later {
                "sub-bbbbbbbbbbbbbbbbbbbbb"
            } else {
                "sub-aaaaaaaaaaaaaaaaaaaaa"
            };
            let arn = format!(
                "arn:aws:health-agent:us-east-1:123456789012:domain/{owner}/subscription/{id}"
            );
            let row = if later {
                json!({"arn":arn,"domainId":owner,"subscriptionId":id,"status":"ACTIVE",
                "createdAt":1767225600,"lastUpdatedAt":1767312000,"activatedAt":1767312000})
            } else {
                json!({"arn":arn,"domainId":owner,"subscriptionId":id,"status":"INACTIVE",
                    "createdAt":1767225600,"lastUpdatedAt":1767225600})
            };
            let query = if later {
                vec!["nextToken=later"]
            } else {
                Vec::new()
            };
            Page {
                body: response("subscriptions", row, "nextToken", later),
                id: arn,
                wire: NativeRequest::get(format!("/domains/{owner}/subscriptions"), &query),
            }
        };
        Owner {
            value: owner.to_owned(),
            pages: [page(false), page(true)],
        }
    };
    let cases = vec![Case {
        operation: "ListSubscriptions",
        kind: "AWS::ApiOnly::ConnectHealthSubscription",
        parent_field: "domainId",
        requires_request_identity: false,
        owners: domain_ids.map(owner),
    }];
    let excluded = ["ActivateSubscription", "StartMedicalScribeListeningSession"];
    let operations = [
        "ListDomains",
        "ListSubscriptions",
        "ActivateSubscription",
        "StartMedicalScribeListeningSession",
    ];
    run(
        "connecthealth",
        "2025-01-29",
        "ListDomains",
        "domainId",
        json!({"domains":domains}),
        &operations,
        &excluded,
        cases,
    )
    .await
}
