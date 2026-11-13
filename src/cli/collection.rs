use super::*;

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScanArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub profile: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub region: Vec<String>,
    #[arg(long, value_delimiter = ',', default_value = "all")]
    pub service: Vec<String>,
    #[arg(long)]
    pub config_aggregator: Option<String>,
    #[arg(long)]
    pub resource_explorer_view: Option<String>,
    /// Discover existing Config/Explorer sources in selected regions; ambiguity stays explicit.
    #[arg(long)]
    pub discover_sources: bool,
    #[command(flatten)]
    pub accounts: AccountArgs,
    /// Add the catalog of regional Cloud Control list/read resource types.
    #[arg(long)]
    pub cloud_control: bool,
    /// CloudFormation type names; empty selects the full Cloud Control catalog.
    #[arg(long, value_delimiter = ',', requires = "cloud_control")]
    pub resource_type: Vec<String>,
    #[arg(long, default_value_t = 128)]
    pub concurrency: usize,
    #[arg(long, default_value_t = 4)]
    pub per_bucket_concurrency: usize,
    /// Conservative initial request-start ceiling per account/region/service bucket.
    #[arg(long, default_value_t = 5)]
    pub rps: u32,
    #[arg(long, default_value_t = 5)]
    pub attempts: u32,
    #[arg(long, default_value_t = 10000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 100)]
    pub page_size: i32,
    /// JSON map of exact quota-bucket keys to integer or requests/period rate policies.
    #[arg(long)]
    pub rate_policy: Option<PathBuf>,
    /// Optional post-collection outputs; old persisted jobs default to no delivery.
    #[command(flatten)]
    #[serde(default)]
    pub delivery: DeliveryArgs,
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    #[arg(long, value_delimiter = ',')]
    pub profile: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub region: Vec<String>,
    #[arg(long, value_delimiter = ',', default_value = "all")]
    pub service: Vec<String>,
    #[arg(long)]
    pub cloud_control: bool,
    #[arg(long, value_delimiter = ',', requires = "cloud_control")]
    pub resource_type: Vec<String>,
    #[arg(long)]
    pub config_aggregator: Option<String>,
    #[arg(long)]
    pub resource_explorer_view: Option<String>,
    #[arg(long)]
    pub discover_sources: bool,
    #[command(flatten)]
    pub accounts: AccountArgs,
}

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountArgs {
    /// Enumerate ACTIVE Organizations accounts and assume this existing role in each.
    #[arg(long, conflicts_with = "role_arn")]
    pub organization_role: Option<String>,
    /// Assume an explicitly named existing role before resolving the target account.
    #[arg(long, conflicts_with = "organization_role")]
    pub role_arn: Option<String>,
    /// Restrict Organizations collection to these exact account IDs.
    #[arg(long, value_delimiter = ',', requires = "organization_role")]
    pub account_id: Vec<String>,
    /// Environment variable containing an optional STS external ID; its value is not persisted.
    #[arg(long)]
    pub external_id_env: Option<String>,
    #[arg(long, default_value = "aws-inventory")]
    pub role_session_name: String,
    #[arg(long, default_value_t = 3600)]
    pub role_duration_seconds: u32,
    /// Fail before scanning when the selected account set exceeds this bound.
    #[arg(long, default_value_t = 128)]
    pub max_accounts: usize,
}

impl Default for AccountArgs {
    fn default() -> Self {
        Self {
            organization_role: None,
            role_arn: None,
            account_id: Vec::new(),
            external_id_env: None,
            role_session_name: "aws-inventory".into(),
            role_duration_seconds: 3600,
            max_accounts: 128,
        }
    }
}

#[derive(Args, Debug)]
pub struct DiscoverArgs {
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "us-east-1")]
    pub region: String,
    #[arg(long)]
    pub organization: bool,
    #[arg(long, requires = "organization")]
    pub hierarchy: bool,
    #[arg(long)]
    pub config_aggregators: bool,
    #[arg(long)]
    pub resource_explorer: bool,
    #[arg(long, default_value_t = 10000)]
    pub max_pages: u32,
    #[arg(long, default_value_t = 100)]
    pub page_size: i32,
}
