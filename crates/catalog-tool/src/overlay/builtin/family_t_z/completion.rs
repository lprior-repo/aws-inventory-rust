//! Reviewed summary-only collections from the pinned T–Z service models.
use super::{bind, inventory, name, path, paths, token};
use crate::overlay::{
    Binding, ContextSource, Identity, IdentityField, NameError, OperationOverlay, Paginator, Scope,
    ServiceOverlay,
};

mod lattice;
mod mail;
mod metadata;
mod waf;
mod workspaces;

pub(super) fn apply(services: Vec<ServiceOverlay>) -> Result<Vec<ServiceOverlay>, NameError> {
    services
        .into_iter()
        .map(|mut service| {
            match service.service.as_str() {
                "vpc-lattice" => lattice::apply(&mut service)?,
                "workmail" => mail::apply(&mut service)?,
                "workspaces" => workspaces::apply(&mut service)?,
                "waf" | "waf-regional" => waf::apply(&mut service)?,
                "wellarchitected" | "wisdom" | "wickr" | "voice-id" => {
                    metadata::apply(&mut service)?
                }
                _ => {}
            }
            Ok(service)
        })
        .collect()
}

fn insert(
    service: &mut ServiceOverlay,
    operation: &str,
    plan: OperationOverlay,
) -> Result<(), NameError> {
    service.operations.insert(name(operation)?, plan);
    Ok(())
}

fn correlated(request: &[&str], selected: &[&str]) -> Result<Identity, NameError> {
    let fields = request
        .iter()
        .map(|field| path(field).map(IdentityField::Request))
        .chain(
            selected
                .iter()
                .map(|field| path(field).map(IdentityField::Selected)),
        )
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Identity::CorrelatedKey(fields))
}

fn child(
    items: &str,
    identity: Identity,
    parent: &str,
    input: &str,
    field: &str,
    paginated: bool,
) -> Result<OperationOverlay, NameError> {
    let paginator = if paginated {
        token("NextToken", "NextToken")?
    } else {
        Paginator::None
    };
    let mut plan = inventory(items, identity, paginator, Scope::Region)?;
    bind(&mut plan, input, parent, field)?;
    Ok(plan)
}

fn request_context(
    plan: &mut OperationOverlay,
    input: &str,
    parent: &str,
    field: &str,
) -> Result<(), NameError> {
    plan.bind.insert(
        path(input)?,
        Binding::Context {
            operation: name(parent)?,
            source: ContextSource::Request,
            path: path(field)?,
        },
    );
    Ok(())
}
