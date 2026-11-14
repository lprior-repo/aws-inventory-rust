use super::*;

#[tracing::instrument(skip_all)]
pub(crate) async fn config(ctx: &Context, aggregator: &str) -> Result<(), AwsScanError> {
    let client = aws_sdk_config::Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_config_aggregator", Some(aggregator), Source::AwsConfig),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("config", "read", || async {
                        client
                            .select_aggregate_resource_config()
                            .configuration_aggregator_name(aggregator)
                            .expression(CONFIG_QUERY)
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.results().len(), budget)?;
                let observed = now_millis()?;
                let resources = output
                    .results()
                    .iter()
                    .map(|row| parse_config_row(row, observed))
                    .collect::<Result<_, _>>()?;
                Ok(PageBody {
                    resources,
                    listed_count: output.results().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

pub(super) fn parse_config_row(text: &str, observed_at: i64) -> Result<Resource, AwsScanError> {
    if text.len() > MAX_PROPERTY_BYTES {
        return Err(AwsScanError::InvalidData("Config row exceeds 1 MiB".into()));
    }
    let row: Value =
        serde_json::from_str(text).map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    let account = string(&row, "accountId")?;
    let region = string(&row, "awsRegion")?;
    let resource_type = string(&row, "resourceType")?;
    let resource_id = string(&row, "resourceId")?;
    let kind = resource_kind(&resource_type);
    let id = row
        .get("arn")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("aws-config:{account}:{region}:{resource_type}:{resource_id}"));
    let configuration = match row.get("configuration") {
        Some(Value::String(value)) => serde_json::from_str(value)
            .map_err(|error| AwsScanError::InvalidData(error.to_string()))?,
        Some(value) => value.clone(),
        None => Value::Null,
    };
    let tags = json_tags(row.get("tags"));
    let name = row
        .get("resourceName")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| tags.get("Name").cloned());
    let mut properties = json!({"resource_type":resource_type,"resource_id":resource_id,
        "configuration":configuration,"configuration_capture_time":row.get("configurationItemCaptureTime"),
        "coverage":"recorded resources available to the existing aggregator"});
    if let Some(object) = properties.as_object_mut() {
        for (target, source) in config_aliases(&kind) {
            if let Some(value) = config_alias_value(&configuration, source) {
                object.insert((*target).to_owned(), value.clone());
            }
        }
    }
    let resource = Resource {
        id,
        account_id: account,
        region,
        kind,
        name,
        properties,
        tags,
        source: Source::AwsConfig,
        observed_at,
    };
    resource.validate()?;
    Ok(resource)
}

mod aliases;
pub(super) use aliases::{config_alias_value, config_aliases};
