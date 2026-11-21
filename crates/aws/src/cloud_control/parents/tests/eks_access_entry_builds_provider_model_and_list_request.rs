use super::*;

use std::sync::Mutex;

#[tokio::test]
async fn eks_access_entry_builds_provider_model_and_list_request() -> TestResult {
    let (ctx, _receiver) = context(3)?;
    let types = selected(&["AWS::EKS::AccessEntry"])?;
    let group = one_group(&types)?;
    assert_eq!(group.descriptor.type_name, "AWS::EKS::Cluster");
    assert_eq!(group.children.len(), 1);

    let requests = Mutex::new(Vec::new());
    let get_requests = Mutex::new(Vec::new());

    collect_group_with(
        &ctx,
        &group,
        |request| {
            let recorded = requests.lock().map(|mut values| values.push(request.clone()))
                .map_err(|_| ApiError::Permanent("poisoned requests".into()));
            async move {
                recorded?;
                if request.type_name == "AWS::EKS::Cluster" {
                    return Ok(listed(&request.type_name, vec![description("alpha", json!({}))], None));
                }
                if request.type_name == "AWS::EKS::AccessEntry" {
                    assert_eq!(request.resource_model.as_deref(), Some("{\"ClusterName\":\"alpha\"}"));
                    return Ok(listed(
                        &request.type_name,
                        vec![description(
                            "arn:aws:eks:us-east-1:111111111111:cluster/alpha/entry/p1",
                            json!({}),
                        )],
                        None,
                    ));
                }
                Err(ApiError::Permanent("unexpected list".into()))
            }
        },
        |request| {
            let recorded = get_requests.lock().map(|mut values| values.push(request.clone()))
                .map_err(|_| ApiError::Permanent("poisoned requests".into()));
            async move {
                recorded?;
                if request.type_name == "AWS::EKS::Cluster" {
                    return Ok(got(&request, json!({"Name": "alpha"})));
                }
                if request.type_name == "AWS::EKS::AccessEntry" {
                    return Ok(got(&request, json!({
                        "ClusterName": "alpha",
                        "PrincipalArn": "arn:aws:iam::111111111111:role/p1",
                        "AccessEntryArn": "arn:aws:eks:us-east-1:111111111111:cluster/alpha/entry/p1"
                    })));
                }
                Err(ApiError::Permanent("unexpected get".into()))
            }
        },
    )
    .await?;

    let requests = requests.into_inner().map_err(|_| "poisoned requests")?;
    let get_requests = get_requests.into_inner().map_err(|_| "poisoned requests")?;

    assert!(requests.iter().any(|r| r.type_name == "AWS::EKS::Cluster"));
    assert!(
        requests
            .iter()
            .any(|r| r.type_name == "AWS::EKS::AccessEntry")
    );
    assert!(
        get_requests
            .iter()
            .any(|r| r.type_name == "AWS::EKS::Cluster")
    );
    assert!(
        get_requests
            .iter()
            .any(|r| r.type_name == "AWS::EKS::AccessEntry")
    );

    let list_requests: Vec<_> = requests
        .iter()
        .filter(|r| r.type_name == "AWS::EKS::AccessEntry")
        .collect();
    assert_eq!(list_requests.len(), 1);
    let list_request = list_requests.first().ok_or("list request")?;
    let model_json = list_request
        .resource_model
        .as_ref()
        .ok_or("resource model missing")?;
    let model: Value =
        serde_json::from_str(model_json).map_err(|e| format!("invalid model: {e}"))?;
    assert_eq!(model, json!({"ClusterName": "alpha"}));
    Ok(())
}
