use super::*;

pub(super) fn profiles(options: &ScanArgs) -> Result<Vec<Option<String>>> {
    anyhow::ensure!(
        options.profile.len() <= MAX_PROFILES,
        "at most 128 profiles per invocation"
    );
    let profiles = if options.profile.is_empty() {
        vec![None]
    } else {
        options
            .profile
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(Some)
            .collect()
    };
    Ok(profiles)
}

pub(super) fn job(
    options: &ScanArgs,
    profile: Option<String>,
    assume_role: Option<AssumeRoleOptions>,
    selection: &Selection,
) -> Job {
    Job {
        scan_id: uuid::Uuid::new_v4().to_string(),
        options: AwsScanOptions {
            profile,
            regions: options.region.clone(),
            services: options.service.clone(),
            config_aggregator: options.config_aggregator.clone(),
            resource_explorer_view: options.resource_explorer_view.clone(),
            discover_sources: options.discover_sources,
            assume_role,
            cloud_control: options.cloud_control,
            selection: selection.clone(),
            max_pages: options.max_pages,
            page_size: options.page_size,
        },
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn prepare_jobs(
    options: &ScanArgs,
    executor: Arc<ApiExecutor>,
) -> Result<(Vec<Job>, Option<Value>)> {
    let profiles = profiles(options)?;
    let selection = options.selection()?;
    let external_id = options
        .accounts
        .external_id_env
        .as_ref()
        .map(|name| {
            std::env::var(name).with_context(|| {
                format!("external ID environment variable {name} is missing or not UTF-8")
            })
        })
        .transpose()?;
    if let Some(role_name) = &options.accounts.organization_role {
        let report = discovery::discover(
            DiscoveryOptions {
                profile: options.profile.first().cloned(),
                region: options
                    .region
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "us-east-1".into()),
                organization: true,
                hierarchy: false,
                config_aggregators: false,
                resource_explorer: false,
                max_pages: options.max_pages,
                page_size: options.page_size,
            },
            executor,
        )
        .await?;
        let organization = report
            .organization
            .context("organization discovery returned no account report")?;
        let accounts = select_accounts(&organization, &options.accounts)?;
        let jobs = accounts
            .iter()
            .map(|account| {
                let arn = discovery::account_role_arn(&report.partition, account, role_name)?;
                let role = role_options(&options.accounts, arn, external_id.clone())?;
                Ok(job(
                    options,
                    options.profile.first().cloned(),
                    Some(role),
                    &selection,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let summary = json!({"source_account":report.account_id,"partition":report.partition,
            "accounts_discovered":organization.accounts.len(),"eligible_accounts":organization.eligible_accounts().count(),
            "selected_accounts":accounts,"account_listing_complete":organization.account_coverage.complete,
            "role_name":role_name,"semantics":"ACTIVE organization accounts only; one immutable scan per selected role target"});
        return Ok((jobs, Some(summary)));
    }
    let role = options
        .accounts
        .role_arn
        .as_ref()
        .map(|arn| role_options(&options.accounts, arn.clone(), external_id))
        .transpose()?;
    Ok((
        profiles
            .into_iter()
            .map(|profile| job(options, profile, role.clone(), &selection))
            .collect(),
        None,
    ))
}

pub(super) fn role_options(
    options: &AccountArgs,
    role_arn: String,
    external_id: Option<String>,
) -> Result<AssumeRoleOptions> {
    let role = AssumeRoleOptions {
        role_arn,
        session_name: options.role_session_name.clone(),
        external_id,
        duration_seconds: options.role_duration_seconds,
    };
    role.validate()?;
    Ok(role)
}

pub(super) fn select_accounts(
    organization: &OrganizationDiscovery,
    options: &AccountArgs,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        organization.account_coverage.complete,
        "organization account listing is incomplete; inspect `discover --organization` before selecting targets"
    );
    let eligible: BTreeSet<&str> = organization
        .eligible_accounts()
        .map(|account| account.id.as_str())
        .collect();
    let requested: BTreeSet<&str> = options.account_id.iter().map(String::as_str).collect();
    for account in &requested {
        anyhow::ensure!(
            eligible.contains(account),
            "requested account {account} is absent or not ACTIVE"
        );
    }
    let selected: Vec<String> = eligible
        .into_iter()
        .filter(|id| requested.is_empty() || requested.contains(id))
        .map(str::to_owned)
        .collect();
    anyhow::ensure!(
        !selected.is_empty(),
        "organization target selection is empty"
    );
    anyhow::ensure!(
        selected.len() <= options.max_accounts,
        "{} selected accounts exceeds --max-accounts {}; narrow --account-id or raise the bound explicitly",
        selected.len(),
        options.max_accounts
    );
    Ok(selected)
}
