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

/// Scan one profile. The caller owns the bounded channel and persists events.
/// No AWS services, indexes, recorders, views, or roles are created or changed.
pub async fn scan(
    options: AwsScanOptions,
    executor: Arc<ApiExecutor>,
    sender: Sender<CollectionEvent>,
) -> Result<(), AwsScanError> {
    // Erase the SDK future tree at the crate boundary. This is one allocation
    // per profile, and prevents every downstream caller expanding its layout.
    let work: futures_util::future::BoxFuture<'static, Result<(), AwsScanError>> =
        Box::pin(scan_inner(options, executor, sender));
    work.await
}

#[tracing::instrument(skip_all, fields(profile = options.profile.as_deref().unwrap_or("default")))]
async fn scan_inner(
    options: AwsScanOptions,
    executor: Arc<ApiExecutor>,
    sender: Sender<CollectionEvent>,
) -> Result<(), AwsScanError> {
    if executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let services = Arc::new(options.validate()?);
    let resource_types = Arc::new(if options.cloud_control {
        cloud_control::validate_selection(&options.resource_types)?
    } else {
        Vec::new()
    });
    let mut region_builder = aws_config::default_provider::region::Builder::default();
    if let Some(profile) = &options.profile {
        region_builder = region_builder.profile_name(profile);
    }
    let region_provider =
        RegionProviderChain::first_try(options.regions.first().cloned().map(Region::new))
            .or_else(region_builder.build())
            .or_else(Region::new("us-east-1"));
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .retry_config(RetryConfig::standard().with_max_attempts(1));
    if let Some(profile) = &options.profile {
        loader = loader.profile_name(profile);
    }
    let config = tokio::select! {
        biased;
        () = executor.cancelled() => return Err(ApiError::Cancelled.into()),
        loaded = tokio::time::timeout(Duration::from_secs(30), loader.load()) =>
            loaded.map_err(|_| ApiError::Timeout)?,
    };
    let config = if let Some(role) = &options.assume_role {
        discovery::assume_role_config(&config, role, &executor).await?
    } else {
        config
    };
    let bootstrap_region = config
        .region()
        .map(|region| region.as_ref().to_owned())
        .ok_or_else(|| AwsScanError::InvalidOptions("unable to select bootstrap region".into()))?;
    let sts = aws_sdk_sts::Client::new(&config);
    // Identity is not known before this request. Conservatively share one STS
    // bootstrap bucket per endpoint region across profiles.
    let identity_bucket = format!("identity:{bootstrap_region}:sts");
    let identity = executor
        .execute(&identity_bucket, || async {
            sts.get_caller_identity().send().await.map_err(sdk_error)
        })
        .await?;
    let account = required(identity.account(), "STS account")?;
    if let Some(role) = &options.assume_role
        && role.role_arn.split(':').nth(4) != Some(account.as_str())
    {
        return Err(AwsScanError::InvalidData(
            "assumed role identity does not match the requested target account".into(),
        ));
    }
    let caller_arn = required(identity.arn(), "STS caller ARN")?;
    let partition = caller_arn
        .split(':')
        .nth(1)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AwsScanError::InvalidData("invalid STS caller ARN".into()))?
        .to_owned();
    let base = Context {
        config,
        executor,
        sender,
        account,
        partition,
        region: bootstrap_region,
        max_pages: options.max_pages,
        page_size: options.page_size,
    };
    tracing::info!(account = %base.account, "AWS account identity resolved");

    let auto_config = options.discover_sources && options.config_aggregator.is_none();
    let auto_explorer = options.discover_sources && options.resource_explorer_view.is_none();
    let needs_regions = services
        .iter()
        .any(|service| NATIVE_SERVICES.contains(&service.as_str()))
        || !resource_types.is_empty()
        || auto_config
        || auto_explorer;
    let regions = if !needs_regions {
        Vec::new()
    } else if options.regions.is_empty() {
        match discover_regions(&base).await {
            Ok(regions) => regions,
            Err(error) => {
                base.failure(
                    &base.scope("aws_regions", None, Source::Direct),
                    common::error_category(&error),
                    error.to_string(),
                )
                .await?;
                Vec::new()
            }
        }
    } else {
        options
            .regions
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    let regions: Vec<Context> = regions
        .into_iter()
        .map(|region| region_context(&base, region))
        .collect();
    let regional = stream::iter(regions)
        .map(|context| {
            let services = Arc::clone(&services);
            let resource_types = Arc::clone(&resource_types);
            let work: BoxFuture<'static, Result<(), AwsScanError>> = Box::pin(collect_region(
                context,
                services,
                resource_types,
                auto_config,
                auto_explorer,
            ));
            work
        })
        .buffer_unordered(4)
        .try_for_each(|()| async { Ok(()) });
    let global = async {
        if global_inventory::GLOBAL_SERVICES
            .iter()
            .any(|service| services.contains(*service))
        {
            global_inventory::collect(&base, &services).await?;
        }
        Ok::<(), AwsScanError>(())
    };
    let supplemental = explicit_sources(&base, &options);
    tokio::try_join!(regional, global, supplemental)?;
    Ok(())
}

