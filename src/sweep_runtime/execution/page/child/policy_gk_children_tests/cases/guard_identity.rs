use super::*;

fn detectors() -> [Parent; 2] {
    ["detector-a", "detector-b"].map(|id| Parent {
        binding: id.to_owned(),
        id: id.to_owned(),
        kind: "AWS::GuardDuty::Detector",
        row: json!(id),
    })
}

fn guard(operation: &'static str, items: &'static str, kind: &'static str) -> Case {
    Case {
        service: "guardduty",
        date: "2017-11-28",
        root: "ListDetectors",
        root_items: "DetectorIds",
        operation,
        items,
        input: "DetectorId",
        token: "NextToken",
        kind,
        relation: "IN_DETECTOR",
        forbidden: &[
            "GetFilter",
            "DescribePublishingDestination",
            "GetFindings",
            "GetAdministratorAccount",
            "ListIPSets",
            "ListThreatIntelSets",
            "ListThreatEntitySets",
            "ListTrustedEntitySets",
        ],
    }
}

#[tokio::test]
async fn given_two_detectors_when_empty_filter_pages_resume_then_same_names_remain_detector_qualified_in_sqlite()
-> TestResult {
    exercise(
        guard("ListFilters", "FilterNames", "AWS::GuardDuty::Filter"),
        detectors(),
    )
    .await
}

#[tokio::test]
async fn given_two_detectors_when_publishing_destinations_resume_then_destination_ids_retain_detector_edges_without_detail()
-> TestResult {
    exercise(
        guard(
            "ListPublishingDestinations",
            "Destinations",
            "AWS::GuardDuty::PublishingDestination",
        ),
        detectors(),
    )
    .await
}

#[tokio::test]
async fn given_two_identity_stores_when_groups_resume_then_real_group_arns_and_store_edges_survive_without_users()
-> TestResult {
    let parents = ["d-1111111111", "d-2222222222"].map(|store| {
        let arn = format!("arn:aws:identitystore::123456789012:identitystore/{store}");
        Parent {
            binding: store.to_owned(),
            id: arn.clone(),
            kind: "AWS::ApiOnly::IdentityStore",
            row: json!({"IdentityStoreId":store,"IdentityStoreArn":arn}),
        }
    });
    exercise(
        Case {
            service: "identitystore",
            date: "2020-06-15",
            root: "ListIdentityStores",
            root_items: "IdentityStores",
            operation: "ListGroups",
            items: "Groups",
            input: "IdentityStoreId",
            token: "NextToken",
            kind: "AWS::IdentityStore::Group",
            relation: "IN_IDENTITY_STORE",
            forbidden: &[
                "ListUsers",
                "DescribeUser",
                "DescribeGroup",
                "ListGroupMemberships",
            ],
        },
        parents,
    )
    .await
}
