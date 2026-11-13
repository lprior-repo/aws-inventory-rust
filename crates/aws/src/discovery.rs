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
pub struct DiscoveryReport {
    pub account_id: String,
    pub partition: String,
    pub region: String,
    pub organization: Option<OrganizationDiscovery>,
    pub supplemental: SupplementalDiscovery,
    pub complete: bool,
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

mod execution;
pub use execution::discover;
pub(crate) use execution::supplemental_sources;
mod roles;
pub use roles::account_role_arn;
pub(crate) use roles::assume_role_config;
use roles::{role_character, validate_role_arn};
mod validation;
use validation::{
    bounded_text, data_error, ensure_running, invalid, optional_text, validate_account_id,
    validate_partition,
};

mod organization_model;
pub use organization_model::*;
mod source_model;
pub use source_model::*;

#[cfg(test)]
use roles::{credential_timeouts, validate_role_path};
