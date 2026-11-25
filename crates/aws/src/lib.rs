#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub mod model_credentials;

#[cfg(feature = "adapters")]
pub mod cloud_control;
#[cfg(feature = "adapters")]
mod common;
#[cfg(feature = "adapters")]
pub mod discovery;
#[cfg(feature = "adapters")]
mod dynamo;
#[cfg(feature = "adapters")]
mod ec2;
#[cfg(feature = "adapters")]
mod ecs;
#[cfg(feature = "adapters")]
mod eks;
#[cfg(feature = "adapters")]
mod global_inventory;
#[cfg(feature = "adapters")]
mod kms;
#[cfg(feature = "adapters")]
mod lambda;
#[cfg(feature = "adapters")]
mod load_balancing;
#[cfg(feature = "adapters")]
mod network;
#[cfg(feature = "adapters")]
mod sns;
#[cfg(feature = "adapters")]
mod source_discovery;
#[cfg(feature = "adapters")]
mod sqs;
#[cfg(feature = "adapters")]
mod step_functions;
#[cfg(feature = "adapters")]
mod supplemental;
#[cfg(feature = "adapters")]
mod task_definitions;

#[cfg(feature = "adapters")]
use std::collections::BTreeSet;
#[cfg(feature = "adapters")]
use std::sync::Arc;
#[cfg(feature = "adapters")]
use std::time::Duration;

#[cfg(feature = "adapters")]
use aws_config::meta::region::RegionProviderChain;
#[cfg(feature = "adapters")]
use aws_config::{BehaviorVersion, retry::RetryConfig};
#[cfg(feature = "adapters")]
use aws_types::region::Region;
#[cfg(feature = "adapters")]
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
#[cfg(feature = "adapters")]
use inventory_core::{CollectionEvent, Selection, Source};
#[cfg(feature = "adapters")]
use inventory_engine::limiter::{ApiError, ApiExecutor};
#[cfg(feature = "adapters")]
use thiserror::Error;
#[cfg(feature = "adapters")]
use tokio::sync::mpsc::Sender;

#[cfg(feature = "adapters")]
use common::{Context, MAX_SCAN_PAGES, required, sdk_error};

#[cfg(feature = "adapters")]
const MAX_REGIONS: usize = 64;
#[cfg(feature = "adapters")]
const NATIVE_SERVICES: [&str; 12] = [
    "ec2",
    "network",
    "dynamodb",
    "lambda",
    "ecs",
    "ecs-task-definitions",
    "eks",
    "elbv2",
    "sqs",
    "sns",
    "kms",
    "step-functions",
];

#[cfg(feature = "adapters")]
#[derive(Clone, Debug)]
pub struct AwsScanOptions {
    pub profile: Option<String>,
    pub regions: Vec<String>,
    pub services: Vec<String>,
    pub config_aggregator: Option<String>,
    pub resource_explorer_view: Option<String>,
    pub discover_sources: bool,
    pub assume_role: Option<discovery::AssumeRoleOptions>,
    pub cloud_control: bool,
    pub selection: Selection,
    pub max_pages: u32,
    pub page_size: i32,
}

#[cfg(feature = "adapters")]
#[derive(Debug, Error)]
pub enum AwsScanError {
    #[error("invalid scan options: {0}")]
    InvalidOptions(String),
    #[error("invalid AWS response: {0}")]
    InvalidData(String),
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(transparent)]
    Validation(#[from] inventory_core::ValidationError),
    #[error(transparent)]
    TrustSummary(#[from] inventory_core::iam_trust::TrustSummaryError),
    #[error("inventory consumer closed its channel")]
    ChannelClosed,
    #[error("scope child cap would be exceeded before child enrichment")]
    ChildLimit,
}

#[cfg(feature = "adapters")]
impl AwsScanOptions {
    fn validate(&self) -> Result<BTreeSet<String>, AwsScanError> {
        if let Some(role) = &self.assume_role {
            role.validate()?;
        }
        if !(1..=MAX_SCAN_PAGES).contains(&self.max_pages) || !(1..=1_000).contains(&self.page_size)
        {
            return Err(AwsScanError::InvalidOptions(
                "max_pages must be 1..=10000 and page_size 1..=1000".into(),
            ));
        }
        if self.regions.len() > MAX_REGIONS || self.services.len() > 32 {
            return Err(AwsScanError::InvalidOptions(
                "at most 64 regions and 32 service selectors are accepted".into(),
            ));
        }
        for region in &self.regions {
            validate_region(region)?;
        }
        if !self.cloud_control && !self.selection.is_empty() {
            return Err(AwsScanError::InvalidOptions(
                "resource type selection requires Cloud Control".into(),
            ));
        }
        if self.cloud_control {
            cloud_control::validate_selection(&self.selection)?;
        }
        for (label, value) in [
            ("profile", &self.profile),
            ("Config aggregator", &self.config_aggregator),
            ("Resource Explorer view", &self.resource_explorer_view),
        ] {
            if value.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 2_048 || value.contains('\0')
            }) {
                return Err(AwsScanError::InvalidOptions(format!("invalid {label}")));
            }
        }
        let mut selected = BTreeSet::new();
        if self.services.is_empty() || self.services.iter().any(|service| service == "all") {
            selected.extend(NATIVE_SERVICES.into_iter().map(str::to_owned));
            selected.extend(
                global_inventory::GLOBAL_SERVICES
                    .into_iter()
                    .map(str::to_owned),
            );
        }
        for service in &self.services {
            match service.as_str() {
                "all" => {}
                "ec2" | "vpc" | "ebs" => {
                    selected.insert("ec2".to_owned());
                    selected.insert("network".to_owned());
                }
                "network" => {
                    selected.insert("network".to_owned());
                }
                "elbv2" | "elb" | "load-balancing" => {
                    selected.insert("elbv2".to_owned());
                }
                "dynamodb" | "dynamo" => {
                    selected.insert("dynamodb".to_owned());
                }
                "ecs" => {
                    selected.insert("ecs".to_owned());
                    selected.insert("ecs-task-definitions".to_owned());
                }
                "lambda"
                | "ecs-task-definitions"
                | "eks"
                | "sqs"
                | "sns"
                | "kms"
                | "step-functions" => {
                    selected.insert(service.clone());
                }
                "stepfunctions" | "sfn" => {
                    selected.insert("step-functions".to_owned());
                }
                "s3" | "iam" | "route53" | "cloudfront" => {
                    selected.insert(service.clone());
                }
                "config" if self.config_aggregator.is_some() || self.discover_sources => {}
                "resource-explorer"
                    if self.resource_explorer_view.is_some() || self.discover_sources => {}
                "cloud-control" if self.cloud_control => {}
                _ => {
                    return Err(AwsScanError::InvalidOptions(format!(
                        "unsupported service selector or missing supplemental source: {service}"
                    )));
                }
            }
        }
        Ok(selected)
    }
}

#[cfg(feature = "adapters")]
fn validate_region(region: &str) -> Result<(), AwsScanError> {
    if region.is_empty()
        || region.len() > 64
        || !region
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(AwsScanError::InvalidOptions(format!(
            "invalid region: {region}"
        )));
    }
    Ok(())
}

#[cfg(feature = "adapters")]
#[cfg(test)]
mod options_property_tests;

#[cfg(feature = "adapters")]
#[cfg(test)]
mod tests;

#[cfg(feature = "adapters")]
mod scan;
#[cfg(feature = "adapters")]
pub use scan::scan;
