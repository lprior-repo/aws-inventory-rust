//! configuration operations.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn collect_config(client: &DiscoveryClient<'_>) -> ConfigDiscovery {
    let sdk = aws_sdk_config::Client::new(client.config);
    let listing = pages::collect(
        client,
        "config:DescribeConfigurationAggregators",
        None,
        MAX_DISCOVERED_SOURCES,
        |token| {
            let sdk = &sdk;
            async move {
                let output = client
                    .execute("config", false, || async {
                        sdk.describe_configuration_aggregators()
                            .set_next_token(token.clone())
                            .limit(client.page_size.clamp(1, 100))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                Page::map(
                    output.configuration_aggregators.unwrap_or_default(),
                    output.next_token,
                    |aggregator| map_aggregator(client, aggregator),
                )
            }
        },
    )
    .await;
    let names = listing
        .items
        .iter()
        .map(|aggregator| aggregator.name.clone())
        .collect();
    let selection = select_unique(
        names,
        listing.coverage.complete,
        "only existing Config aggregator in this region",
    );
    ConfigDiscovery {
        candidates: listing.items,
        selection,
        coverage: Some(listing.coverage),
    }
}

pub(super) fn map_aggregator(
    client: &DiscoveryClient<'_>,
    value: ConfigurationAggregator,
) -> Result<ConfigAggregator, AwsScanError> {
    let name = required(
        value.configuration_aggregator_name(),
        "Config aggregator name",
    )?;
    bounded_text(&name, "Config aggregator name", 256)?;
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(data_error("invalid Config aggregator name"));
    }
    let arn = required(
        value.configuration_aggregator_arn(),
        "Config aggregator ARN",
    )?;
    validate_scoped_arn(client, &arn, "config", "config-aggregator/")?;
    let account_sources = map_aggregation_scopes(value.account_aggregation_sources())?;
    let organization = value.organization_aggregation_source();
    let organization_regions = organization
        .map(|source| regions(source.aws_regions()))
        .transpose()?
        .unwrap_or_default();
    Ok(ConfigAggregator {
        name,
        arn,
        account_sources,
        organization_source: organization.is_some(),
        organization_all_regions: organization.is_some_and(|source| source.all_aws_regions()),
        organization_regions,
        has_filters: value.aggregator_filters().is_some(),
    })
}

pub(super) fn map_aggregation_scopes(
    scopes: &[aws_sdk_config::types::AccountAggregationSource],
) -> Result<Vec<AccountAggregationScope>, AwsScanError> {
    if scopes.len() > MAX_CONFIG_SOURCE_GROUPS {
        return Err(data_error("too many Config account source groups"));
    }
    let total_ids = scopes.iter().fold(0usize, |count, scope| {
        count.saturating_add(scope.account_ids().len())
    });
    if total_ids > MAX_CONFIG_ACCOUNT_IDS {
        return Err(data_error(
            "Config aggregator account metadata limit exceeded",
        ));
    }
    scopes
        .iter()
        .map(|source| {
            source.account_ids().iter().try_for_each(|account| {
                validate_account_id(account)
                    .map_err(|_| data_error("invalid Config source account ID"))
            })?;
            Ok(AccountAggregationScope {
                account_ids: source.account_ids().to_vec(),
                all_regions: source.all_aws_regions(),
                regions: regions(source.aws_regions())?,
            })
        })
        .collect()
}

pub(super) fn regions(values: &[String]) -> Result<Vec<String>, AwsScanError> {
    if values.len() > MAX_REGIONS {
        return Err(data_error("regional metadata exceeds 64 entries"));
    }
    values.iter().try_for_each(|region| {
        crate::validate_region(region).map_err(|_| data_error("invalid discovered source region"))
    })?;
    Ok(values.to_vec())
}

pub(super) fn select_unique(
    candidates: Vec<String>,
    complete: bool,
    reason: &str,
) -> SourceSelection {
    if !complete {
        return SourceSelection::Unavailable {
            reason: "source enumeration is incomplete".into(),
        };
    }
    match candidates.as_slice() {
        [] => SourceSelection::Absent,
        [identifier] => SourceSelection::Selected {
            identifier: identifier.clone(),
            reason: reason.to_owned(),
        },
        _ => SourceSelection::Ambiguous { candidates },
    }
}
