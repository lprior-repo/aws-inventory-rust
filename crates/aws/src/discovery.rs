//! Read-only account and supplemental-source discovery with explicit coverage.
//!
//! Discovery never creates AWS resources. A failed listing retains preceding
//! observations and cannot produce a selected source or a complete account set.

mod organizations;
mod pages;
mod sources;

use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use aws_config::{BehaviorVersion, retry::RetryConfig, sts::AssumeRoleProvider};
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::{SdkConfig, region::Region};
use futures_util::future::BoxFuture;
use inventory_engine::limiter::{ApiError, ApiExecutor};
use serde::{Deserialize, Serialize};

use crate::AwsScanError;
use crate::common::{Context, MAX_SCAN_PAGES, error_category, required, sdk_error};

pub const MAX_DISCOVERED_ACCOUNTS: usize = 100_000;
pub const MAX_DISCOVERED_UNITS: usize = 10_000;
pub const MAX_DISCOVERED_SOURCES: usize = 1_000;
const MAX_FAILURES: usize = 1_024;
const MAX_METADATA_BYTES: usize = 8_192;
const CONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

/// One endpoint region is explicit. Organization requests use the SDK's global
/// endpoint resolution; Config and Resource Explorer discovery are regional.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryOptions {
    pub profile: Option<String>,
    pub region: String,
    pub organization: bool,
    pub hierarchy: bool,
    pub config_aggregators: bool,
    pub resource_explorer: bool,
    pub max_pages: u32,
    pub page_size: i32,
}

