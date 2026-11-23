use super::{RuntimeCatalogCompileError, encode};
use crate::overlay::{OperationOverlay, RequestRouting};
use inventory_core::{
    catalog::{Operation, Service},
    record::Value,
};

pub(super) fn compile(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
    checked: &crate::overlay::CheckedOverlays,
    services: &super::super::ServiceIndex<'_>,
) -> Result<[(u32, Value); 2], RuntimeCatalogCompileError> {
    let route = compile_routing(overlay, services)?;
    validate_branches(overlay)?;
    let branches = compile_branches(service, operation, overlay, checked, services)?;
    Ok([(15, route), (16, Value::List(branches))])
}

fn compile_routing(
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match &overlay.routing {
        RequestRouting::Regional => encode::map([(1, Value::Int(0))]),
        RequestRouting::FixedRegion { region } => {
            inventory_core::inventory::RegionId::parse(region).map_err(|_| {
                RuntimeCatalogCompileError::Path {
                    path: "fixed region".to_owned(),
                }
            })?;
            encode::map([(1, Value::Int(1)), (2, Value::Text(region.clone()))])
        }
        RequestRouting::Discovered {
            operation,
            endpoints_path,
            address_path,
            cache_minutes_path,
            trusted_dns_suffix,
        } => {
            if operation != "DescribeEndpoints"
                || trusted_dns_suffix != ".timestream.{region}.amazonaws.com"
            {
                return Err(RuntimeCatalogCompileError::Path {
                    path: "discovered routing".to_owned(),
                });
            }
            Ok(encode::map([
                (1, Value::Int(2)),
                (2, Value::Text(operation.clone())),
                (3, encode::path(services, endpoints_path.as_str())?),
                (4, encode::path(services, address_path.as_str())?),
                (5, encode::path(services, cache_minutes_path.as_str())?),
                (6, Value::Text(trusted_dns_suffix.clone())),
            ])?)
        }
        RequestRouting::S3Outposts => encode::map([(1, Value::Int(3))]),
        RequestRouting::S3ExpressControl => encode::map([(1, Value::Int(4))]),
    }
}

fn validate_branches(overlay: &OperationOverlay) -> Result<(), RuntimeCatalogCompileError> {
    if overlay.branches.len() > 16 {
        return Err(RuntimeCatalogCompileError::Path {
            path: "branch count".to_owned(),
        });
    }
    let names = overlay
        .branches
        .iter()
        .map(|branch| &branch.discriminator)
        .collect::<std::collections::BTreeSet<_>>();
    if names.len() != overlay.branches.len() {
        return Err(RuntimeCatalogCompileError::Path {
            path: "branch duplicate".to_owned(),
        });
    }
    Ok(())
}

fn compile_branches(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
    checked: &crate::overlay::CheckedOverlays,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    overlay
        .branches
        .iter()
        .map(|branch| {
            compile_branch(
                service,
                operation,
                branch,
                overlay.items.as_ref(),
                checked,
                services,
            )
        })
        .collect()
}
fn compile_branch(
    service: Service<'_>,
    operation: Operation<'_>,
    branch: &crate::overlay::ExecutionBranch,
    items: Option<&crate::overlay::ItemSelector>,
    checked: &crate::overlay::CheckedOverlays,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let name = &branch.discriminator;
    validate_branch(name, branch)?;
    let bindings = compile_branch_bindings(service, operation, branch, services)?;
    let region = compile_branch_region(branch)?;
    let emission = compile_branch_emission(branch);
    let ctx = BranchCompileCtx {
        service,
        operation,
        items,
        checked,
        services,
    };
    let row_policy = compile_branch_row_policy(&ctx, branch)?;
    encode::map(
        [
            (1, Value::Text(name.clone())),
            (2, Value::List(bindings)),
            (3, Value::Int(super::scope(Some(branch.scope))?)),
        ]
        .into_iter()
        .chain(region)
        .chain(emission)
        .chain(row_policy),
    )
}

fn validate_branch(
    name: &str,
    branch: &crate::overlay::ExecutionBranch,
) -> Result<(), RuntimeCatalogCompileError> {
    if !valid_branch_name(name)
        || branch.bind.len() > 4096
        || branch
            .bind
            .keys()
            .any(|path| branch.fixed.contains_key(path))
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: name.to_owned(),
        });
    }
    Ok(())
}

fn compile_branch_bindings(
    service: Service<'_>,
    operation: Operation<'_>,
    branch: &crate::overlay::ExecutionBranch,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    branch
        .fixed
        .iter()
        .map(|(path, value)| {
            let target =
                super::super::model_path::resolve(service, operation.input()?, path.as_str())?.1;
            let value = super::fixed_input::compile(service, target, value, path.as_str())?;
            super::binding_record(path, 2, (value, Value::Null, Value::Null), services)
        })
        .chain(
            branch.bind.iter().map(|(path, binding)| {
                super::binding_value(service.name()?, path, binding, services)
            }),
        )
        .collect()
}

fn compile_branch_region(
    branch: &crate::overlay::ExecutionBranch,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    branch
        .request_region
        .as_ref()
        .map(|region| -> Result<_, RuntimeCatalogCompileError> {
            inventory_core::inventory::RegionId::parse(region).map_err(|_| {
                RuntimeCatalogCompileError::Path {
                    path: "branch request_region".to_owned(),
                }
            })?;
            Ok((4, Value::Text(region.to_owned())))
        })
        .transpose()
}

fn compile_branch_emission(branch: &crate::overlay::ExecutionBranch) -> Option<(u32, Value)> {
    branch.emission.map(|emission| {
        (
            5,
            Value::Int(match emission {
                crate::overlay::Emission::EmitMetadata => 0,
                crate::overlay::Emission::DiscoveryOnly => 1,
            }),
        )
    })
}

struct BranchCompileCtx<'a> {
    service: Service<'a>,
    operation: Operation<'a>,
    items: Option<&'a crate::overlay::ItemSelector>,
    checked: &'a crate::overlay::CheckedOverlays,
    services: &'a super::super::ServiceIndex<'a>,
}

fn compile_branch_row_policy(
    ctx: &BranchCompileCtx<'_>,
    branch: &crate::overlay::ExecutionBranch,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    let policy = branch
        .row_policy
        .as_ref()
        .map(|policy| {
            super::row_policy::compile(
                ctx.service,
                ctx.operation,
                policy,
                ctx.items,
                ctx.checked,
                ctx.service.name()?,
                ctx.services,
            )
        })
        .transpose()?
        .flatten();
    Ok(policy.map(|(_, value)| (6, value)))
}

pub(super) fn valid_branch_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
