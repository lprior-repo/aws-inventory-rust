//! scan operations.
use super::*;

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
pub(super) async fn scan_inner(
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

pub(super) async fn explicit_sources(
    base: &Context,
    options: &AwsScanOptions,
) -> Result<(), AwsScanError> {
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

pub(super) fn region_context(base: &Context, region: String) -> Context {
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
pub(super) async fn discover_regions(ctx: &Context) -> Result<Vec<String>, AwsScanError> {
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
pub(super) async fn collect_region(
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

pub(super) fn service_future<'a>(
    ctx: &'a Context,
    service: &str,
) -> BoxFuture<'a, Result<(), AwsScanError>> {
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
