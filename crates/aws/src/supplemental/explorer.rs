use super::*;

#[tracing::instrument(skip_all)]
pub(crate) async fn explorer(ctx: &Context, view: &str) -> Result<(), AwsScanError> {
    let client = aws_sdk_resourceexplorer2::Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope(
            "aws_resource_explorer_view",
            Some(view),
            Source::ResourceExplorer,
        ),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("resource-explorer-2", "read", || async {
                        client
                            .list_resources()
                            .view_arn(view)
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.resources().len(), budget)?;
                let observed = now_millis()?;
                let resources = output
                    .resources()
                    .iter()
                    .map(|resource| map_explorer(resource, observed))
                    .collect::<Result<_, _>>()?;
                Ok(PageBody {
                    resources,
                    listed_count: output.resources().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

pub(super) fn map_explorer(
    resource: &aws_sdk_resourceexplorer2::types::Resource,
    observed_at: i64,
) -> Result<Resource, AwsScanError> {
    let id = required(resource.arn(), "Resource Explorer ARN")?;
    let account = required(
        resource.owning_account_id(),
        "Resource Explorer owning account",
    )?;
    let region = required(resource.region(), "Resource Explorer resource region")?;
    let resource_type = required(resource.resource_type(), "Resource Explorer resource type")?;
    let mut tags = BTreeMap::new();
    for property in resource.properties() {
        if property.name() == Some("tags")
            && let Some(data) = property.data()
        {
            tags.extend(document_tags(data));
        }
    }
    let last_reported = resource
        .last_reported_at()
        .map(|date| date.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    let result = Resource {
        kind: explorer_resource_kind(&resource_type, &id),
        id,
        account_id: account,
        region,
        name: tags.get("Name").cloned(),
        tags,
        source: Source::ResourceExplorer,
        observed_at,
        properties: json!({"resource_type":resource_type,"service":resource.service(),"last_reported_at":last_reported,
            "property_names":resource.properties().iter().filter_map(|property|property.name()).collect::<Vec<_>>(),
            "coverage":"resources visible through the existing view; index and view limits apply"}),
    };
    result.validate()?;
    Ok(result)
}

pub(super) fn document_tags(document: &Document) -> BTreeMap<String, String> {
    if let Some(array) = document.as_array() {
        array
            .iter()
            .filter_map(|entry| {
                let entry = entry.as_object()?;
                let key = entry.get("Key").or_else(|| entry.get("key"))?.as_string()?;
                let value = entry
                    .get("Value")
                    .or_else(|| entry.get("value"))?
                    .as_string()?;
                Some((key.to_owned(), value.to_owned()))
            })
            .collect()
    } else if let Some(object) = document.as_object() {
        object
            .iter()
            .filter_map(|(key, value)| Some((key.clone(), value.as_string()?.to_owned())))
            .collect()
    } else {
        BTreeMap::new()
    }
}
