use super::*;

use std::sync::Mutex;

#[tokio::test]
async fn appconfig_experiment_definition_builds_provider_model_and_list_request() -> TestResult {
    let (ctx, _receiver) = context(3)?;
    let types = selected(&["AWS::AppConfig::ExperimentDefinition"])?;
    let group = one_group(&types)?;
    assert_eq!(group.descriptor.type_name, "AWS::AppConfig::Application");
    assert_eq!(group.children.len(), 1);

    let requests = Mutex::new(Vec::new());
    let get_requests = Mutex::new(Vec::new());

    collect_group_with(
        &ctx,
        &group,
        |request| {
            let recorded = requests
                .lock()
                .map(|mut values| values.push(request.clone()))
                .map_err(|_| ApiError::Permanent("poisoned requests".into()));
            async move {
                recorded?;
                if request.type_name == "AWS::AppConfig::Application" {
                    return Ok(listed(
                        &request.type_name,
                        vec![description("app123", json!({}))],
                        None,
                    ));
                }
                if request.type_name == "AWS::AppConfig::ExperimentDefinition" {
                    assert_eq!(
                        request.resource_model.as_deref(),
                        Some("{\"ApplicationIdentifier\":\"app123\"}")
                    );
                    return Ok(listed(
                        &request.type_name,
                        vec![description("exp1", json!({}))],
                        None,
                    ));
                }
                Err(ApiError::Permanent("unexpected list".into()))
            }
        },
        |request| {
            let recorded = get_requests
                .lock()
                .map(|mut values| values.push(request.clone()))
                .map_err(|_| ApiError::Permanent("poisoned requests".into()));
            async move {
                recorded?;
                if request.type_name == "AWS::AppConfig::Application" {
                    return Ok(got(&request, json!({"ApplicationId": "app123"})));
                }
                if request.type_name == "AWS::AppConfig::ExperimentDefinition" {
                    return Ok(got(
                        &request,
                        json!({
                            "ApplicationId": "app123",
                            "Id": "exp1",
                            "Name": "rollout-exp"
                        }),
                    ));
                }
                Err(ApiError::Permanent("unexpected get".into()))
            }
        },
    )
    .await?;

    let requests = requests.into_inner().map_err(|_| "poisoned requests")?;
    let get_requests = get_requests.into_inner().map_err(|_| "poisoned requests")?;

    assert!(
        requests
            .iter()
            .any(|r| r.type_name == "AWS::AppConfig::Application")
    );
    assert!(
        requests
            .iter()
            .any(|r| r.type_name == "AWS::AppConfig::ExperimentDefinition")
    );
    assert!(
        get_requests
            .iter()
            .any(|r| r.type_name == "AWS::AppConfig::Application")
    );
    assert!(
        get_requests
            .iter()
            .any(|r| r.type_name == "AWS::AppConfig::ExperimentDefinition")
    );

    let list_requests: Vec<_> = requests
        .iter()
        .filter(|r| r.type_name == "AWS::AppConfig::ExperimentDefinition")
        .collect();
    assert_eq!(list_requests.len(), 1);
    let list_request = list_requests.first().ok_or("list request")?;
    let model_json = list_request
        .resource_model
        .as_ref()
        .ok_or("resource model missing")?;
    let model: Value =
        serde_json::from_str(model_json).map_err(|e| format!("invalid model: {e}"))?;
    assert_eq!(model, json!({"ApplicationIdentifier": "app123"}));
    Ok(())
}
