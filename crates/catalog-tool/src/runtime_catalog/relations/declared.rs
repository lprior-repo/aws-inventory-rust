mod encode;
mod model;
mod reachability;
mod resources;

use super::*;
use crate::overlay::{Identity, Scope};
use crate::runtime_catalog::{
    emission_sources::{self, ResourceSource},
    model_path::ValidationContext,
    selection,
};
use inventory_core::catalog::{Service, Shape, Target};
use std::collections::BTreeSet;

include!(concat!(env!("OUT_DIR"), "/thin_services.rs"));

type Reviewed<'a> = BTreeMap<(&'a str, &'a str, &'a str), bool>;
type Seen = BTreeSet<(String, String, Vec<(String, String)>)>;

struct ServiceContext<'a, 's, 'p> {
    services: &'a ServiceIndex<'s>,
    sources: &'a [ResourceSource<'p>],
    resources: resources::Resources<'s, 'p>,
    references: reachability::Reachability,
}

struct Output<'a, 'p> {
    result: &'a mut RelationsResult,
    seen: Seen,
    reviewed: Reviewed<'p>,
}

pub(super) fn append(
    context: &ValidationContext<'_, '_>,
    result: &mut RelationsResult,
) -> Result<(), RuntimeCatalogCompileError> {
    let sources = emission_sources::sources(context.kinds, context.checked).collect::<Vec<_>>();
    let names = sources
        .iter()
        .map(|source| source.service)
        .collect::<BTreeSet<_>>();
    let mut output = Output {
        result,
        seen: BTreeSet::new(),
        reviewed: reviewed(context),
    };
    names.into_iter().try_for_each(|name| {
        let service = context.services.get(name)?;
        let resources = resources::Resources::new(service, name, context, &sources)?;
        let references = reachability::Reachability::new(service, &resources)?;
        let compiler = ServiceContext {
            services: context.services,
            sources: &sources,
            resources,
            references,
        };
        sources
            .iter()
            .filter(|source| source.service == name)
            .try_for_each(|source| {
                let item = selection::target(
                    service,
                    operation(service, source.operation)?.output()?,
                    source.items,
                )?;
                model::declarations(service, item, &compiler.references)?
                    .iter()
                    .try_for_each(|declaration| {
                        compile(&compiler, source, declaration, &mut output)
                    })
            })
    })
}

fn compile(
    context: &ServiceContext<'_, '_, '_>,
    source: &ResourceSource<'_>,
    declaration: &model::Declaration,
    output: &mut Output<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    if let Some(reason) = invalid(declaration) {
        pending(
            output.result,
            source.kind,
            declaration,
            &declaration.resource,
            reason,
        );
        return Ok(());
    }
    let resource = context.resources.get(&declaration.resource);
    if resource.is_none_or(|resource| resource.callable.is_empty() && resource.pending.is_empty()) {
        pending(
            output.result,
            source.kind,
            declaration,
            &declaration.resource,
            "target kind not callable",
        );
    }
    let Some(resource) = resource else {
        return Ok(());
    };
    for kind in &resource.pending {
        if !output
            .reviewed
            .contains_key(&(source.kind, declaration.path.as_str(), *kind))
        {
            pending(
                output.result,
                source.kind,
                declaration,
                kind,
                "target kind not callable",
            );
        }
    }
    resource.callable.iter().try_for_each(|index| {
        let target = context
            .sources
            .get(*index)
            .ok_or(RuntimeCatalogCompileError::Capability)?;
        output.emit(
            context.services,
            source,
            target,
            declaration,
            &resource.identifiers,
        )
    })
}

fn invalid(declaration: &model::Declaration) -> Option<&'static str> {
    declaration.invalid.or_else(|| {
        std::iter::once(&declaration.path)
            .chain(declaration.members.iter().flat_map(BTreeMap::values))
            .any(|path| path.split('.').count() > 8)
            .then_some("reference path exceeds eight members")
    })
}

