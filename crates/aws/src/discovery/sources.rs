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

#[cfg(test)]
mod tests;

mod configuration;
use configuration::{collect_config, regions, select_unique};
mod explorer;
use explorer::collect_explorer;
mod views;
use views::{
    fail_explorer, get_default_view, get_view, select_view_candidate, validate_scoped_arn,
};

#[cfg(test)]
use views::map_view;

#[cfg(test)]
use configuration::map_aggregator;
