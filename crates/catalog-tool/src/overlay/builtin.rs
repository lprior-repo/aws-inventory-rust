mod cloudfront;
mod cloudwatch;
mod dynamodb;
mod ec2;
mod ecs;
mod eks;
mod elastic_load_balancing_v2;
mod iam;
mod kms;
mod lambda;
mod organizations;
mod rds;
mod route_53;
mod s3;
mod sfn;
mod sns;
mod sqs;
mod sts;
mod family_a_c;
mod family_d_f;
mod family_g_k;
mod family_l_p;
mod family_q_s;
mod family_t_z;

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    let mut overlays = vec![
        cloudfront::overlay()?,
        cloudwatch::overlay()?,
        dynamodb::overlay()?,
        ec2::overlay()?,
        ecs::overlay()?,
        eks::overlay()?,
        elastic_load_balancing_v2::overlay()?,
        iam::overlay()?,
        kms::overlay()?,
        lambda::overlay()?,
        organizations::overlay()?,
        rds::overlay()?,
        route_53::overlay()?,
        s3::overlay()?,
        sfn::overlay()?,
        sns::overlay()?,
        sqs::overlay()?,
        sts::overlay()?,
    ];
    overlays.extend(family_a_c::overlays()?);
    overlays.extend(family_d_f::overlays()?);
    overlays.extend(family_g_k::overlays()?);
    overlays.extend(family_l_p::overlays()?);
    overlays.extend(family_q_s::overlays()?);
    overlays.extend(family_t_z::overlays()?);
    overlays.sort_unstable_by(|left, right| left.service().cmp(right.service()));
    Ok(overlays)
}

use super::{MemberPath, NameError, OperationName, OperationOverlay, ServiceOverlay};
use std::collections::BTreeMap;

pub(super) fn empty() -> OperationOverlay {
    OperationOverlay::default()
}

pub(super) fn service(
    service: &str,
    operations: Vec<(&str, OperationOverlay)>,
) -> Result<ServiceOverlay, NameError> {
    if !service.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        || !service.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        || operations.len() > super::MAX_OPERATIONS
    {
        return Err(NameError { value: service.to_owned() });
    }
    let operations = operations.into_iter().try_fold(BTreeMap::new(), |mut result, (key, value)| {
        let key = name(key)?;
        if result.insert(key.clone(), value).is_some() {
            return Err(NameError { value: format!("{service}/{key}: duplicate operation") });
        }
        Ok(result)
    })?;
    super::policy::validate(&operations)?;
    Ok(ServiceOverlay {
        service: service.to_owned(),
        format: 1,
        reviewed: "2026-11-15".to_owned(),
        reviewer: "Automated Sol model review; not emulator verified".to_owned(),
        operations,
    })
}

pub(super) fn path(value: &str) -> Result<MemberPath, NameError> {
    MemberPath::selected(value)
}

pub(super) fn name(value: &str) -> Result<OperationName, NameError> {
    OperationName::new(value)
}
