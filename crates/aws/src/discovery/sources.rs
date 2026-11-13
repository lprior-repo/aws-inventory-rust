//! Existing Config and Resource Explorer source selection. Selection requires
//! complete evidence; multiple Config candidates are never chosen arbitrarily.

use aws_sdk_config::types::ConfigurationAggregator;
use aws_sdk_resourceexplorer2::operation::get_index::GetIndexOutput;
use aws_sdk_resourceexplorer2::types::View;

use super::pages::{self, Identified, Page};
use super::{
    AccountAggregationScope, ConfigAggregator, ConfigDiscovery, DiscoveryClient, DiscoveryCoverage,
    ExplorerDiscovery, ExplorerIndex, ExplorerView, MAX_DISCOVERED_SOURCES, SourceSelection,
    SupplementalDiscovery, bounded_text, data_error, optional_text, validate_account_id,
};
use crate::AwsScanError;
use crate::common::{required, sdk_error};

const MAX_CONFIG_SOURCE_GROUPS: usize = 16;
const MAX_CONFIG_ACCOUNT_IDS: usize = 10_000;
const MAX_REGIONS: usize = 64;

impl Identified for ConfigAggregator {
    fn identifier(&self) -> &str {
        &self.name
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn collect(
    client: &DiscoveryClient<'_>,
    config: bool,
    explorer: bool,
) -> SupplementalDiscovery {
    let config_work = async {
        if config {
            collect_config(client).await
        } else {
            ConfigDiscovery::default()
        }
    };
    let explorer_work = async {
        if explorer {
            collect_explorer(client).await
        } else {
            ExplorerDiscovery::default()
        }
    };
    let (config, resource_explorer) = tokio::join!(config_work, explorer_work);
    SupplementalDiscovery {
        config,
        resource_explorer,
    }
}

#[tracing::instrument(skip_all)]
async fn collect_config(client: &DiscoveryClient<'_>) -> ConfigDiscovery {
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

fn map_aggregator(
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

fn map_aggregation_scopes(
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

fn regions(values: &[String]) -> Result<Vec<String>, AwsScanError> {
    if values.len() > MAX_REGIONS {
        return Err(data_error("regional metadata exceeds 64 entries"));
    }
    values.iter().try_for_each(|region| {
        crate::validate_region(region).map_err(|_| data_error("invalid discovered source region"))
    })?;
    Ok(values.to_vec())
}

fn select_unique(candidates: Vec<String>, complete: bool, reason: &str) -> SourceSelection {
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

#[tracing::instrument(skip_all)]
async fn collect_explorer(client: &DiscoveryClient<'_>) -> ExplorerDiscovery {
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
async fn get_index(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_resourceexplorer2::Client,
) -> Result<Option<ExplorerIndex>, AwsScanError> {
    let output = client
        .execute("resource-explorer-2", false, || async {
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

fn map_index(
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
async fn list_explorer_views(
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
                .execute("resource-explorer-2", false, || async {
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
async fn select_explorer_view(
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

fn select_view_candidate(candidates: &[String], default: Option<&str>) -> SourceSelection {
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
async fn get_default_view(
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
async fn get_view(
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

fn map_view(
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

fn fail_explorer(result: &mut ExplorerDiscovery, operation: &str, error: &AwsScanError) {
    if let Some(coverage) = &mut result.coverage {
        coverage.fail(operation, None, error);
    }
    result.selection = SourceSelection::Unavailable {
        reason: format!("{operation} failed; inspect discovery coverage"),
    };
}

fn validate_scoped_arn(
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

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::BehaviorVersion;
    use aws_types::{SdkConfig, region::Region};
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};

    fn config() -> SdkConfig {
        SdkConfig::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .build()
    }
    fn client<'a>(config: &'a SdkConfig, executor: &'a ApiExecutor) -> DiscoveryClient<'a> {
        DiscoveryClient {
            config,
            executor,
            account: "123456789012",
            partition: "aws",
            region: "us-east-1",
            max_pages: 10,
            page_size: 100,
        }
    }

    #[test]
    fn unique_source_selection_never_uses_partial_or_ambiguous_evidence() {
        assert_eq!(
            select_unique(vec![], true, "fixture"),
            SourceSelection::Absent
        );
        assert!(matches!(
            select_unique(vec!["one".into()], true, "fixture"),
            SourceSelection::Selected { .. }
        ));
        assert!(matches!(
            select_unique(vec!["one".into(), "two".into()], true, "fixture"),
            SourceSelection::Ambiguous { .. }
        ));
        assert!(matches!(
            select_unique(vec!["one".into()], false, "fixture"),
            SourceSelection::Unavailable { .. }
        ));
    }

    #[test]
    fn regional_default_is_selected_only_when_observed_in_complete_listing() {
        let candidates = vec!["one".into(), "two".into()];
        assert_eq!(
            select_view_candidate(&candidates, Some("two")).selected_identifier(),
            Some("two")
        );
        assert!(matches!(
            select_view_candidate(&candidates, Some("missing")),
            SourceSelection::Unavailable { .. }
        ));
        assert!(matches!(
            select_view_candidate(&candidates, None),
            SourceSelection::Ambiguous { .. }
        ));
    }

    #[test]
    fn sources_are_account_and_region_scoped() -> Result<(), Box<dyn std::error::Error>> {
        let config = config();
        let executor = ApiExecutor::new(ExecutorConfig::default())?;
        let client = client(&config, &executor);
        assert!(
            validate_scoped_arn(
                &client,
                "arn:aws:resource-explorer-2:us-east-1:123456789012:view/all/id",
                "resource-explorer-2",
                "view/"
            )
            .is_ok()
        );
        assert!(
            validate_scoped_arn(
                &client,
                "arn:aws:resource-explorer-2:us-west-2:123456789012:view/all/id",
                "resource-explorer-2",
                "view/"
            )
            .is_err()
        );
        assert!(
            validate_scoped_arn(
                &client,
                "arn:aws:resource-explorer-2:us-east-1:223456789012:view/all/id",
                "resource-explorer-2",
                "view/"
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn view_filter_values_are_never_persisted() -> Result<(), Box<dyn std::error::Error>> {
        let config = config();
        let executor = ApiExecutor::new(ExecutorConfig::default())?;
        let client = client(&config, &executor);
        let arn = "arn:aws:resource-explorer-2:us-east-1:123456789012:view/all/id";
        let view = View::builder()
            .view_arn(arn)
            .owner(client.account)
            .filters(
                aws_sdk_resourceexplorer2::types::SearchFilter::builder()
                    .filter_string("tag:token=DO_NOT_PERSIST")
                    .build()?,
            )
            .build();
        let mapped = map_view(&client, arn, view)?;
        assert!(mapped.has_filter);
        assert!(!serde_json::to_string(&mapped)?.contains("DO_NOT_PERSIST"));
        Ok(())
    }

    #[test]
    fn config_aggregator_metadata_retains_declared_scope_and_filter_presence()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config();
        let executor = ApiExecutor::new(ExecutorConfig::default())?;
        let client = client(&config, &executor);
        let source = aws_sdk_config::types::AccountAggregationSource::builder()
            .account_ids("123456789012")
            .aws_regions("us-east-1")
            .build()?;
        let value = ConfigurationAggregator::builder()
            .configuration_aggregator_name("audit")
            .configuration_aggregator_arn(
                "arn:aws:config:us-east-1:123456789012:config-aggregator/example",
            )
            .account_aggregation_sources(source)
            .build();
        let mapped = map_aggregator(&client, value)?;
        assert_eq!(mapped.name, "audit");
        assert!(!mapped.organization_source);
        assert_eq!(
            mapped
                .account_sources
                .first()
                .map(|source| source.account_ids.as_slice()),
            Some(["123456789012".to_owned()].as_slice())
        );
        Ok(())
    }

    #[test]
    fn view_metadata_must_match_the_selected_identity() -> Result<(), Box<dyn std::error::Error>> {
        let config = config();
        let executor = ApiExecutor::new(ExecutorConfig::default())?;
        let client = client(&config, &executor);
        let arn = "arn:aws:resource-explorer-2:us-east-1:123456789012:view/other/id";
        assert!(map_view(&client, "different", View::builder().view_arn(arn).build()).is_err());
        assert!(
            map_view(
                &client,
                arn,
                View::builder().view_arn(arn).owner("223456789012").build()
            )
            .is_err()
        );
        Ok(())
    }
}