async fn explicit_sources(base: &Context, options: &AwsScanOptions) -> Result<(), AwsScanError> {
    if let Some(aggregator) = &options.config_aggregator {
        supplemental::config(base, aggregator).await?;
    }
    if let Some(view) = &options.resource_explorer_view {
        let region = view
            .split(':')
            .nth(3)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AwsScanError::InvalidOptions("Resource Explorer view must be a regional ARN".into())
            })?;
        validate_region(region)?;
        supplemental::explorer(&region_context(base, region.to_owned()), view).await?;
    }
    Ok(())
}

fn region_context(base: &Context, region: String) -> Context {
    let config = base
        .config
        .to_builder()
        .region(Region::new(region.clone()))
        .build();
    Context {
        config,
        region,
        ..base.clone()
    }
}

#[tracing::instrument(skip_all)]
async fn discover_regions(ctx: &Context) -> Result<Vec<String>, AwsScanError> {
    let client = aws_sdk_ec2::Client::new(&ctx.config);
    let result = ctx
        .execute("ec2", "describe_regions", || async {
            client
                .describe_regions()
                .all_regions(false)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => {
            let regions: BTreeSet<String> = output
                .regions()
                .iter()
                .filter_map(|region| region.region_name().map(str::to_owned))
                .collect();
            if regions.is_empty() || regions.len() > MAX_REGIONS {
                return Err(AwsScanError::InvalidData(
                    "region discovery returned zero or more than 64 regions".into(),
                ));
            }
            regions
                .iter()
                .try_for_each(|region| validate_region(region))?;
            Ok(regions.into_iter().collect())
        }
        Err(error) => Err(AwsScanError::Api(error)),
    }
}

#[tracing::instrument(skip_all, fields(account = %ctx.account, region = %ctx.region))]
async fn collect_region(
    ctx: Context,
    services: Arc<BTreeSet<String>>,
    resource_types: Arc<Vec<cloud_control::ResourceType>>,
    auto_config: bool,
    auto_explorer: bool,
) -> Result<(), AwsScanError> {
    tracing::info!("collecting AWS region");
    let selected: Vec<&str> = NATIVE_SERVICES
        .into_iter()
        .filter(|service| services.contains(*service))
        .collect();
    let mut work: Vec<BoxFuture<'_, Result<(), AwsScanError>>> = selected
        .into_iter()
        .map(|service| service_future(&ctx, service))
        .collect();
    if !resource_types.is_empty() {
        work.push(Box::pin(cloud_control::collect(&ctx, &resource_types)));
    }
    if auto_config || auto_explorer {
        work.push(Box::pin(source_discovery::collect(
            &ctx,
            auto_config,
            auto_explorer,
        )));
    }
    stream::iter(work)
        .buffer_unordered(10)
        .try_for_each(|()| async { Ok(()) })
        .await
}

fn service_future<'a>(ctx: &'a Context, service: &str) -> BoxFuture<'a, Result<(), AwsScanError>> {
    match service {
        "ec2" => Box::pin(ec2::collect(ctx)),
        "network" => Box::pin(network::collect(ctx)),
        "elbv2" => Box::pin(load_balancing::collect(ctx)),
        "dynamodb" => Box::pin(dynamo::collect(ctx)),
        "lambda" => Box::pin(lambda::collect(ctx)),
        "ecs" => Box::pin(ecs::collect(ctx)),
        "ecs-task-definitions" => Box::pin(task_definitions::collect(ctx)),
        "eks" => Box::pin(eks::collect(ctx)),
        _ => Box::pin(async {
            Err(AwsScanError::InvalidOptions(
                "internal unknown collector".into(),
            ))
        }),
    }
}