impl Output<'_, '_> {
    fn emit(
        &mut self,
        services: &ServiceIndex<'_>,
        source: &ResourceSource<'_>,
        target: &ResourceSource<'_>,
        declaration: &model::Declaration,
        identifiers: &BTreeSet<&str>,
    ) -> Result<(), RuntimeCatalogCompileError> {
        let bindings = identity_bindings(declaration, target.identity, identifiers);
        let primary = bindings
            .as_ref()
            .and_then(|bindings| bindings.first())
            .map_or(declaration.path.as_str(), |(primary, _)| *primary);
        if self.reviewed.get(&(source.kind, primary, target.kind)) == Some(&true) {
            return Ok(());
        }
        let Some(bindings) = bindings else {
            pending(
                self.result,
                source.kind,
                declaration,
                target.kind,
                if matches!(declaration.origin, model::Origin::Arn)
                    && matches!(target.identity, Some(Identity::Key(_)))
                {
                    "ARN reference requires an ARN identity"
                } else {
                    "target identity members cannot be bound"
                },
            );
            return Ok(());
        };
        let key = (
            source.kind.to_owned(),
            target.kind.to_owned(),
            bindings
                .iter()
                .map(|(source, target)| ((*source).to_owned(), (*target).to_owned()))
                .collect(),
        );
        if !self.seen.insert(key) {
            return Ok(());
        }
        self.result
            .rows
            .push(encode::relation(services, source.kind, target, &bindings)?);
        self.result.declared = self.result.declared.saturating_add(1);
        if THIN_SERVICES.contains(&source.service) {
            self.result.declared_thin = self.result.declared_thin.saturating_add(1);
        }
        Ok(())
    }
}

fn identity_bindings<'a>(
    declaration: &'a model::Declaration,
    identity: Option<&'a Identity>,
    identifiers: &BTreeSet<&str>,
) -> Option<Vec<(&'a str, &'a str)>> {
    let members = declaration.members.as_ref();
    if members.is_none() && !matches!(identity, Some(Identity::Arn(_))) {
        return None;
    }
    if matches!(members, Some(members) if members.is_empty()) {
        return None;
    }
    let paths = match identity? {
        Identity::Arn(path) => std::slice::from_ref(path),
        Identity::Key(paths) if !paths.is_empty() && paths.len() <= 8 => paths.as_slice(),
        _ => return None,
    };
    let names = paths
        .iter()
        .map(|path| path.as_str().rsplit('.').next())
        .collect::<Option<BTreeSet<_>>>()?;
    if names.len() != paths.len() {
        return None;
    }
    paths
        .iter()
        .map(|path| {
            let source = match members {
                None if paths.len() == 1 => declaration.path.as_str(),
                Some(members) => {
                    let name = path.as_str().rsplit('.').next()?;
                    let identifier = if identifiers.contains(name) {
                        name
                    } else if paths.len() == 1 && identifiers.len() == 1 {
                        identifiers.first().copied()?
                    } else {
                        return None;
                    };
                    members.get(identifier)?.as_str()
                }
                None => return None,
            };
            Some((source, path.as_str()))
        })
        .collect()
}

fn reviewed<'a>(context: &'a ValidationContext<'_, '_>) -> Reviewed<'a> {
    let active = |kind: &str| crate::runtime_catalog::model_path::active_kind(context, kind);
    let scalar = context
        .kinds
        .relations()
        .iter()
        .filter(|row| active(&row.source_kind))
        .map(|row| (&row.source_kind, Some(&row.source), &row.target_kinds));
    let compound = context
        .kinds
        .compound_relations()
        .iter()
        .filter(|row| active(&row.source_kind))
        .map(|row| {
            (
                &row.source_kind,
                row.components.first().map(|component| &component.source),
                &row.target_kinds,
            )
        });
    scalar
        .chain(compound)
        .flat_map(|(kind, source, targets)| {
            let callable = targets.iter().all(active);
            let path = match source {
                Some(ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(path))) => {
                    Some(path.as_str())
                }
                Some(ReviewedRelationSource::SelectedOutput(ReviewedValuePath::RootText)) => {
                    Some("$")
                }
                _ => None,
            };
            targets.iter().filter_map(move |target| {
                path.map(|path| ((kind.as_str(), path, target), callable))
            })
        })
        .fold(BTreeMap::new(), |mut reviewed, (key, callable)| {
            reviewed
                .entry(key)
                .and_modify(|live| *live |= callable)
                .or_insert(callable);
            reviewed
        })
}

fn pending(
    result: &mut RelationsResult,
    kind: &str,
    declaration: &model::Declaration,
    target: &str,
    reason: &str,
) {
    result.inactive.insert(
        format!(
            "pending {kind} {} {target} - REFERENCES ({reason})",
            declaration.path
        ),
        reason.to_owned(),
    );
}

fn operation<'a>(
    service: Service<'a>,
    name: &str,
) -> Result<inventory_core::catalog::Operation<'a>, RuntimeCatalogCompileError> {
    service
        .operations()?
        .try_fold(None, |found, operation| {
            let operation = operation?;
            Ok::<_, RuntimeCatalogCompileError>(if operation.name()? == name {
                Some(operation)
            } else {
                found
            })
        })?
        .ok_or(RuntimeCatalogCompileError::Capability)
}
