//! views operations.
use super::*;

pub(super) fn select_view_candidate(
    candidates: &[String],
    default: Option<&str>,
) -> SourceSelection {
    if let Some(default) = default {
        if candidates.iter().any(|candidate| candidate == default) {
            return SourceSelection::Selected {
                identifier: default.to_owned(),
                reason: "AWS regional default view".into(),
            };
        }
        return SourceSelection::Unavailable {
            reason: "default view was not in the completed view listing".into(),
        };
    }
    select_unique(
        candidates.to_vec(),
        true,
        "only existing view and no regional default",
    )
}

#[tracing::instrument(skip_all)]
pub(super) async fn get_default_view(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
) -> Result<Option<String>, AwsScanError> {
    let response = client
        .execute("resource-explorer-2", false, || async {
            match sdk.get_default_view().send().await {
                Ok(output) => Ok(output.view_arn),
                Err(error)
                    if error
                        .as_service_error()
                        .is_some_and(|error| error.is_resource_not_found_exception()) =>
                {
                    Ok(None)
                }
                Err(error) => Err(sdk_error(error)),
            }
        })
        .await?;
    if let Some(arn) = &response {
        validate_scoped_arn(client, arn, "resource-explorer-2", "view/")?;
    }
    Ok(response)
}

#[tracing::instrument(skip_all)]
pub(super) async fn get_view(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
    arn: &str,
) -> Result<ExplorerView, AwsScanError> {
    let output = client
        .execute("resource-explorer-2", false, || async {
            sdk.get_view().view_arn(arn).send().await.map_err(sdk_error)
        })
        .await?;
    let view = output
        .view
        .ok_or_else(|| data_error("GetView omitted its view"))?;
    map_view(client, arn, view)
}

pub(super) fn map_view(
    client: &DiscoveryClient<'_>,
    expected: &str,
    view: View,
) -> Result<ExplorerView, AwsScanError> {
    let arn = required(view.view_arn(), "Resource Explorer view ARN")?;
    validate_scoped_arn(client, &arn, "resource-explorer-2", "view/")?;
    if arn != expected || view.owner().is_some_and(|owner| owner != client.account) {
        return Err(data_error(
            "Resource Explorer view identity/owner changed during discovery",
        ));
    }
    if view.included_properties().len() > 64 {
        return Err(data_error("too many included view properties"));
    }
    let included_properties = view
        .included_properties()
        .iter()
        .map(|property| {
            bounded_text(property.name(), "included view property", 256)?;
            Ok(property.name().to_owned())
        })
        .collect::<Result<Vec<_>, AwsScanError>>()?;
    Ok(ExplorerView {
        arn,
        owner: optional_text(view.owner(), "view owner", 128)?,
        scope: optional_text(view.scope(), "view scope", 2_048)?,
        has_filter: view
            .filters()
            .is_some_and(|filter| !filter.filter_string().is_empty()),
        included_properties,
    })
}

pub(super) fn fail_explorer(result: &mut ExplorerDiscovery, operation: &str, error: &AwsScanError) {
    if let Some(coverage) = &mut result.coverage {
        coverage.fail(operation, None, error);
    }
    result.selection = SourceSelection::Unavailable {
        reason: format!("{operation} failed; inspect discovery coverage"),
    };
}

pub(super) fn validate_scoped_arn(
    client: &DiscoveryClient<'_>,
    arn: &str,
    service: &str,
    resource_prefix: &str,
) -> Result<(), AwsScanError> {
    bounded_text(arn, "source ARN", 2_048)?;
    let mut fields = arn.splitn(6, ':');
    let parts = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    );
    match parts {
        (
            Some("arn"),
            Some(partition),
            Some(found_service),
            Some(region),
            Some(account),
            Some(resource),
        ) if partition == client.partition
            && found_service == service
            && region == client.region
            && account == client.account
            && resource.starts_with(resource_prefix)
            && resource.len() > resource_prefix.len() =>
        {
            Ok(())
        }
        _ => Err(data_error(
            "discovered source ARN is outside the requested account, region or service",
        )),
    }
}
