//! source model model.
use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SourceSelection {
    #[default]
    NotRequested,
    Absent,
    Selected {
        identifier: String,
        reason: String,
    },
    Ambiguous {
        candidates: Vec<String>,
    },
    Unavailable {
        reason: String,
    },
}

impl SourceSelection {
    pub fn selected_identifier(&self) -> Option<&str> {
        match self {
            Self::Selected { identifier, .. } => Some(identifier),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountAggregationScope {
    pub account_ids: Vec<String>,
    pub all_regions: bool,
    pub regions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigAggregator {
    pub name: String,
    pub arn: String,
    pub account_sources: Vec<AccountAggregationScope>,
    pub organization_source: bool,
    pub organization_all_regions: bool,
    pub organization_regions: Vec<String>,
    /// Filters can exclude recorded resources; their presence is retained.
    pub has_filters: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDiscovery {
    pub candidates: Vec<ConfigAggregator>,
    pub selection: SourceSelection,
    pub coverage: Option<DiscoveryCoverage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplorerIndex {
    pub arn: String,
    pub index_type: Option<String>,
    pub state: Option<String>,
    pub replicating_from: Vec<String>,
    pub replicating_to: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplorerView {
    pub arn: String,
    pub owner: Option<String>,
    pub scope: Option<String>,
    /// The SDK marks filter strings sensitive. Only their presence is retained.
    pub has_filter: bool,
    pub included_properties: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplorerDiscovery {
    pub index: Option<ExplorerIndex>,
    pub candidates: Vec<String>,
    pub default_view: Option<String>,
    pub selected_view: Option<ExplorerView>,
    pub selection: SourceSelection,
    pub coverage: Option<DiscoveryCoverage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupplementalDiscovery {
    pub config: ConfigDiscovery,
    pub resource_explorer: ExplorerDiscovery,
}

impl SupplementalDiscovery {
    pub fn complete(&self) -> bool {
        self.config
            .coverage
            .as_ref()
            .is_none_or(|coverage| coverage.complete)
            && self
                .resource_explorer
                .coverage
                .as_ref()
                .is_none_or(|coverage| coverage.complete)
    }
}
