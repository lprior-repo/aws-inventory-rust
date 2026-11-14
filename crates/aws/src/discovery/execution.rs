use super::*;

#[tracing::instrument(skip_all, fields(region = %options.region))]
pub async fn discover(
    options: DiscoveryOptions,
    executor: Arc<ApiExecutor>,
) -> Result<DiscoveryReport, AwsScanError> {
    let work: BoxFuture<'static, Result<DiscoveryReport, AwsScanError>> =
        Box::pin(discover_inner(options, executor));
    work.await
}

#[tracing::instrument(skip_all)]
pub(super) async fn discover_inner(
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
    Box::pin(discover_with_client(&client, &options)).await
}

#[tracing::instrument(skip_all)]
pub(super) async fn discover_with_client(
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
pub(super) async fn load_configuration(
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
pub(super) async fn caller_identity(
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
