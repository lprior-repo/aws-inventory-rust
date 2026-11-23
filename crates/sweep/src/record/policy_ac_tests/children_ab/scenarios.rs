use super::*;
mod owned_names;
fn arn(service: &str, suffix: &str) -> String {
    format!("arn:aws:{service}:us-east-1:123456789012:{suffix}")
}

fn scoped(parent: &str, child: &str) -> String {
    format!("{}:{parent}|{}:{child}", parent.len(), child.len())
}

#[tokio::test]
async fn given_two_workspaces_when_empty_first_child_pages_resume_then_both_amp_child_kinds_persist()
-> TestResult {
    let workspaces = ["a", "b"].map(|name| {
        json!({"workspaceId":name,"arn":arn("aps", &format!("workspace/{name}")),
        "status":{"statusCode":"ACTIVE"},"createdAt":0})
    });
    let mut pages = vec![
        Page::new(
            "ListWorkspaces",
            json!({}),
            json!({"workspaces":workspaces}),
            vec![arn("aps", "workspace/a"), arn("aps", "workspace/b")],
        )
        .rest("/workspaces".to_owned(), None),
    ];
    for workspace in ["a", "b"] {
        let parent = arn("aps", &format!("workspace/{workspace}"));
        for (operation, items, suffix) in [
            (
                "ListAnomalyDetectors",
                "anomalyDetectors",
                "anomalydetectors",
            ),
            (
                "ListRuleGroupsNamespaces",
                "ruleGroupsNamespaces",
                "rulegroupsnamespaces",
            ),
        ] {
            let id = arn("aps", &format!("workspace/{workspace}/{suffix}/same"));
            let mut item = json!({"arn":id,"status":{"statusCode":"ACTIVE"},"createdAt":0,"modifiedAt":0,"tags":{"owner":"inventory"}});
            let fields = item
                .as_object_mut()
                .ok_or("fixture item is not an object")?;
            if operation == "ListAnomalyDetectors" {
                fields.insert("anomalyDetectorId".to_owned(), json!("same"));
                fields.insert("alias".to_owned(), json!("same"));
            } else {
                fields.insert("name".to_owned(), json!("same"));
            }
            pages.extend(later_pages(
                (operation, json!({"workspaceId":workspace})),
                &parent,
                (items, "nextToken"),
                (item, id),
                &format!("/workspaces/{workspace}/{suffix}"),
                Some("nextToken"),
                &[],
            ));
        }
    }
    consume(
        "amp",
        "2020-08-01",
        &["ListWorkspaces"],
        &[
            ("ListAnomalyDetectors", "AWS::APS::AnomalyDetector"),
            ("ListRuleGroupsNamespaces", "AWS::APS::RuleGroupsNamespace"),
        ],
        &[
            "DescribeRuleGroupsNamespace",
            "CreateAnomalyDetector",
            "PutRuleGroupsNamespace",
        ],
        pages,
    )
    .await
}

fn mesh_ref(mesh: &str, field: Option<&str>, suffix: &str) -> Value {
    let mut item = json!({"meshName":mesh,"meshOwner":"123456789012","resourceOwner":"123456789012",
        "arn":arn("appmesh", suffix),"version":1,"createdAt":0,"lastUpdatedAt":0});
    if let (Some(field), Some(obj)) = (field, item.as_object_mut()) {
        obj.insert(field.to_owned(), json!("same"));
    }
    item
}

