use super::Error;
use crate::{
    kind_map::KindMap,
    overlay::{
        CheckedOverlays, Emission, Identity, IdentityChoice, IdentityField, IdentityLeaf,
        ItemSelector, OperationOverlay, Scope,
    },
    runtime_catalog::{ServiceIndex, selection},
};
use inventory_core::catalog::{Catalog, CatalogError, Service, Target};
use std::collections::BTreeMap;

pub(super) struct Endpoint<'a, 'b> {
    pub service: Service<'a>,
    pub item: Target,
    pub identity: &'b Identity,
    pub scope: Option<Scope>,
}

pub(super) type Endpoints<'a, 'b> = BTreeMap<&'b str, Vec<Endpoint<'a, 'b>>>;

pub(super) fn build<'a, 'b>(
    catalog: Catalog<'a>,
    kinds: &'b KindMap,
    checked: &'b CheckedOverlays,
) -> Result<Endpoints<'a, 'b>, Error> {
    let services = ServiceIndex::build(catalog)?;
    let mut endpoints = BTreeMap::new();
    kinds.operations().iter().try_for_each(|row| {
        let call = checked
            .services()
            .iter()
            .find(|review| review.service() == row.service)
            .and_then(|review| {
                review
                    .callable()
                    .iter()
                    .find(|call| call.operation() == row.operation)
            });
        let Some(plan) = call.map(crate::overlay::CallableOperation::plan) else {
            return Ok::<_, Error>(());
        };
        if plan.tag_endpoint.is_some() || plan.endpoint_discovery {
            return Ok(());
        }
        let service = services.get(&row.service)?;
        let output = output(service, &row.operation)?;
        if (plan.emission == Emission::EmitMetadata
            || plan
                .branches
                .iter()
                .any(|branch| branch.emission.unwrap_or(plan.emission) == Emission::EmitMetadata))
            && let Some(identity) = &plan.identity
        {
            insert(
                &mut endpoints,
                &row.kind,
                service,
                output,
                plan.items.as_ref(),
                identity,
                plan.scope,
            )?;
        }
        insert_additional(
            &mut endpoints,
            kinds,
            plan,
            service,
            output,
            &row.service,
            &row.operation,
        )
    })?;
    Ok(endpoints)
}

fn insert_additional<'a, 'b>(
    endpoints: &mut Endpoints<'a, 'b>,
    kinds: &'b KindMap,
    plan: &'b OperationOverlay,
    service: Service<'a>,
    output: Target,
    service_name: &str,
    operation: &str,
) -> Result<(), Error> {
    kinds
        .additional_emissions()
        .iter()
        .filter(|extra| extra.service == service_name && extra.operation == operation)
        .try_for_each(|extra| {
            let emission = plan
                .additional_resources
                .iter()
                .find(|emission| emission.key == extra.emitter_key)
                .ok_or(Error::Invalid("additional emission missing"))?;
            insert(
                endpoints,
                &extra.kind,
                service,
                output,
                Some(&emission.items),
                &emission.identity,
                plan.scope,
            )
        })
}

pub(super) fn output(service: Service<'_>, operation: &str) -> Result<Target, Error> {
    let operation = service
        .operations()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            Ok::<_, CatalogError>(if candidate.name()? == operation {
                Some(candidate)
            } else {
                found
            })
        })?
        .ok_or(Error::Invalid("callable operation absent from catalog"))?;
    Ok(operation.output()?)
}

fn insert<'a, 'b>(
    endpoints: &mut Endpoints<'a, 'b>,
    kind: &'b str,
    service: Service<'a>,
    output: Target,
    items: Option<&ItemSelector>,
    identity: &'b Identity,
    scope: Option<Scope>,
) -> Result<(), Error> {
    let item = selection::target(service, output, items)?;
    endpoints.entry(kind).or_default().push(Endpoint {
        service,
        item,
        identity,
        scope,
    });
    Ok(())
}

pub(super) fn identity_path(identity: &Identity, format: &str) -> Option<String> {
    match identity {
        Identity::Arn(path) if format == "arn" => Some(path.as_str().to_owned()),
        Identity::Choice(IdentityChoice::PreferArn { path, fallback, .. }) => {
            if format == "arn" {
                Some(path.as_str().to_owned())
            } else {
                leaf_path(fallback)
            }
        }
        Identity::Key(paths) if format != "arn" && paths.len() == 1 => {
            paths.first().map(|path| path.as_str().to_owned())
        }
        Identity::Template(template) if format != "arn" => template_path(template),
        Identity::CorrelatedKey(fields) if format != "arn" => field_path(fields),
        _ => None,
    }
}

fn leaf_path(identity: &IdentityLeaf) -> Option<String> {
    match identity {
        IdentityLeaf::Arn(_) | IdentityLeaf::OwnedArn { .. } => None,
        IdentityLeaf::Key(paths) if paths.len() == 1 => {
            paths.first().map(|path| path.as_str().to_owned())
        }
        IdentityLeaf::Template(template) => template_path(template),
        IdentityLeaf::Fields(fields) => field_path(fields),
        _ => None,
    }
}

fn template_path(template: &str) -> Option<String> {
    let mut fields = template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(path, _)| path))
        .filter(|path| !matches!(*path, "partition" | "account" | "region"));
    let first = fields.next()?;
    fields.next().is_none().then(|| first.to_owned())
}

fn field_path(fields: &[IdentityField]) -> Option<String> {
    if fields.len() != 1 {
        return None;
    }
    match fields.first()? {
        IdentityField::Selected(path) | IdentityField::SelectedArn(path) => {
            Some(path.as_str().to_owned())
        }
        _ => None,
    }
}
