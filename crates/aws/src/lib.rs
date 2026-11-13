//! Read-only AWS inventory with explicit partial-coverage records.
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub mod cloud_control;
mod common;
pub mod discovery;
mod dynamo;
mod ec2;
mod ecs;
mod eks;
mod global_inventory;
mod lambda;
mod load_balancing;
mod network;
mod source_discovery;
mod supplemental;
mod task_definitions;

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, retry::RetryConfig};
use aws_types::region::Region;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{CollectionEvent, Source};
use inventory_engine::limiter::{ApiError, ApiExecutor};
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use common::{Context, MAX_SCAN_PAGES, required, sdk_error};

const MAX_REGIONS: usize = 64;
const NATIVE_SERVICES: [&str; 8] = [
    "ec2",
    "network",
    "dynamodb",
    "lambda",
    "ecs",
    "ecs-task-definitions",
    "eks",
    "elbv2",
];

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
    pub resource_types: Vec<String>,
    pub max_pages: u32,
    pub page_size: i32,
}

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
    #[error("inventory consumer closed its channel")]
    ChannelClosed,
    #[error("scope child cap would be exceeded before child enrichment")]
    ChildLimit,
}

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
        if !self.cloud_control && !self.resource_types.is_empty() {
            return Err(AwsScanError::InvalidOptions(
                "resource type selection requires Cloud Control".into(),
            ));
        }
        if self.cloud_control {
            cloud_control::validate_selection(&self.resource_types)?;
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
                "lambda" | "ecs-task-definitions" | "eks" => {
                    selected.insert(service.clone());
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

#[cfg(test)]
mod options_property_tests;

#[cfg(test)]
mod tests;

mod scan;
pub use scan::scan;
