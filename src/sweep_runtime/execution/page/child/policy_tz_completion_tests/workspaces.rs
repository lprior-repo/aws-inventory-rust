use super::*;

#[tokio::test]
async fn given_two_directory_pool_and_application_parents_when_untraited_pagers_and_typed_association_selectors_execute_then_every_owned_child_survives_sqlite()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-workspaces-completion").await?;
    scenario
        .seed(
            &catalog,
            "workspaces",
            &[
                "DescribeWorkspaceDirectories",
                "DescribeWorkspacesPools",
                "DescribeApplications",
                "DescribeWorkspaceBundles",
                "DescribeWorkspaceImages",
                "DescribeWorkspaces",
            ],
        )
        .await?;
    let (scenario, seen) = scenario.drain(&catalog, "workspaces", |plan, request, wire, resumed| {
        let native = if wire.headers().get("smithy-protocol") == Some(&http::HeaderValue::from_static("rpc-v2-cbor")) {
            crate::sweep_runtime::execution::page::child::policy_tz_completion_tests::decode_rpc_body(wire.body())?
        } else {
            let body: Json = if wire.body().is_empty() { Json::Null } else { serde_json::from_slice(wire.body())? };
            match body {
                Json::Object(map) => map,
                _ => return Err("expected object body".into()),
            }
        };
        assert_eq!(wire.method(), http::Method::POST);
        if !plan.pagination().tokens.is_empty() { assert_token(wire,"NextToken",resumed)?; }
        let p = if resumed { "2" } else { "1" };
        match plan.operation_name() {
            "DescribeWorkspaceDirectories" => Ok(json!({"Directories":[{"DirectoryId":"d-one","DirectoryName":"one"},{"DirectoryId":"d-two","DirectoryName":"two"}]})),
            "DescribeWorkspacesPools" => Ok(json!({"WorkspacesPools":[{"PoolId":"wspool-one","PoolArn":"arn:aws:workspaces:us-east-1:123456789012:workspacespool/wspool-one","CapacityStatus":{"AvailableUserSessions":8,"DesiredUserSessions":10,"ActualUserSessions":10,"ActiveUserSessions":2},"PoolName":"one","State":"RUNNING","CreatedAt":1609459200.0,"BundleId":"wsb-one","DirectoryId":"d-one","RunningMode":"AUTO_STOP"},{"PoolId":"wspool-two","PoolArn":"arn:aws:workspaces:us-east-1:123456789012:workspacespool/wspool-two","CapacityStatus":{"AvailableUserSessions":7,"DesiredUserSessions":10,"ActualUserSessions":10,"ActiveUserSessions":3},"PoolName":"two","State":"RUNNING","CreatedAt":1609459200.0,"BundleId":"wsb-two","DirectoryId":"d-two","RunningMode":"AUTO_STOP"}]})),
            "DescribeApplications" => {
                assert_eq!(native.get("Owner"),Some(&json!("123456789012")));
                Ok(json!({"Applications":[{"ApplicationId":"wsa-one","Name":"one","Owner":"123456789012"},{"ApplicationId":"wsa-two","Name":"two","Owner":"123456789012"}]}))
            }
            "DescribeWorkspaceBundles" => Ok(json!({"Bundles":[{"BundleId":"wsb-one","Name":"one","Owner":"123456789012"},{"BundleId":"wsb-two","Name":"two","Owner":"123456789012"}]})),
            "DescribeWorkspaceImages" => Ok(json!({"Images":[{"ImageId":"wsi-one","Name":"one","OwnerAccountId":"123456789012"},{"ImageId":"wsi-two","Name":"two","OwnerAccountId":"123456789012"}]})),
            "DescribeWorkspaces" => Ok(json!({"Workspaces":[{"WorkspaceId":"ws-one","DirectoryId":"d-one","BundleId":"wsb-one","State":"AVAILABLE","ComputerName":"ws-one"},{"WorkspaceId":"ws-two","DirectoryId":"d-two","BundleId":"wsb-two","State":"AVAILABLE","ComputerName":"ws-two"}]})),
            "DescribeConnectClientAddIns" => {
                let directory = native.get("ResourceId").and_then(Json::as_str).ok_or("ResourceId absent")?;
                assert!(matches!(directory,"d-one"|"d-two"));
                assert_eq!(text(request,"ResourceId")?,directory);
                assert_eq!(native.get("MaxResults"),Some(&json!(25)));
                Ok(page("AddIns",json!({"ResourceId":directory,"AddInId":format!("addin-{p}"),"Name":"metadata addin","URL":"https://addin.example.test"}),!resumed,"NextToken"))
            }
            "DescribeWorkspacesPoolSessions" => {
                let pool = native.get("PoolId").and_then(Json::as_str).ok_or("PoolId absent")?;
                assert!(matches!(pool,"wspool-one"|"wspool-two"));
                assert_eq!(text(request,"PoolId")?,pool);
                assert_eq!(native.get("Limit"),Some(&json!(50)));
                assert_eq!(native.get("UserId"),None,"session user filter must not narrow pool inventory");
                Ok(page("Sessions",json!({"PoolId":pool,"SessionId":format!("session-{p}"),"UserId":"metadata-user","AuthenticationType":"SAML","ConnectionState":"CONNECTED"}),!resumed,"NextToken"))
            }
            "DescribeApplicationAssociations" => {
                let application = native.get("ApplicationId").and_then(Json::as_str).ok_or("ApplicationId absent")?;
                assert!(matches!(application,"wsa-one"|"wsa-two"));
                assert_eq!(text(request,"ApplicationId")?,application);
                assert_eq!(native.get("AssociatedResourceTypes"),Some(&json!(["WORKSPACE","BUNDLE","IMAGE"])));
                let mut response = json!({"Associations":[
                    {"ApplicationId":application,"AssociatedResourceId":if resumed {"ws-two"} else {"ws-one"},"AssociatedResourceType":"WORKSPACE","State":"COMPLETED"},
                    {"ApplicationId":application,"AssociatedResourceId":if resumed {"wsb-two"} else {"wsb-one"},"AssociatedResourceType":"BUNDLE","State":"COMPLETED"},
                    {"ApplicationId":application,"AssociatedResourceId":if resumed {"wsi-two"} else {"wsi-one"},"AssociatedResourceType":"IMAGE","State":"COMPLETED"}
                ]});
                if !resumed {
                    response.get_mut("Associations").and_then(Json::as_array_mut)
                        .ok_or("native application associations array absent")?
                        .push(json!({"ApplicationId":application,"AssociatedResourceId":"wsb-one","AssociatedResourceType":"WORKSPACE","State":"COMPLETED"}));
                }
                if !resumed {
                    response
                        .as_object_mut()
                        .ok_or("native response object absent")?
                        .insert("NextToken".to_owned(), json!("page-two"));
                }
                Ok(response)
            }
            "DescribeBundleAssociations" | "DescribeImageAssociations" | "DescribeWorkspaceAssociations" => {
                let field = match plan.operation_name() { "DescribeBundleAssociations" => "BundleId", "DescribeImageAssociations" => "ImageId", _ => "WorkspaceId" };
                let parent = native.get(field).and_then(Json::as_str).ok_or_else(|| format!("{field} absent"))?;
                assert!(parent.ends_with("-one") || parent.ends_with("-two"));
                assert_eq!(text(request,field)?,parent);
                assert_eq!(native.get("AssociatedResourceTypes"),Some(&json!(["APPLICATION"])));
                assert!(!resumed);
                Ok(page("Associations",json!({field:parent,"AssociatedResourceId":"wsa-one","AssociatedResourceType":"APPLICATION","State":"COMPLETED"}),false,"NextToken"))
            }
            other => Err(format!("unexpected WorkSpaces operation {other}").into()),
        }
    }).await?;
    assert_eq!(seen.len(), 12);
    assert_eq!(seen.get("DescribeConnectClientAddIns"), Some(&4));
    assert_eq!(seen.get("DescribeWorkspacesPoolSessions"), Some(&4));
    assert_eq!(seen.get("DescribeApplicationAssociations"), Some(&4));
    let addins = ["d-one", "d-two"]
        .into_iter()
        .flat_map(|parent| ["1", "2"].map(move |p| correlated(&[parent, &format!("addin-{p}")])))
        .collect();
    scenario
        .assert_ids("AWS::ApiOnly::WorkSpacesConnectClientAddIn", addins)
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WorkSpacesConnectClientAddIn",
            "IN_DIRECTORY",
            ["d-one", "d-two"]
                .into_iter()
                .flat_map(|parent| {
                    ["1", "2"].map(move |p| {
                        (
                            correlated(&[parent, &format!("addin-{p}")]),
                            "AWS::WorkSpaces::Directory".to_owned(),
                            parent.to_owned(),
                        )
                    })
                })
                .collect(),
        )
        .await?;
    let sessions = ["wspool-one", "wspool-two"]
        .into_iter()
        .flat_map(|parent| ["1", "2"].map(move |p| correlated(&[parent, &format!("session-{p}")])))
        .collect();
    scenario
        .assert_ids("AWS::ApiOnly::WorkSpacesPoolSession", sessions)
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WorkSpacesPoolSession",
            "IN_POOL",
            ["wspool-one", "wspool-two"]
                .into_iter()
                .flat_map(|parent| {
                    ["1", "2"].map(move |p| {
                        (
                            correlated(&[parent, &format!("session-{p}")]),
                            "AWS::WorkSpaces::WorkspacesPool".to_owned(),
                            format!(
                                "arn:aws:workspaces:us-east-1:123456789012:workspacespool/{parent}"
                            ),
                        )
                    })
                })
                .collect(),
        )
        .await?;
    let mut applications = Vec::new();
    let mut parent_edges = Vec::new();
    let mut target_edges = BTreeMap::<&str, Vec<(String, String, String)>>::new();
    for application in ["wsa-one", "wsa-two"] {
        for suffix in ["one", "two"] {
            for (prefix, kind, target_kind, relation) in [
                (
                    "ws",
                    "WORKSPACE",
                    "AWS::WorkSpaces::Workspace",
                    "ASSOCIATES_WORKSPACE",
                ),
                (
                    "wsb",
                    "BUNDLE",
                    "AWS::ApiOnly::WorkSpacesBundle",
                    "ASSOCIATES_BUNDLE",
                ),
                (
                    "wsi",
                    "IMAGE",
                    "AWS::ApiOnly::WorkSpacesImage",
                    "ASSOCIATES_IMAGE",
                ),
            ] {
                let target = format!("{prefix}-{suffix}");
                let id = correlated(&[application, &target, kind]);
                applications.push(id.clone());
                parent_edges.push((
                    id.clone(),
                    "AWS::ApiOnly::WorkSpacesApplication".to_owned(),
                    application.to_owned(),
                ));
                target_edges.entry(relation).or_default().push((
                    id,
                    target_kind.to_owned(),
                    target,
                ));
            }
        }
    }
    for application in ["wsa-one", "wsa-two"] {
        let mismatched = correlated(&[application, "wsb-one", "WORKSPACE"]);
        applications.push(mismatched.clone());
        parent_edges.push((
            mismatched,
            "AWS::ApiOnly::WorkSpacesApplication".to_owned(),
            application.to_owned(),
        ));
    }
    scenario
        .assert_ids(
            "AWS::ApiOnly::WorkSpacesApplicationAssociation",
            applications,
        )
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WorkSpacesApplicationAssociation",
            "FOR_APPLICATION",
            parent_edges,
        )
        .await?;
    for (relation, edges) in target_edges {
        scenario
            .assert_edges(
                "AWS::ApiOnly::WorkSpacesApplicationAssociation",
                relation,
                edges,
            )
            .await?;
    }
    for (kind, prefix) in [
        ("WorkSpacesBundleAssociation", "wsb"),
        ("WorkSpacesImageAssociation", "wsi"),
        ("WorkSpacesWorkspaceAssociation", "ws"),
    ] {
        let expected = ["one", "two"]
            .map(|suffix| correlated(&[&format!("{prefix}-{suffix}"), "wsa-one", "APPLICATION"]))
            .to_vec();
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), expected)
            .await?;
        let (target_kind, relation) = match prefix {
            "wsb" => ("AWS::ApiOnly::WorkSpacesBundle", "FOR_BUNDLE"),
            "wsi" => ("AWS::ApiOnly::WorkSpacesImage", "FOR_IMAGE"),
            _ => ("AWS::WorkSpaces::Workspace", "FOR_WORKSPACE"),
        };
        let parents = ["one", "two"]
            .map(|suffix| {
                let parent = format!("{prefix}-{suffix}");
                (
                    correlated(&[&parent, "wsa-one", "APPLICATION"]),
                    target_kind.to_owned(),
                    parent,
                )
            })
            .to_vec();
        scenario
            .assert_edges(&format!("AWS::ApiOnly::{kind}"), relation, parents)
            .await?;
        scenario
            .assert_edges(
                &format!("AWS::ApiOnly::{kind}"),
                "ASSOCIATES_APPLICATION",
                ["one", "two"]
                    .map(|suffix| {
                        (
                            correlated(&[&format!("{prefix}-{suffix}"), "wsa-one", "APPLICATION"]),
                            "AWS::ApiOnly::WorkSpacesApplication".to_owned(),
                            "wsa-one".to_owned(),
                        )
                    })
                    .to_vec(),
            )
            .await?;
    }
    Ok(())
}