#[tokio::test]
async fn given_equal_router_and_gateway_names_in_two_meshes_when_routes_resume_then_parent_pairs_never_cross()
-> TestResult {
    let mut pages = vec![
        Page::new(
            "ListMeshes",
            json!({}),
            json!({"meshes":[mesh_ref("a", None, "mesh/a"),mesh_ref("b", None, "mesh/b")]}),
            vec![arn("appmesh", "mesh/a"), arn("appmesh", "mesh/b")],
        )
        .rest("/v20190125/meshes".to_owned(), None),
    ];
    for mesh in ["a", "b"] {
        for (
            parent_op,
            parent_items,
            field,
            segment,
            operation,
            items,
            child_field,
            child_segment,
        ) in [
            (
                "ListVirtualGateways",
                "virtualGateways",
                "virtualGatewayName",
                "virtualGateway",
                "ListGatewayRoutes",
                "gatewayRoutes",
                "gatewayRouteName",
                "gatewayRoutes",
            ),
            (
                "ListVirtualRouters",
                "virtualRouters",
                "virtualRouterName",
                "virtualRouter",
                "ListRoutes",
                "routes",
                "routeName",
                "routes",
            ),
        ] {
            let parent_suffix = format!("mesh/{mesh}/{segment}/same");
            let parent = arn("appmesh", &parent_suffix);
            pages.push(
                Page::new(
                    parent_op,
                    json!({"meshName":mesh}),
                    json!({parent_items:[mesh_ref(mesh, Some(field), &parent_suffix)]}),
                    vec![parent.clone()],
                )
                .under(&arn("appmesh", &format!("mesh/{mesh}")))
                .rest(format!("/v20190125/meshes/{mesh}/{segment}s"), None),
            );
            let id = arn("appmesh", &format!("{parent_suffix}/{child_segment}/same"));
            let mut item = mesh_ref(
                mesh,
                Some(field),
                &format!("{parent_suffix}/{child_segment}/same"),
            );
            item.as_object_mut()
                .ok_or("fixture item is not an object")?
                .insert(child_field.to_owned(), json!("same"));
            pages.extend(later_pages(
                (
                    operation,
                    json!({"meshName":mesh,field:"same","meshOwner":"123456789012"}),
                ),
                &parent,
                (items, "nextToken"),
                (item, id),
                &format!(
                    "/v20190125/meshes/{mesh}/{segment}/same/{child_segment}?meshOwner=123456789012"
                ),
                Some("nextToken"),
                &[],
            ));
        }
    }
    consume(
        "app-mesh",
        "2019-01-25",
        &["ListMeshes"],
        &[
            ("ListGatewayRoutes", "AWS::AppMesh::GatewayRoute"),
            ("ListRoutes", "AWS::AppMesh::Route"),
        ],
        &["DescribeGatewayRoute", "DescribeRoute", "CreateRoute"],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_two_backup_and_restore_plans_when_equal_selection_names_resume_then_selected_parent_keys_prevent_overwrites()
-> TestResult {
    let mut pages = vec![
        Page::new("ListBackupPlans", json!({}), json!({"BackupPlansList":[
            {"BackupPlanId":"a","BackupPlanArn":arn("backup", "backup-plan:a")}, {"BackupPlanId":"b","BackupPlanArn":arn("backup", "backup-plan:b")}]}),
            vec![arn("backup", "backup-plan:a"),arn("backup", "backup-plan:b")]).rest("/backup/plans".to_owned(), None),
        Page::new("ListRestoreTestingPlans", json!({}), json!({"RestoreTestingPlans":[
            {"RestoreTestingPlanName":"a","RestoreTestingPlanArn":arn("backup", "restore-testing-plan:a"),"CreationTime":0,"ScheduleExpression":"cron(0 5 ? * * *)"},
            {"RestoreTestingPlanName":"b","RestoreTestingPlanArn":arn("backup", "restore-testing-plan:b"),"CreationTime":0,"ScheduleExpression":"cron(0 5 ? * * *)"}]}),
            vec![arn("backup", "restore-testing-plan:a"),arn("backup", "restore-testing-plan:b")]).rest("/restore-testing/plans".to_owned(), None),
    ];
    for plan in ["a", "b"] {
        pages.extend(later_pages(
            ("ListBackupSelections", json!({"BackupPlanId":plan})),
            &arn("backup", &format!("backup-plan:{plan}")),
            ("BackupSelectionsList", "NextToken"),
            (
                json!({"BackupPlanId":plan,"SelectionId":"same","SelectionName":"same","CreationDate":0,
                    "IamRoleArn":"arn:aws:iam::123456789012:role/backup"}),
                scoped(plan, "same"),
            ),
            &format!("/backup/plans/{plan}/selections"),
            Some("nextToken"),
            &[],
        ));
        pages.extend(later_pages(
            ("ListRestoreTestingSelections", json!({"RestoreTestingPlanName":plan})),
            &arn("backup", &format!("restore-testing-plan:{plan}")),
            ("RestoreTestingSelections", "NextToken"),
            (
                json!({"RestoreTestingPlanName":plan,"RestoreTestingSelectionName":"same","CreationTime":0,
                    "IamRoleArn":"arn:aws:iam::123456789012:role/backup","ProtectedResourceType":"EBS","ValidationWindowHours":1}),
                scoped(plan, "same"),
            ),
            &format!("/restore-testing/plans/{plan}/selections"),
            Some("NextToken"),
            &[],
        ));
    }
    consume(
        "backup",
        "2018-11-15",
        &["ListBackupPlans", "ListRestoreTestingPlans"],
        &[
            ("ListBackupSelections", "AWS::Backup::BackupSelection"),
            (
                "ListRestoreTestingSelections",
                "AWS::Backup::RestoreTestingSelection",
            ),
        ],
        &["GetBackupSelection", "StartBackupJob"],
        pages,
    )
    .await
}