impl DiscoveryOptions {
    pub fn validate(&self) -> Result<(), AwsScanError> {
        crate::validate_region(&self.region)?;
        if !(1..=MAX_SCAN_PAGES).contains(&self.max_pages)
            || !(1..=1_000).contains(&self.page_size)
            || self.hierarchy && !self.organization
        {
            return invalid(
                "discovery requires bounded pages/size and hierarchy requires organization",
            );
        }
        if let Some(profile) = &self.profile {
            bounded_text(profile, "profile", 2_048)?;
        }
        if !(self.organization || self.config_aggregators || self.resource_explorer) {
            return invalid("at least one discovery source must be selected");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryFailure {
    pub operation: String,
    pub parent_id: Option<String>,
    pub category: String,
    pub message: String,
}

/// Failure records have a fixed cap. Suppressed records remain counted, so
/// exhaustion of diagnostics cannot turn partial work into successful work.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryCoverage {
    pub pages: u64,
    pub items_seen: u64,
    pub complete: bool,
    pub cancelled: bool,
    pub failures: Vec<DiscoveryFailure>,
    pub suppressed_failures: u64,
}

impl DiscoveryCoverage {
    fn success() -> Self {
        Self {
            complete: true,
            ..Self::default()
        }
    }

    fn fail(&mut self, operation: &str, parent: Option<&str>, error: &AwsScanError) {
        self.complete = false;
        self.cancelled |= matches!(error, AwsScanError::Api(ApiError::Cancelled));
        if self.failures.len() >= MAX_FAILURES {
            self.suppressed_failures = self.suppressed_failures.saturating_add(1);
            return;
        }
        self.failures.push(DiscoveryFailure {
            operation: operation.to_owned(),
            parent_id: parent.map(str::to_owned),
            category: error_category(error).to_owned(),
            message: error.to_string().chars().take(2_048).collect(),
        });
    }

    fn merge(&mut self, other: Self) {
        self.pages = self.pages.saturating_add(other.pages);
        self.items_seen = self.items_seen.saturating_add(other.items_seen);
        self.complete &= other.complete;
        self.cancelled |= other.cancelled;
        self.suppressed_failures = self
            .suppressed_failures
            .saturating_add(other.suppressed_failures);
        let remaining = MAX_FAILURES.saturating_sub(self.failures.len());
        let excess = other.failures.len().saturating_sub(remaining);
        self.suppressed_failures = self
            .suppressed_failures
            .saturating_add(u64::try_from(excess).unwrap_or(u64::MAX));
        self.failures
            .extend(other.failures.into_iter().take(remaining));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationAccount {
    pub id: String,
    pub arn: Option<String>,
    pub name: Option<String>,
    /// Uses the current State field, never the retired Status field.
    pub state: Option<String>,
    pub eligible_for_scan: bool,
    pub joined_method: Option<String>,
    pub joined_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationNode {
    pub id: String,
    pub arn: Option<String>,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub depth: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationMembership {
    pub account_id: String,
    pub parent_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDiscovery {
    pub accounts: Vec<OrganizationAccount>,
    pub account_coverage: DiscoveryCoverage,
    pub hierarchy_requested: bool,
    pub roots: Vec<OrganizationNode>,
    pub organizational_units: Vec<OrganizationNode>,
    pub memberships: Vec<OrganizationMembership>,
    pub hierarchy_coverage: Option<DiscoveryCoverage>,
}

impl OrganizationDiscovery {
    pub fn complete(&self) -> bool {
        self.account_coverage.complete
            && self
                .hierarchy_coverage
                .as_ref()
                .is_none_or(|coverage| coverage.complete)
    }

    pub fn eligible_accounts(&self) -> impl Iterator<Item = &OrganizationAccount> {
        self.accounts
            .iter()
            .filter(|account| account.eligible_for_scan)
    }
}

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryReport {
    pub account_id: String,
    pub partition: String,
    pub region: String,
    pub organization: Option<OrganizationDiscovery>,
    pub supplemental: SupplementalDiscovery,
    pub complete: bool,
}

/// A bounded bootstrap/discovery operation. Credential configuration failures
/// return Err; service-listing failures stay in the report with partial data.
#[tracing::instrument(skip_all, fields(region = %options.region))]
pub async fn discover(
    options: DiscoveryOptions,
    executor: Arc<ApiExecutor>,
) -> Result<DiscoveryReport, AwsScanError> {
    // Keep the SDK future tree out of downstream worker/CLI future layouts.
    // The caller still owns and awaits the single discovery future.
    let work: BoxFuture<'static, Result<DiscoveryReport, AwsScanError>> =
        Box::pin(discover_inner(options, executor));
    work.await
}

#[tracing::instrument(skip_all)]
async fn discover_inner(
    options: DiscoveryOptions,
    executor: Arc<ApiExecutor>,
) -> Result<DiscoveryReport, AwsScanError> {
    options.validate()?;
    ensure_running(&executor)?;
    let config = load_configuration(&options, &executor).await?;
    let (account, partition) = caller_identity(&config, &options.region, &executor).await?;
    let client = DiscoveryClient {
        config: &config,
        executor: &executor,
        account: &account,
        partition: &partition,
        region: &options.region,
        max_pages: options.max_pages,
        page_size: options.page_size,
    };
    discover_with_client(&client, &options).await
}

#[tracing::instrument(skip_all)]
async fn discover_with_client(
    client: &DiscoveryClient<'_>,
    options: &DiscoveryOptions,
) -> Result<DiscoveryReport, AwsScanError> {
    let organizations = async {
        if options.organization {
            Some(organizations::collect(client, options.hierarchy).await)
        } else {
            None
        }
    };
    let (organization, supplemental) = tokio::join!(
        organizations,
        sources::collect(
            client,
            options.config_aggregators,
            options.resource_explorer
        )
    );
    let complete = organization
        .as_ref()
        .is_none_or(OrganizationDiscovery::complete)
        && supplemental.complete();
    Ok(DiscoveryReport {
        account_id: client.account.to_owned(),
        partition: client.partition.to_owned(),
        region: client.region.to_owned(),
        organization,
        supplemental,
        complete,
    })
}

#[tracing::instrument(skip_all)]
async fn load_configuration(
    options: &DiscoveryOptions,
    executor: &ApiExecutor,
) -> Result<SdkConfig, AwsScanError> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(options.region.clone()))
        .retry_config(RetryConfig::standard().with_max_attempts(1));
    if let Some(profile) = &options.profile {
        loader = loader.profile_name(profile);
    }
    tokio::select! {
        biased;
        () = executor.cancelled() => Err(ApiError::Cancelled.into()),
        result = tokio::time::timeout(CONFIGURATION_TIMEOUT, loader.load()) =>
            result.map_err(|_| AwsScanError::Api(ApiError::Timeout)),
    }
}

#[tracing::instrument(skip_all)]
async fn caller_identity(
    config: &SdkConfig,
    region: &str,
    executor: &ApiExecutor,
) -> Result<(String, String), AwsScanError> {
    let sts = aws_sdk_sts::Client::new(config);
    let bucket = format!("identity:{region}:sts");
    let identity = executor
        .execute(&bucket, || async {
            sts.get_caller_identity().send().await.map_err(sdk_error)
        })
        .await?;
    let account = required(identity.account(), "STS account")?;
    validate_account_id(&account)?;
    let arn = required(identity.arn(), "STS caller ARN")?;
    let partition = arn
        .split(':')
        .nth(1)
        .ok_or_else(|| data_error("invalid STS caller ARN"))?;
    validate_partition(partition)?;
    Ok((account, partition.to_owned()))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn supplemental_sources(
    ctx: &Context,
    config: bool,
    explorer: bool,
) -> SupplementalDiscovery {
    sources::collect(&DiscoveryClient::from(ctx), config, explorer).await
}

/// Shares admission with the native collectors without inventing an event
/// receiver for a report-only discovery command.
struct DiscoveryClient<'a> {
    config: &'a SdkConfig,
    executor: &'a ApiExecutor,
    account: &'a str,
    partition: &'a str,
    region: &'a str,
    max_pages: u32,
    page_size: i32,
}

impl<'a> From<&'a Context> for DiscoveryClient<'a> {
    fn from(context: &'a Context) -> Self {
        Self {
            config: &context.config,
            executor: &context.executor,
            account: &context.account,
            partition: &context.partition,
            region: &context.region,
            max_pages: context.max_pages,
            page_size: context.page_size,
        }
    }
}

impl DiscoveryClient<'_> {
    #[tracing::instrument(skip_all, fields(service))]
    async fn execute<T, F, Fut>(
        &self,
        service: &str,
        global: bool,
        operation: F,
    ) -> Result<T, AwsScanError>
    where
        T: Send,
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, ApiError>> + Send,
    {
        let region = if global { "global" } else { self.region };
        let bucket = format!(
            "{}:{}:{region}:{service}:discovery",
            self.partition, self.account
        );
        self.executor
            .execute(&bucket, operation)
            .await
            .map_err(AwsScanError::Api)
    }
}

/// Only the role ARN/session configuration is retained. Credentials are supplied
/// lazily by the SDK and refreshed through its normal identity cache.
#[derive(Clone)]
pub struct AssumeRoleOptions {
    pub role_arn: String,
    pub session_name: String,
    pub external_id: Option<String>,
    pub duration_seconds: u32,
}

impl fmt::Debug for AssumeRoleOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssumeRoleOptions")
            .field("role_arn", &self.role_arn)
            .field("session_name", &self.session_name)
            .field("external_id_present", &self.external_id.is_some())
            .field("duration_seconds", &self.duration_seconds)
            .finish()
    }
}

impl AssumeRoleOptions {
    pub fn validate(&self) -> Result<(), AwsScanError> {
        validate_role_arn(&self.role_arn)?;
        if !(2..=64).contains(&self.session_name.len())
            || !self.session_name.bytes().all(role_character)
            || !(900..=3_600).contains(&self.duration_seconds)
        {
            return invalid(
                "role session requires 2..=64 valid name bytes and duration 900..=3600 seconds",
            );
        }
        if self.external_id.as_ref().is_some_and(|value| {
            !(2..=1_224).contains(&value.len())
                || !value
                    .bytes()
                    .all(|byte| role_character(byte) || matches!(byte, b':' | b'/'))
        }) {
            return invalid("invalid external ID syntax or length");
        }
        Ok(())
    }
}

/// Construct an ARN for an existing role only. The name/path contains no
/// wildcard, empty path segment, relative segment, URL escape, or control byte.
pub fn account_role_arn(
    partition: &str,
    account_id: &str,
    role_name_or_path: &str,
) -> Result<String, AwsScanError> {
    validate_partition(partition)?;
    validate_account_id(account_id)?;
    validate_role_path(role_name_or_path)?;
    Ok(format!(
        "arn:{partition}:iam::{account_id}:role/{role_name_or_path}"
    ))
}

/// Configure AWS's refreshable AssumeRole provider. Building it performs no
/// explicit inventory request; its later STS credential requests are not
/// counted by ApiExecutor. Credential requests have a 30-second SDK deadline;
/// the consuming operation remains cancellation- and timeout-bound. Credentials
/// and external IDs are never put into reports.
#[tracing::instrument(skip_all)]
pub async fn assume_role_config(
    base: &SdkConfig,
    options: &AssumeRoleOptions,
    executor: &ApiExecutor,
) -> Result<SdkConfig, AwsScanError> {
    options.validate()?;
    ensure_running(executor)?;
    if base.credentials_provider().is_none()
        || base.time_source().is_none()
        || base.region().is_none()
    {
        return invalid(
            "AssumeRole requires a loaded base SDK configuration with credentials, time and region providers",
        );
    }
    let mut builder = base
        .to_builder()
        .behavior_version(BehaviorVersion::latest())
        .retry_config(RetryConfig::standard().with_max_attempts(1));
    // Service defaults inspect origin metadata. In particular, recent
    // DynamoDB clients replace an unknown-origin retry limit with four attempts.
    builder.insert_origin("retry_config", aws_types::origin::Origin::shared_config());
    let base = builder.build();
    let credentials_config = base
        .to_builder()
        .timeout_config(credential_timeouts(base.timeout_config()))
        .build();
    let mut builder = AssumeRoleProvider::builder(&options.role_arn)
        .configure(&credentials_config)
        .session_name(&options.session_name)
        .session_length(Duration::from_secs(u64::from(options.duration_seconds)));
    if let Some(external_id) = &options.external_id {
        builder = builder.external_id(external_id);
    }
    let provider = tokio::select! {
        biased;
        () = executor.cancelled() => return Err(ApiError::Cancelled.into()),
        result = tokio::time::timeout(CONFIGURATION_TIMEOUT, builder.build()) =>
            result.map_err(|_| ApiError::Timeout)?,
    };
    Ok(base
        .to_builder()
        .credentials_provider(SharedCredentialsProvider::new(provider))
        .build())
}

fn credential_timeouts(existing: Option<&TimeoutConfig>) -> TimeoutConfig {
    let bound = |value: Option<Duration>, limit: Duration| value.unwrap_or(limit).min(limit);
    TimeoutConfig::builder()
        .operation_timeout(bound(
            existing.and_then(TimeoutConfig::operation_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .operation_attempt_timeout(bound(
            existing.and_then(TimeoutConfig::operation_attempt_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .connect_timeout(bound(
            existing.and_then(TimeoutConfig::connect_timeout),
            Duration::from_secs(10),
        ))
        .read_timeout(bound(
            existing.and_then(TimeoutConfig::read_timeout),
            CONFIGURATION_TIMEOUT,
        ))
        .build()
}

fn validate_role_arn(arn: &str) -> Result<(), AwsScanError> {
    bounded_text(arn, "role ARN", 2_048)?;
    let mut fields = arn.splitn(6, ':');
    if fields.next() != Some("arn") {
        return invalid("role ARN must start with arn");
    }
    let partition = fields
        .next()
        .ok_or_else(|| data_error("missing role ARN partition"))?;
    validate_partition(partition)?;
    if fields.next() != Some("iam") || fields.next() != Some("") {
        return invalid("role ARN must name IAM without a region");
    }
    validate_account_id(
        fields
            .next()
            .ok_or_else(|| data_error("missing role ARN account"))?,
    )?;
    let path = fields
        .next()
        .and_then(|value| value.strip_prefix("role/"))
        .ok_or_else(|| data_error("ARN must identify an IAM role"))?;
    validate_role_path(path)
}

fn validate_role_path(path: &str) -> Result<(), AwsScanError> {
    if path.is_empty()
        || path.len() > 575
        || !path
            .bytes()
            .all(|byte| role_character(byte) || byte == b'/')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || path.rsplit('/').next().is_none_or(|name| name.len() > 64)
        || path
            .rsplit_once('/')
            .is_some_and(|(prefix, _)| prefix.len().saturating_add(2) > 512)
    {
        return invalid("invalid IAM role name/path");
    }
    Ok(())
}

fn role_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'=' | b',' | b'.' | b'@' | b'-')
}

fn validate_account_id(account: &str) -> Result<(), AwsScanError> {
    if account.len() != 12 || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return invalid("AWS account ID must have exactly 12 decimal digits");
    }
    Ok(())
}

fn validate_partition(partition: &str) -> Result<(), AwsScanError> {
    if !(3..=32).contains(&partition.len())
        || !partition.split('-').enumerate().all(|(position, segment)| {
            if position == 0 {
                segment == "aws"
            } else {
                !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_lowercase())
            }
        })
    {
        return invalid("invalid AWS partition");
    }
    Ok(())
}

fn bounded_text(value: &str, label: &str, limit: usize) -> Result<(), AwsScanError> {
    if value.is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(data_error(&format!("invalid or oversized {label}")));
    }
    Ok(())
}

fn optional_text(
    value: Option<&str>,
    label: &str,
    limit: usize,
) -> Result<Option<String>, AwsScanError> {
    value
        .map(|value| {
            bounded_text(value, label, limit)?;
            Ok(value.to_owned())
        })
        .transpose()
}

fn data_error(message: &str) -> AwsScanError {
    AwsScanError::InvalidData(message.to_owned())
}
fn invalid<T>(message: &str) -> Result<T, AwsScanError> {
    Err(AwsScanError::InvalidOptions(message.to_owned()))
}
fn ensure_running(executor: &ApiExecutor) -> Result<(), AwsScanError> {
    if executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;
