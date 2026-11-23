mod target;

use crate::{SweepError, record, resource_projection};
use inventory_core::{
    catalog::runtime::{ReferenceOwnerRule, RelationSource, RelationshipPlan},
    inventory::{
        Observation, ReferenceComponent, ReferenceTarget, RegionId, ResourceId, ResourceKey,
        UnresolvedReference,
    },
    record::{FieldId, Record, ValueRef},
};
use std::borrow::Cow;

pub(super) struct Context<'a> {
    pub row: &'a Record,
    pub request: &'a Record,
    pub observation: &'a Observation,
    pub home: &'a RegionId,
    pub parent: Option<&'a ResourceKey>,
}

impl Context<'_> {
    pub(super) fn append_plan(
        &self,
        catalog: &inventory_core::catalog::runtime::RuntimeCatalog<'_>,
        plan: &inventory_core::catalog::runtime::CallablePlan<'_>,
        output: &mut Vec<UnresolvedReference>,
        limit: usize,
    ) -> Result<(), SweepError> {
        for relation in catalog.relationships().iter()
            .filter(|relation| relation.source_kind == self.observation.key().kind) {
            if !plan.supports_relationship(relation)? { continue; }
            self.append(relation, output, limit)?;
        }
        Ok(())
    }

    pub(super) fn append(
        &self,
        relation: &RelationshipPlan,
        output: &mut Vec<UnresolvedReference>,
        limit: usize,
    ) -> Result<(), SweepError> {
        if !self.guard(relation)? {
            return Ok(());
        }
        if relation.source == RelationSource::Parent {
            return self.append_parent(relation, output, limit);
        }
        let Some(value) = self.value(relation.source, &relation.source_path)? else {
            return Ok(());
        };
        self.append_value(relation, value, output, limit)
    }

    fn append_value(
        &self,
        relation: &RelationshipPlan,
        value: ValueRef<'_>,
        output: &mut Vec<UnresolvedReference>,
        limit: usize,
    ) -> Result<(), SweepError> {
        if let ValueRef::List(values) = value {
            if !relation.components.is_empty() {
                return Err(SweepError::Operation(
                    "compound relationship requires scalar source terms",
                ));
            }
            for value in values.iter()? {
                if let Some(text) = text(value?, relation.source)? {
                    self.push(relation, &text, output, limit)?;
                }
            }
        } else if let Some(text) = text(value, relation.source)? {
            self.push(relation, &text, output, limit)?;
        }
        Ok(())
    }

    fn guard(&self, relation: &RelationshipPlan) -> Result<bool, SweepError> {
        let Some(guard) = &relation.guard else { return Ok(true); };
        match self.value(guard.source, &guard.path)? {
            None | Some(ValueRef::Null) => Ok(false),
            Some(ValueRef::Text(text)) => Ok(text == guard.equals),
            Some(_) => Err(SweepError::Operation("relationship guard is not modeled text")),
        }
    }

    fn append_parent(
        &self,
        relation: &RelationshipPlan,
        output: &mut Vec<UnresolvedReference>,
        limit: usize,
    ) -> Result<(), SweepError> {
        let Some(parent) = self.parent else { return Ok(()); };
        if !relation.target_kinds.contains(&parent.kind) { return Ok(()); }
        self.push(relation, parent.id.as_str(), output, limit)
    }

    fn value(
        &self,
        source: RelationSource,
        path: &[FieldId],
    ) -> Result<Option<ValueRef<'_>>, SweepError> {
        let source = match source {
            RelationSource::SelectedOutput => self.row,
            RelationSource::Request => self.request,
            RelationSource::Parent => {
                return Err(SweepError::Operation(
                    "parent relationship cannot select modeled fields",
                ));
            }
        };
        record::path_value(source.root(), path)
    }

    fn push(
        &self,
        relation: &RelationshipPlan,
        value: &str,
        output: &mut Vec<UnresolvedReference>,
        limit: usize,
    ) -> Result<(), SweepError> {
        let Some(target) = self.target(relation, value)? else { return Ok(()); };
        let reference = UnresolvedReference {
            from: self.observation.key().clone(),
            relation: relation.relation.clone(),
            source: inventory_core::inventory::EdgeSource::ReviewedRule,
            target,
        };
        push_reference(relation, reference, output, limit)
    }

    fn validate_request_arn(
        &self,
        source: RelationSource,
        relation: &RelationshipPlan,
        text: &str,
    ) -> Result<(), SweepError> {
        if source == RelationSource::Request
            && matches!(relation.owner, ReferenceOwnerRule::ExecutionAccount)
            && text.starts_with("arn:")
        {
            record::trusted_arn::validate_arn(
                text,
                Some(&record::IdentityContext {
                    account: &self.observation.key().account,
                    region: self.home,
                }),
            )?;
        }
        Ok(())
    }
}

fn push_reference(
    relation: &RelationshipPlan,
    reference: UnresolvedReference,
    output: &mut Vec<UnresolvedReference>,
    limit: usize,
) -> Result<(), SweepError> {
    if !inventory_engine::inventory::admits_reference(relation, &reference) {
        return Ok(());
    }
    if output.len() >= limit {
        return Err(SweepError::Operation("page exceeds the atomic reference bound"));
    }
    output.try_reserve(1)?;
    output.push(reference);
    Ok(())
}

fn text(value: ValueRef<'_>, source: RelationSource) -> Result<Option<Cow<'_, str>>, SweepError> {
    match value {
        ValueRef::Null | ValueRef::Text("") => Ok(None),
        ValueRef::Text(value) => Ok(Some(Cow::Borrowed(value))),
        ValueRef::Int(value) if source == RelationSource::SelectedOutput => {
            Ok(Some(Cow::Owned(value.to_string())))
        }
        _ => Err(SweepError::Operation(
            "relationship source is not modeled reference text",
        )),
    }
}
