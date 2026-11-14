use super::*;

pub(super) fn map_rule(
    ctx: &Context,
    listener: &str,
    model: &Rule,
) -> Result<PageBody, AwsScanError> {
    check_nested(model.conditions(), "rule conditions")?;
    check_nested(model.transforms(), "rule transforms")?;
    let conditions = model
        .conditions()
        .iter()
        .map(map_condition)
        .collect::<Result<Vec<_>, _>>()?;
    let transforms = model
        .transforms()
        .iter()
        .map(map_transform)
        .collect::<Result<Vec<_>, _>>()?;
    let (actions, targets) = map_actions(model.actions())?;
    let resource = ctx.resource(
        "aws_elbv2_rule",
        required(model.rule_arn(), "rule ARN")?,
        None,
        json!({
            "rule_arn": model.rule_arn(), "listener_arn": listener, "priority": model.priority(),
            "is_default": model.is_default(), "conditions": conditions, "actions": actions,
            "transforms": transforms, "target_group_arns": targets, "tags_collected": false,
        }),
        BTreeMap::new(),
    )?;
    let mut edges = vec![ctx.edge(&resource, listener.to_owned(), "RULE_OF")];
    edges.extend(
        targets
            .into_iter()
            .map(|arn| ctx.edge(&resource, arn, "FORWARDS_TO")),
    );
    Ok(PageBody {
        resources: vec![resource],
        edges,
        ..PageBody::default()
    })
}

pub(super) fn map_actions(
    actions: &[Action],
) -> Result<(Vec<Value>, BTreeSet<String>), AwsScanError> {
    check_nested(actions, "listener actions")?;
    let mut targets = BTreeSet::new();
    let values = actions.iter().map(|action| {
        let action_type = required(action.r#type().map(|kind| kind.as_str()), "listener action type")?;
        let forward = action.forward_config();
        let groups = forward.map(|config| config.target_groups()).unwrap_or_default();
        check_nested(groups, "forward target groups")?;


        if action_type == "forward" {
            if action.target_group_arn().is_none() && groups.is_empty() {
                return Err(AwsScanError::InvalidData("forward action lacks target groups".into()));
            }
            if let Some(arn) = action.target_group_arn() {
                targets.insert(required(Some(arn), "forward target group ARN")?);
            }
            for group in groups {
                targets.insert(required(group.target_group_arn(), "weighted target group ARN")?);
            }
        }
        Ok(json!({
            "type": action_type, "order": action.order(),
            "target_group_arn": action.target_group_arn(),
            "forward_config": forward.map(|config| json!({
                "target_groups": config.target_groups().iter().map(|group| json!({
                    "target_group_arn": group.target_group_arn(), "weight": group.weight(),
                })).collect::<Vec<_>>(),
                "target_group_stickiness_config": config.target_group_stickiness_config().map(|value| json!({
                    "enabled": value.enabled(), "duration_seconds": value.duration_seconds(),
                })),
            })),
            "redirect_config": action.redirect_config().map(|value| json!({
                "protocol": value.protocol(), "port": value.port(), "host": value.host(),
                "path": value.path(), "query": value.query(), "status_code": value.status_code().map(|status| status.as_str()),
            })),
            "fixed_response_config": action.fixed_response_config().map(|value| json!({
                "status_code": value.status_code(), "content_type": value.content_type(), "message_body": value.message_body(),
            })),
            "authenticate_oidc_config_present": action.authenticate_oidc_config().is_some(),
            "authenticate_cognito_config_present": action.authenticate_cognito_config().is_some(),
            "jwt_validation_config_present": action.jwt_validation_config().is_some(),
            "authentication_details_collected": false,
        }))
    }).collect::<Result<Vec<_>, AwsScanError>>()?;
    Ok((values, targets))
}

pub(super) fn map_condition(condition: &RuleCondition) -> Result<Value, AwsScanError> {
    check_nested(condition.values(), "condition values")?;
    check_nested(condition.regex_values(), "condition regex values")?;
    let host = condition.host_header_config();
    let path = condition.path_pattern_config();
    let header = condition.http_header_config();
    let methods = condition
        .http_request_method_config()
        .map(|value| value.values())
        .unwrap_or_default();
    let ips = condition
        .source_ip_config()
        .map(|value| value.values())
        .unwrap_or_default();
    let query = condition
        .query_string_config()
        .map(|value| value.values())
        .unwrap_or_default();
    for values in [
        host.map(|value| value.values()).unwrap_or_default(),
        host.map(|value| value.regex_values()).unwrap_or_default(),
        path.map(|value| value.values()).unwrap_or_default(),
        path.map(|value| value.regex_values()).unwrap_or_default(),
        header.map(|value| value.values()).unwrap_or_default(),
        header.map(|value| value.regex_values()).unwrap_or_default(),
        methods,
        ips,
    ] {
        check_nested(values, "condition configuration values")?;
    }
    check_nested(query, "query string condition values")?;
    Ok(json!({
        "field": condition.field(), "values": condition.values(), "regex_values": condition.regex_values(),
        "host_header_config": host.map(|value| json!({"values": value.values(), "regex_values": value.regex_values()})),
        "path_pattern_config": path.map(|value| json!({"values": value.values(), "regex_values": value.regex_values()})),
        "http_header_config": header.map(|value| json!({"http_header_name": value.http_header_name(), "values": value.values(), "regex_values": value.regex_values()})),
        "http_request_method_config": condition.http_request_method_config().map(|value| json!({"values": value.values()})),
        "source_ip_config": condition.source_ip_config().map(|value| json!({"values": value.values(), "ip_address_type": value.ip_address_type().map(|kind| kind.as_str())})),
        "query_string_config": condition.query_string_config().map(|value| json!({"values": value.values().iter().map(|pair| json!({"key": pair.key(), "value": pair.value()})).collect::<Vec<_>>()})),
    }))
}

pub(super) fn map_transform(transform: &RuleTransform) -> Result<Value, AwsScanError> {
    let host = transform
        .host_header_rewrite_config()
        .map(|value| value.rewrites())
        .unwrap_or_default();
    let url = transform
        .url_rewrite_config()
        .map(|value| value.rewrites())
        .unwrap_or_default();
    check_nested(host, "host header rewrites")?;
    check_nested(url, "URL rewrites")?;
    Ok(json!({
        "type": transform.r#type().map(|value| value.as_str()),
        "host_header_rewrites": host.iter().map(|rewrite| json!({"regex": rewrite.regex(), "replace": rewrite.replace()})).collect::<Vec<_>>(),
        "url_rewrites": url.iter().map(|rewrite| json!({"regex": rewrite.regex(), "replace": rewrite.replace()})).collect::<Vec<_>>(),
    }))
}
