use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn collect_explorer(client: &DiscoveryClient<'_>) -> ExplorerDiscovery {
    let sdk = aws_sdk_resourceexplorer2::Client::new(client.config);
    let mut result = ExplorerDiscovery {
        coverage: Some(DiscoveryCoverage::success()),
        ..ExplorerDiscovery::default()
    };
    match get_index(client, &sdk).await {
        Ok(Some(index)) => result.index = Some(index),
        Ok(None) => {
            result.selection = SourceSelection::Absent;
            return result;
        }
        Err(error) => {
            fail_explorer(&mut result, "resource-explorer-2:GetIndex", &error);
            return result;
        }
    }
    if result
        .index
        .as_ref()
        .and_then(|index| index.state.as_deref())
        != Some("ACTIVE")
    {
        result.selection = SourceSelection::Unavailable {
            reason: "existing Resource Explorer index is not ACTIVE".into(),
        };
        return result;
    }
    list_explorer_views(client, &sdk, &mut result).await;
    select_explorer_view(client, &sdk, &mut result).await;
    result
}

#[tracing::instrument(skip_all)]
pub(super) async fn get_index(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
) -> Result<Option<ExplorerIndex>, AwsScanError> {
    let output = client
        .execute("resource-explorer-2", client.region, || async {
            match sdk.get_index().send().await {
                Ok(output) => Ok(Some(output)),
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
    output.map(|output| map_index(client, output)).transpose()
}

pub(super) fn map_index(
    client: &DiscoveryClient<'_>,
    value: GetIndexOutput,
) -> Result<ExplorerIndex, AwsScanError> {
    let arn = required(value.arn(), "Resource Explorer index ARN")?;
    validate_scoped_arn(client, &arn, "resource-explorer-2", "index/")?;
    Ok(ExplorerIndex {
        arn,
        index_type: optional_text(value.r#type().map(|kind| kind.as_str()), "index type", 64)?,
        state: optional_text(value.state().map(|state| state.as_str()), "index state", 64)?,
        replicating_from: regions(value.replicating_from())?,
        replicating_to: regions(value.replicating_to())?,
    })
}

#[tracing::instrument(skip_all)]
pub(super) async fn list_explorer_views(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
    result: &mut ExplorerDiscovery,
) {
    let listing = pages::collect(
        client,
        "resource-explorer-2:ListViews",
        None,
        MAX_DISCOVERED_SOURCES,
        |token| async move {
            let response = client
                .execute("resource-explorer-2", client.region, || async {
                    sdk.list_views()
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 1_000))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.views.unwrap_or_default(),
                response.next_token,
                |arn| {
                    validate_scoped_arn(client, &arn, "resource-explorer-2", "view/")?;
                    Ok(arn)
                },
            )
        },
    )
    .await;
    result.candidates = listing.items;
    if let Some(coverage) = &mut result.coverage {
        coverage.merge(listing.coverage);
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn select_explorer_view(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
    result: &mut ExplorerDiscovery,
) {
    if !result
        .coverage
        .as_ref()
        .is_some_and(|coverage| coverage.complete)
    {
        result.selection = SourceSelection::Unavailable {
            reason: "view enumeration is incomplete".into(),
        };
        return;
    }
    match get_default_view(client, sdk).await {
        Ok(default) => result.default_view = default,
        Err(error) => {
            fail_explorer(result, "resource-explorer-2:GetDefaultView", &error);
            return;
        }
    }
    result.selection = select_view_candidate(&result.candidates, result.default_view.as_deref());
    if let Some(arn) = result.selection.selected_identifier().map(str::to_owned) {
        match get_view(client, sdk, &arn).await {
            Ok(view) => result.selected_view = Some(view),
            Err(error) => fail_explorer(result, "resource-explorer-2:GetView", &error),
        }
    }
}