#[cfg(test)]
mod options_property_tests;

#[cfg(test)]
mod tests {
    use super::{AwsScanError, AwsScanOptions, scan, validate_region};
    use futures_util::FutureExt;
    use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
    use std::sync::Arc;

    #[test]
    fn selectors_include_new_groups_without_duplicate_collectors() -> Result<(), AwsScanError> {
        let mut options = AwsScanOptions {
            profile: None,
            regions: vec!["us-east-1".into()],
            services: vec![],
            config_aggregator: None,
            resource_explorer_view: None,
            discover_sources: false,
            assume_role: None,
            cloud_control: false,
            resource_types: vec![],
            max_pages: 1,
            page_size: 100,
        };
        let all = options.validate()?;
        assert_eq!(all.len(), 12);
        assert!(all.contains("network"));
        assert!(all.contains("elbv2"));
        options.services = vec!["ec2".into(), "vpc".into(), "network".into()];
        assert_eq!(
            options.validate()?,
            std::collections::BTreeSet::from(["ec2".to_owned(), "network".to_owned(),])
        );
        options.services = vec!["network".into()];
        assert_eq!(
            options.validate()?,
            std::collections::BTreeSet::from(["network".to_owned(),])
        );
        options.services = vec!["elb".into(), "elbv2".into(), "load-balancing".into()];
        assert_eq!(
            options.validate()?,
            std::collections::BTreeSet::from(["elbv2".to_owned(),])
        );
        options.services = vec!["all".into(), "unknown-service".into()];
        assert!(options.validate().is_err());
        Ok(())
    }

    #[test]
    fn supplemental_only_selection_does_not_enable_native_collectors() -> Result<(), AwsScanError> {
        let mut options = AwsScanOptions {
            profile: None,
            regions: vec![],
            services: vec!["config".into(), "resource-explorer".into()],
            config_aggregator: Some("fixture-aggregator".into()),
            resource_explorer_view: Some("fixture-view".into()),
            discover_sources: false,
            assume_role: None,
            cloud_control: false,
            resource_types: vec![],
            max_pages: 1,
            page_size: 100,
        };
        assert!(options.validate()?.is_empty());
        options.config_aggregator = None;
        assert!(options.validate().is_err());
        Ok(())
    }

    #[test]
    fn rejects_unbounded_options() {
        let options = AwsScanOptions {
            profile: None,
            regions: vec![],
            services: vec![],
            config_aggregator: None,
            resource_explorer_view: None,
            discover_sources: false,
            assume_role: None,
            cloud_control: false,
            resource_types: vec![],
            max_pages: 10_001,
            page_size: 100,
        };
        assert!(options.validate().is_err());
        assert!(validate_region("us-east-1").is_ok());
        assert!(validate_region("us-east-1\n").is_err());
    }

    #[test]
    fn pre_cancelled_scan_returns_before_sdk_setup() -> Result<(), Box<dyn std::error::Error>> {
        let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
        executor.cancel();
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let options = AwsScanOptions {
            profile: None,
            regions: vec!["us-east-1".into()],
            services: vec!["ec2".into()],
            config_aggregator: None,
            resource_explorer_view: None,
            discover_sources: false,
            assume_role: None,
            cloud_control: false,
            resource_types: vec![],
            max_pages: 1,
            page_size: 100,
        };
        let outcome = scan(options, Arc::clone(&executor), sender).now_or_never();
        assert!(matches!(
            outcome,
            Some(Err(AwsScanError::Api(ApiError::Cancelled)))
        ));
        assert_eq!(executor.stats().attempts, 0);
        Ok(())
    }
}
