use std::borrow::Cow;
use inventory_core::{
    catalog::runtime::{ReferenceOwnerRule, RelationSource, RelationshipPlan},
    inventory::{Observation, ReferenceComponent, ReferenceTarget, RegionId, ResourceId, ResourceKey, UnresolvedReference},
    record::{FieldId, Record, ValueRef},
};
use crate::sweep_runtime::{record, resource_projection, SweepError};

pub(super) struct Context<'a> {
    pub row: &'a Record,
    pub request: &'a Record,
    pub observation: &'a Observation,
    pub home: &'a RegionId,
    pub parent: Option<&'a ResourceKey>,
}

impl Context<'_> {
    pub(super) fn append(&self, relation: &RelationshipPlan, output: &mut Vec<UnresolvedReference>, limit: usize) -> Result<(), SweepError> {
        if let Some(guard) = &relation.guard {
            match self.value(guard.source, &guard.path)? {
                None | Some(ValueRef::Null) => return Ok(()),
                Some(ValueRef::Text(text)) if text != guard.equals => return Ok(()),
                Some(ValueRef::Text(_)) => {},
                Some(_) => return Err(SweepError::Operation("relationship guard is not modeled text")),
            }
        }
        if relation.source == RelationSource::Parent {
            let Some(parent) = self.parent else { return Ok(()); };
            if !relation.target_kinds.contains(&parent.kind) { return Ok(()); }
            return self.push(relation, parent.id.as_str(), output, limit);
        }
        let Some(value) = self.value(relation.source, &relation.source_path)? else { return Ok(()); };
        if let ValueRef::List(values) = value {
            if !relation.components.is_empty() {
                return Err(SweepError::Operation("compound relationship requires scalar source terms"));
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

    fn value(&self, source: RelationSource, path: &[FieldId]) -> Result<Option<ValueRef<'_>>, SweepError> {
        let source = match source {
            RelationSource::SelectedOutput => self.row,
            RelationSource::Request => self.request,
            RelationSource::Parent => return Err(SweepError::Operation("parent relationship cannot select modeled fields")),
        };
        record::path_value(source.root(), path)
    }

    fn push(&self, relation: &RelationshipPlan, value: &str, output: &mut Vec<UnresolvedReference>, limit: usize) -> Result<(), SweepError> {
        let mut components = Vec::new();
        components.try_reserve(relation.components.len())?;
        for component in relation.components.iter() {
            let Some(value) = self.value(component.source, &component.source_path)? else { return Ok(()); };
            let Some(text) = text(value, component.source)? else { return Ok(()); };
            self.validate_request_arn(component.source, relation, &text)?;
            components.push(ReferenceComponent { mode: component.target_mode, path: component.target_path.clone(),
                value: ResourceId::parse(&text)? });
        }
        self.validate_request_arn(relation.source, relation, value)?;
        let (account, region, kind) = if relation.source == RelationSource::Parent {
            let parent = self.parent.ok_or(SweepError::Operation("parent relationship has no authoritative parent"))?;
            (parent.account.clone(), parent.region.clone(), parent.kind.clone())
        } else {
            let (account, region) = resource_projection::reference_owner(relation, self.observation, value, self.home)?;
            record::trusted_arn::validate_kind_arn(&relation.target_kind, value)?;
            (account, region, relation.target_kind.clone())
        };
        let target = ReferenceTarget::typed(account, region, kind, relation.target_path.to_vec(),
            ResourceId::parse(value)?, relation.target_mode)?.with_components(components)?;
        let target = if relation.target_kinds.len() > 1 { target.with_candidate_kinds(relation.target_kinds.to_vec())? } else { target };
        let reference = UnresolvedReference { from: self.observation.key().clone(), relation: relation.relation.clone(),
            source: inventory_core::inventory::EdgeSource::ReviewedRule, target };
        if !inventory_engine::inventory::admits_reference(relation, &reference) { return Ok(()); }
        if output.len() >= limit { return Err(SweepError::Operation("page exceeds the atomic reference bound")); }
        output.try_reserve(1)?;
        output.push(reference);
        Ok(())
    }

    fn validate_request_arn(&self, source: RelationSource, relation: &RelationshipPlan, text: &str) -> Result<(), SweepError> {
        if source == RelationSource::Request && matches!(relation.owner, ReferenceOwnerRule::ExecutionAccount) && text.starts_with("arn:") {
            record::trusted_arn::validate_arn(text, Some(&record::IdentityContext {
                account: &self.observation.key().account, region: self.home,
            }))?;
        }
        Ok(())
    }
}

fn text(value: ValueRef<'_>, source: RelationSource) -> Result<Option<Cow<'_, str>>, SweepError> {
    match value {
        ValueRef::Null | ValueRef::Text("") => Ok(None),
        ValueRef::Text(value) => Ok(Some(Cow::Borrowed(value))),
        ValueRef::Int(value) if source == RelationSource::SelectedOutput => Ok(Some(Cow::Owned(value.to_string()))),
        _ => Err(SweepError::Operation("relationship source is not modeled reference text")),
    }
}
