//! Reviewed safe metadata gaps grounded in the adjacent pinned service models.
use super::super::{name, path};
use crate::overlay::{
    Binding, ContextSource, ExecutionBranch, FixedValue, Identity, IdentityField, ItemSelector,
    NameError, OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};
mod children;
mod quicksight;
mod regional;

pub(super) fn apply(mut service: ServiceOverlay) -> Result<ServiceOverlay, NameError> {
    match service.service.as_str() {
        "quicksight" => quicksight::apply(&mut service)?,
        "qbusiness" | "qconnect" | "s3files" | "sagemaker" | "sesv2" | "service-catalog" => {
            children::apply(&mut service)?
        }
        "swf"
        | "rbin"
        | "ssm"
        | "ram"
        | "route53globalresolver"
        | "route53-recovery-control-config" => regional::apply(&mut service)?,
        "securitylake" => {
            let mut plan = arn("dataLakes", "dataLakeArn", None)?;
            plan.bind.insert(path("regions")?, Binding::RegionList);
            put(&mut service, "ListDataLakes", plan)?;
        }
        "s3tables" => namespaces(&mut service)?,
        _ => {}
    }
    Ok(service)
}

fn list(
    items: &str,
    identity: Identity,
    token: Option<(&str, &str)>,
) -> Result<OperationOverlay, NameError> {
    Ok(OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse(items)?),
        identity: Some(identity),
        paginator: Some(match token {
            Some((input, output)) => Paginator::Token(TokenPair {
                input: path(input)?,
                output: path(output)?,
                optional: false,
            }),
            None => Paginator::None,
        }),
        ..OperationOverlay::default()
    })
}

fn arn(
    items: &str,
    field: &str,
    token: Option<(&str, &str)>,
) -> Result<OperationOverlay, NameError> {
    let mut plan = list(items, Identity::Arn(path(field)?), token)?;
    plan.queryable.push(path(field)?);
    Ok(plan)
}

fn qualified(
    items: &str,
    request: &[&str],
    selected: &[&str],
    token: Option<(&str, &str)>,
) -> Result<OperationOverlay, NameError> {
    let fields = request
        .iter()
        .map(|field| path(field).map(IdentityField::Request))
        .chain(
            selected
                .iter()
                .map(|field| path(field).map(IdentityField::Selected)),
        )
        .collect::<Result<_, _>>()?;
    let mut plan = list(items, Identity::CorrelatedKey(fields), token)?;
    plan.queryable = selected
        .iter()
        .map(|field| path(field))
        .collect::<Result<_, _>>()?;
    Ok(plan)
}

fn parent(
    plan: &mut OperationOverlay,
    input: &str,
    operation: &str,
    field: &str,
) -> Result<(), NameError> {
    plan.class = Some(OverlayClass::Expand);
    plan.bind.insert(
        path(input)?,
        Binding::From {
            operation: name(operation)?,
            path: path(field)?,
        },
    );
    Ok(())
}

fn inherited(
    plan: &mut OperationOverlay,
    input: &str,
    operation: &str,
    field: &str,
) -> Result<(), NameError> {
    plan.bind.insert(
        path(input)?,
        Binding::Context {
            operation: name(operation)?,
            source: ContextSource::Request,
            path: path(field)?,
        },
    );
    Ok(())
}

fn put(
    service: &mut ServiceOverlay,
    operation: &str,
    plan: OperationOverlay,
) -> Result<(), NameError> {
    service.operations.insert(name(operation)?, plan);
    Ok(())
}

fn branches(plan: &mut OperationOverlay, field: &str, values: &[&str]) -> Result<(), NameError> {
    plan.branches = values
        .iter()
        .map(|value| {
            Ok(ExecutionBranch {
                discriminator: (*value).to_owned(),
                fixed: [(path(field)?, FixedValue::String((*value).to_owned()))].into(),
                scope: Scope::Region,
                request_region: None,
                bind: Default::default(),
                emission: None,
                row_policy: None,
            })
        })
        .collect::<Result<_, NameError>>()?;
    Ok(())
}

fn namespaces(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let identity = Identity::CorrelatedKey(vec![
        IdentityField::RequestArn(path("tableBucketARN")?),
        IdentityField::SelectedStringList {
            path: path("namespace")?,
            max_items: 64,
            max_item_bytes: 255,
        },
    ]);
    let mut plan = list(
        "namespaces",
        identity,
        Some(("continuationToken", "continuationToken")),
    )?;
    parent(&mut plan, "tableBucketARN", "ListTableBuckets", "arn")?;
    put(service, "ListNamespaces", plan)
}
