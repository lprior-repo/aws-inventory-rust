//! AppConfig ExperimentDefinition parent model construction and list request verification.
use super::*;

#[tokio::test]
async fn appconfig_experiment_definition_builds_provider_model_and_list_request() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::AppConfig::ExperimentDefinition"])?;
    let group = one_group(&types)?;
    assert_eq!(group.descriptor.type_name, "AWS::AppConfig::Application");
    assert_eq!(group.children.len(), 1);

    let mut list_model_seen = false;
    let mut get_called = false;

    collect_group_with(
        &ctx,
        &group,
        |request| {
            let type_name = request.type_name.clone();
            let resource_model = request.resource_model.clone();
            async move {
                if type_name == "AWS::AppConfig::Application" {
                    return Ok(listed(&type_name, vec![description("app123", json!({}))], None));
                }
                if type_name == "AWS::AppConfig::ExperimentDefinition" {
                    list_model_seen = true;
                    assert!(resource_model.is_some());
                    let model: Value = serde_json::from_str(&resource_model.unwrap())?;
                    assert_eq!(
                        model,
                        json!({"ApplicationIdentifier": "app123"}),
                        "list request must carry application identifier from parent binding"
                    );
                    return Ok(listed(
                        &type_name,
                        vec![description("exp1", json!({}))],
                        None,
                    ));
                }
                Err(ApiError::Permanent("unexpected list".into()))
            }
        },
        |request| {
            let type_name = request.type_name.clone();
            async move {
                if type_name == "AWS::AppConfig::Application" {
                    return Ok(got(&request, json!({"ApplicationId": "app123"})));
                }
                if type_name == "AWS::AppConfig::ExperimentDefinition" {
                    get_called = true;
                    return Ok(got(&request, json!({
                        "ApplicationId": "app123",
                        "Id": "exp1",
                        "Name": "rollout-exp"
                    })));
                }
                Err(ApiError::Permanent("unexpected get".into()))
            }
        },
    )
    .await?;

    let events = events(&mut receiver)?;
    assert_eq!(events.len(), 4);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CollectionEvent::Page(p) if p.scope.collector == "cloud_control_parent_discovery" && p.resources.iter().any(|r| r.type_name == "AWS::AppConfig::Application")))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CollectionEvent::Page(p) if p.scope.collector == "cloud_control_list" && p.resources.iter().any(|r| r.type_name == "AWS::AppConfig::ExperimentDefinition")))
    );
    assert!(list_model_seen, "list must carry resource model");
    assert!(get_called, "read must be invoked");
    Ok(())
}
