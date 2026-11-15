use super::{tags, CallablePlan, RuntimeCatalogError};
use crate::{catalog::{runtime::{Emission, KindCoverage, KindNamePath, PlanPurpose, RelationshipPlan}, Service, Target}, inventory::{Kind, KindSchema, ScalarType}, record::FieldId};
use std::sync::Arc;

impl CallablePlan<'_> {
    /// Whether this operation can supply the reviewed relation from its actual modeled namespace.
    pub fn supports_relationship(&self, relation: &RelationshipPlan) -> Result<bool, RuntimeCatalogError> {
        if !emits(self) || !has_kind(self, &relation.source_kind) {
            return Ok(false);
        }
        let target = match relation.source {
            crate::catalog::runtime::RelationSource::SelectedOutput => {
                if matches!(self.projection(), Some(crate::catalog::runtime::ResourceProjectionPlan::IamTrustSummary { path: protected }) if protected.as_ref() == relation.source_path.as_ref()) {
                    return Err(RuntimeCatalogError::Capability);
                }
                tags::selected_item(self.service(), self.operation(), self.items())?
            }
            crate::catalog::runtime::RelationSource::Request => self.operation().input()?,
        };
        match optional_target(self.service(), target, &relation.source_path)? {
            None => Ok(false),
            Some(target) if scalar_matches(tags::target_kind(self.service(), target)?, ScalarType::Text) => Ok(true),
            Some(_) => Err(RuntimeCatalogError::Capability),
        }
    }
}

pub(in crate::catalog::runtime::decode) fn validate(plans: &[CallablePlan<'_>], schemas: &[Arc<KindSchema>], names: &[KindNamePath], relations: &[RelationshipPlan], coverage: &[KindCoverage]) -> Result<(), RuntimeCatalogError> {
    for plan in plans.iter().filter(|plan| emits(plan)) {
        linked(plan.kind(), schemas, coverage)?;
        if let Some(classification) = plan.classification() {
            for (_, kind) in classification.variants.iter() { linked(kind, schemas, coverage)?; }
        }
    }
    for schema in schemas {
        for column in schema.columns() {
            validate_path(plans, schema.kind(), column.path(), column.scalar())?;
        }
    }
    for name in names {
        column_link(schemas, &name.kind, &name.path)?;
        validate_path(plans, &name.kind, &name.path, ScalarType::Text)?;
    }
    for relation in relations {
        let supported = plans.iter().try_fold(false, |found, plan| {
            plan.supports_relationship(relation).map(|supported| found || supported)
        })?;
        if !supported { return Err(RuntimeCatalogError::Capability); }
        for kind in relation.target_kinds.iter() {
            linked(kind, schemas, coverage)?;
            column_link(schemas, kind, &relation.target_path)?;
            validate_path(plans, kind, &relation.target_path, ScalarType::Text)?;
        }
    }
    Ok(())
}

fn linked(kind: &Kind, schemas: &[Arc<KindSchema>], coverage: &[KindCoverage]) -> Result<(), RuntimeCatalogError> {
    if schemas.iter().any(|schema| schema.kind() == kind)
        && coverage.iter().any(|entry| entry.mapped && entry.kind.as_ref() == Some(kind)) { Ok(()) }
    else { Err(RuntimeCatalogError::Capability) }
}

fn column_link(schemas: &[Arc<KindSchema>], kind: &Kind, path: &[FieldId]) -> Result<(), RuntimeCatalogError> {
    if schemas.iter().find(|schema| schema.kind() == kind).is_some_and(|schema|
        schema.columns().iter().any(|column| column.path() == path && column.scalar() == ScalarType::Text)) { Ok(()) }
    else { Err(RuntimeCatalogError::Capability) }
}

fn emits(plan: &CallablePlan<'_>) -> bool {
    !matches!(plan.purpose(), PlanPurpose::TagEndpoint(_) | PlanPurpose::EndpointDiscovery)
        && if plan.branches().is_empty() {
            plan.emission() == Emission::EmitMetadata
        } else {
            plan.branches().iter().any(|branch| plan.effective_emission(Some(branch)) == Emission::EmitMetadata)
        }
}

fn has_kind(plan: &CallablePlan<'_>, kind: &Kind) -> bool {
    plan.kind() == kind || plan.classification().is_some_and(|classification| classification.variants.iter().any(|(_, candidate)| candidate == kind))
}

fn validate_path(plans: &[CallablePlan<'_>], kind: &Kind, path: &[FieldId], scalar: ScalarType) -> Result<(), RuntimeCatalogError> {
    let mut found = false;
    for plan in plans.iter().filter(|plan| emits(plan) && has_kind(plan, kind)) {
        if matches!(plan.projection(), Some(crate::catalog::runtime::ResourceProjectionPlan::IamTrustSummary { path: protected }) if protected.as_ref() == path) {
            return Err(RuntimeCatalogError::Capability);
        }
        let item = tags::selected_item(plan.service(), plan.operation(), plan.items())?;
        if let Some(target) = optional_target(plan.service(), item, path)? {
            found = true;
            if !scalar_matches(tags::target_kind(plan.service(), target)?, scalar) {
                return Err(RuntimeCatalogError::Capability);
            }
        }
    }
    if found { Ok(()) } else { Err(RuntimeCatalogError::Capability) }
}


fn optional_target(service: Service<'_>, mut target: Target, path: &[FieldId]) -> Result<Option<Target>, RuntimeCatalogError> {
    for id in path {
        let shape = tags::shape(service, target)?;
        if shape.kind()? != "structure" { return Err(RuntimeCatalogError::Capability); }
        let mut member = None;
        for candidate in shape.members()? {
            let candidate = candidate?;
            if candidate.id == *id {
                if member.replace(candidate.target).is_some() { return Err(RuntimeCatalogError::Capability); }
            }
        }
        let Some(next) = member else { return Ok(None); };
        target = next;
    }
    Ok(Some(target))
}

fn scalar_matches(kind: &str, scalar: ScalarType) -> bool {
    match scalar {
        ScalarType::Text => matches!(kind, "string" | "enum"),
        ScalarType::Bool => kind == "boolean",
        ScalarType::Integer => matches!(kind, "byte" | "short" | "integer" | "long" | "intEnum"),
        ScalarType::Real => matches!(kind, "float" | "double" | "bigDecimal"),
        ScalarType::Blob => kind == "blob",
        ScalarType::Timestamp => kind == "timestamp",
    }
}
