use crate::SweepError;
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ExecutionBranch, PlanScope, RequestRouting, ResolvedEndpoint,
    },
    inventory::{Region, RegionId},
};

pub(crate) fn execution_branch<'a>(
    plan: &'a CallablePlan<'_>,
    branch: Option<&str>,
) -> Result<Option<&'a ExecutionBranch>, SweepError> {
    if plan.branches().is_empty() {
        if branch.is_some() {
            return Err(SweepError::Operation(
                "unbranched work has an unexpected branch",
            ));
        }
        return Ok(None);
    }
    let branch = branch.ok_or(SweepError::Operation(
        "persisted work omitted required branch",
    ))?;
    plan.branches()
        .iter()
        .find(|variant| variant.discriminator == branch)
        .map(Some)
        .ok_or(SweepError::Operation(
            "persisted work branch is not reviewed",
        ))
}

pub(crate) fn request_region(
    plan: &CallablePlan<'_>,
    scope: &Region,
    home: &RegionId,
    branch: Option<&str>,
) -> Result<RegionId, SweepError> {
    if let Some(variant) = execution_branch(plan, branch)?
        && let Some(region) = &variant.request_region
    {
        return RegionId::parse(region).map_err(SweepError::from);
    }
    if let RequestRouting::FixedRegion { region } = plan.routing() {
        return RegionId::parse(region).map_err(SweepError::from);
    }
    Ok(match scope {
        Region::Global => home.clone(),
        Region::Regional(region) => region.clone(),
    })
}

pub(crate) fn resolve(
    plan: &CallablePlan<'_>,
    region: &RegionId,
) -> Result<ResolvedEndpoint, SweepError> {
    let scope = match plan.scope() {
        PlanScope::Account => Region::Global,
        PlanScope::Region | PlanScope::HomeRegion => Region::Regional(region.clone()),
    };
    let mut endpoint = plan.endpoint(region, &scope)?;
    if matches!(plan.routing(), RequestRouting::S3Outposts) {
        endpoint.url = routed_url("s3-outposts", region, &endpoint.url)?;
        endpoint.signing_name = "s3-outposts".to_owned();
        endpoint.signing_region = region.as_str().to_owned();
    } else if matches!(plan.routing(), RequestRouting::S3ExpressControl) {
        endpoint.url = routed_url("s3express-control", region, &endpoint.url)?;
        endpoint.signing_name = "s3express".to_owned();
        endpoint.signing_region = region.as_str().to_owned();
    }
    Ok(endpoint)
}

fn routed_url(service: &str, region: &RegionId, modeled_url: &str) -> Result<String, SweepError> {
    let host = modeled_url
        .strip_prefix("https://")
        .filter(|host| !host.contains(['/', ':', '?', '#', '@']))
        .ok_or(SweepError::Operation(
            "modeled routing endpoint is not a trusted HTTPS origin",
        ))?;
    let (_, regional_suffix) = host.split_once('.').ok_or(SweepError::Operation(
        "modeled routing endpoint omitted its DNS suffix",
    ))?;
    let suffix = regional_suffix
        .strip_prefix(region.as_str())
        .and_then(|suffix| suffix.strip_prefix('.'))
        .unwrap_or(regional_suffix);
    Ok(format!("https://{service}.{}.{suffix}", region.as_str()))
}

#[cfg(test)]
pub(crate) fn branch_bindings(
    plan: &CallablePlan<'_>,
    branch: &inventory_core::catalog::runtime::ExecutionBranch,
    account: &inventory_core::inventory::AccountId,
    region: &Region,
) -> Result<inventory_core::record::Record, SweepError> {
    crate::input::root_bindings_branch(plan, Some(branch), Some(account), Some(region))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_routes_retain_modeled_partition_dns_suffix() -> Result<(), SweepError> {
        for (region, modeled, expected) in [
            (
                "us-east-1",
                "https://s3.amazonaws.com",
                "https://s3express-control.us-east-1.amazonaws.com",
            ),
            (
                "cn-north-1",
                "https://s3-control.cn-north-1.amazonaws.com.cn",
                "https://s3express-control.cn-north-1.amazonaws.com.cn",
            ),
            (
                "us-iso-east-1",
                "https://s3-control.us-iso-east-1.c2s.ic.gov",
                "https://s3express-control.us-iso-east-1.c2s.ic.gov",
            ),
        ] {
            assert_eq!(
                routed_url("s3express-control", &RegionId::parse(region)?, modeled)?,
                expected
            );
        }
        Ok(())
    }
}
