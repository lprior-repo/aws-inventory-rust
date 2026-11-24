use super::*;
use inventory_core::{catalog::runtime::ItemPlan, record::FieldId};

pub(super) async fn bootstrap_account(
    context: &BootstrapContext<'_, '_, impl CredentialSource>,
) -> Result<AccountId, SweepError> {
    let plan = unique_purpose(context.catalog, |purpose| {
        matches!(purpose, PlanPurpose::AccountIdentity { .. })
    })?;
    let PlanPurpose::AccountIdentity { account_path } = plan.purpose() else {
        return Err(SweepError::Operation(
            "account identity plan purpose changed",
        ));
    };
    let response = execute(context, plan, None).await?;
    account_id(&response.record, account_path)
}

pub(crate) fn account_id(
    response: &inventory_core::record::Record,
    path: &[FieldId],
) -> Result<AccountId, SweepError> {
    let value = record::path_value(response.root(), path)?.ok_or(SweepError::Operation(
        "account identity response omitted account",
    ))?;
    let account = match value {
        inventory_core::record::ValueRef::Text(value) => value,
        _ => return Err(SweepError::Operation("account identity value is not text")),
    };
    AccountId::parse(account).map_err(SweepError::from)
}

pub(super) async fn discover_regions(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
) -> Result<Vec<Region>, SweepError> {
    let requested = requested_region(context.bootstrap.config)?;
    let plan = unique_purpose(context.bootstrap.catalog, |purpose| {
        matches!(purpose, PlanPurpose::RegionDiscovery { .. })
    })?;
    let (regions_path, region_name_path) = region_paths(plan)?;
    let response = execute(&context.bootstrap, plan, Some(&context.account)).await?;
    let rows = record::item_records(&response.record, &ItemPlan::Path(regions_path.into()))?;
    narrow_regions(parse_regions(&rows, region_name_path)?, requested)
}

fn requested_region(config: &SweepConfig) -> Result<Option<RegionId>, SweepError> {
    config
        .region
        .as_deref()
        .map(|value| {
            RegionId::parse(value).map_err(|_| SweepError::InvalidConfig("region is invalid"))
        })
        .transpose()
}

fn region_paths<'a>(
    plan: &'a CallablePlan<'_>,
) -> Result<(&'a [FieldId], &'a [FieldId]), SweepError> {
    match plan.purpose() {
        PlanPurpose::RegionDiscovery {
            regions_path,
            region_name_path,
            ..
        } => Ok((regions_path, region_name_path)),
        _ => Err(SweepError::Operation(
            "region discovery plan purpose changed",
        )),
    }
}

async fn execute(
    context: &BootstrapContext<'_, '_, impl CredentialSource>,
    plan: &CallablePlan<'_>,
    account: Option<&AccountId>,
) -> Result<call::DecodedResponse, SweepError> {
    let region = Region::Regional(context.home.clone());
    let endpoint = plan.endpoint(context.home, &region)?;
    let url = endpoint_url(context.config, &endpoint.url)?;
    let base = input::root_bindings(plan, account, Some(&region))?;
    call::Call {
        client: context.client,
        executor: context.executor,
        credentials: context.credentials,
        plan,
        endpoint: &endpoint,
        url: &url,
        base: &base,
        fields: &[],
        metrics: context.metrics,
        account: account.map_or(crate::bucket::BOOTSTRAP_ACCOUNT, AccountId::as_str),
        scope: &region,
    }
    .run()
    .await
}
