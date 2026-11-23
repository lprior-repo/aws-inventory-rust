use super::{Derived, bind_by_name, derive, identity, items, pagination, safety};
use crate::{
    classes::{self, Class, DenyList},
    overlay::{OperationOverlay, OverlayClass, Scope, check::validation},
    plans::{
        self, Expansion, Model, Vocabulary,
        model::{name, target},
    },
};
use std::collections::BTreeMap;

struct Context<'a, 'm, 'd> {
    model: &'a Model<'m>,
    vocabulary: &'a Vocabulary,
    deny: &'a DenyList<'d>,
    parents: &'a BTreeMap<String, OperationOverlay>,
}

pub(super) fn derive(
    model: &Model<'_>,
    operation: &str,
    scope: Scope,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    parents: &BTreeMap<String, OperationOverlay>,
) -> Derived {
    Context {
        model,
        vocabulary,
        deny,
        parents,
    }
    .candidate(operation, scope)
    .map_err(derive::bound_reason)
}

impl Context<'_, '_, '_> {
    fn candidate(&self, operation: &str, scope: Scope) -> Derived {
        let id = self
            .model
            .operation_id(operation)
            .ok_or("expand: missing operation")?;
        let facts = self.model.facts(id).map_err(|error| error.to_string())?;
        let class = classes::classify(&facts, self.deny);
        if !eligible(class, &facts) {
            return Err("not an expansion candidate".to_owned());
        }
        derive::owner_filter(self.model, id)?;
        safety::output(self.model, operation)?;
        let mut entry = self.entry(operation, id, scope)?;
        if !self.modeled_parent(id, operation)? {
            entry.bind =
                bind_by_name::derive(self.model, operation, self.parents, self.vocabulary)?;
        }
        self.checked(operation, &facts, class, entry)
    }

    fn checked(
        &self,
        operation: &str,
        facts: &classes::OperationFacts<'_>,
        class: Class,
        entry: OperationOverlay,
    ) -> Derived {
        derive::accept(validation::operation(validation::StateArgs {
            service: facts.service,
            name: operation,
            item: Some(entry.clone()),
            class,
            model: self.model,
            deny: self.deny,
            vocabulary: self.vocabulary,
            overlays: self.parents,
        }))?;
        Ok(entry)
    }

    fn entry(&self, operation: &str, id: &str, scope: Scope) -> Derived {
        let pagination = plans::pagination(self.model, id, self.vocabulary)
            .map_err(|error| error.to_string())?;
        let items = items::expand(self.model, operation, &pagination)?;
        Ok(OperationOverlay {
            class: Some(OverlayClass::Expand),
            scope: Some(scope),
            identity: Some(identity::derive(
                self.model,
                operation,
                items.as_ref(),
                &pagination,
            )?),
            items,
            paginator: pagination::derive(self.model, operation, &pagination, self.vocabulary)?,
            ..OperationOverlay::default()
        })
    }

    fn modeled_parent(&self, id: &str, operation: &str) -> Result<bool, String> {
        let expansion =
            plans::expansion(self.model, id, self.deny).map_err(|error| error.to_string())?;
        let Expansion::Modeled {
            resource,
            list_source: true,
            ambiguous_resource: false,
            ..
        } = expansion
        else {
            return Ok(false);
        };
        let parent = self.parent(resource)?;
        if parent == operation {
            return Ok(false);
        }
        if !self.parents.contains_key(parent) {
            return Err("expand: parent enumeration is deferred".to_owned());
        }
        Ok(true)
    }

    fn parent(&self, resource: &str) -> Result<&str, String> {
        self.model
            .shape(resource)
            .and_then(|shape| shape.value.get("list"))
            .and_then(target)
            .map(name)
            .ok_or_else(|| "expand: missing modeled parent".to_owned())
    }
}

pub(super) fn eligible(class: Class, facts: &classes::OperationFacts<'_>) -> bool {
    class == Class::Expand
        || (class == Class::Unclassified && facts.readonly && facts.has_required_input)
}
