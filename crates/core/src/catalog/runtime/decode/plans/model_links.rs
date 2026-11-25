use super::{CallablePlan, RuntimeCatalogError, tags};
use crate::{
    catalog::{
        Service, Target,
        runtime::{
            Emission, KindCoverage, KindNamePath, PlanPurpose, RelationshipPlan,
            ResourceEmissionPlan, ResourceProjectionPlan,
        },
    },
    inventory::{Kind, KindSchema, ScalarType},
    record::FieldId,
};
use std::sync::Arc;

impl CallablePlan<'_> {
    pub fn supports_relationship(
        &self,
        relation: &RelationshipPlan,
    ) -> Result<bool, RuntimeCatalogError> {
        if !emits_resource(self) {
            return Ok(false);
        }
        self.resource_emissions()
            .iter()
            .try_fold(false, |found, emission| {
                supports_emission(self, emission, relation).map(|supported| found || supported)
            })
    }
}

fn supports_emission(
    plan: &CallablePlan<'_>,
    emission: &ResourceEmissionPlan,
    relation: &RelationshipPlan,
) -> Result<bool, RuntimeCatalogError> {
    if !emission.emits_kind(&relation.source_kind) {
        return Ok(false);
    }
    if relation.source == crate::catalog::runtime::RelationSource::Parent {
        return Ok(true);
    }
    if !supports_source(
        plan,
        emission,
        relation.source,
        &relation.source_path,
        relation.components.is_empty(),
    )? {
        return Ok(false);
    }
    if let Some(guard) = &relation.guard
        && !supports_source(plan, emission, guard.source, &guard.path, false)?
    {
        return Ok(false);
    }
    relation
        .components
        .iter()
        .try_fold(true, |supported, component| {
            supports_source(
                plan,
                emission,
                component.source,
                &component.source_path,
                false,
            )
            .map(|present| supported && present)
        })
}

fn supports_source(
    plan: &CallablePlan<'_>,
    emission: &ResourceEmissionPlan,
    source: crate::catalog::runtime::RelationSource,
    path: &[FieldId],
    allow_list: bool,
) -> Result<bool, RuntimeCatalogError> {
    use crate::catalog::runtime::RelationSource;
    let target = match source {
        RelationSource::SelectedOutput => {
            if matches!(emission.projection(), Some(ResourceProjectionPlan::IamTrustSummary { path: protected }) if protected.as_ref() == path)
            {
                return Err(RuntimeCatalogError::Capability);
            }
            tags::selected_item(plan.service(), plan.operation(), emission.items())?
        }
        RelationSource::Request => plan.operation().input()?,
        RelationSource::Parent => return Ok(true),
    };
    match optional_target(plan.service(), target, path)? {
        None => Ok(false),
        Some(target)
            if scalar_matches(tags::target_kind(plan.service(), target)?, ScalarType::Text)
                || (allow_list && relationship_text(plan.service(), target)?) =>
        {
            Ok(true)
        }
        Some(_) => Err(RuntimeCatalogError::Capability),
    }
}

pub(in crate::catalog::runtime::decode) fn validate(
    plans: &[CallablePlan<'_>],
    schemas: &[Arc<KindSchema>],
    names: &[KindNamePath],
    relations: &[RelationshipPlan],
    coverage: &[KindCoverage],
) -> Result<(), RuntimeCatalogError> {
    for plan in plans.iter().filter(|plan| emits_resource(plan)) {
        for emission in plan.resource_emissions().iter() {
            linked(emission.kind(), schemas, coverage)?;
            if let Some(classification) = emission.classification() {
                for (_, kind) in classification.variants.iter() {
                    linked(kind, schemas, coverage)?;
                }
            }
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
            plan.supports_relationship(relation)
                .map(|supported| found || supported)
        })?;
        if !supported {
            return Err(RuntimeCatalogError::Capability);
        }
        for kind in relation.target_kinds.iter() {
            linked(kind, schemas, coverage)?;
            validate_target(
                plans,
                schemas,
                kind,
                relation.target_mode,
                &relation.target_path,
            )?;
            for component in relation.components.iter() {
                validate_target(
                    plans,
                    schemas,
                    kind,
                    component.target_mode,
                    &component.target_path,
                )?;
            }
        }
    }
    Ok(())
}
fn validate_target(
    plans: &[CallablePlan<'_>],
    schemas: &[Arc<KindSchema>],
    kind: &Kind,
    mode: crate::catalog::runtime::RelationTargetMode,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    use crate::catalog::runtime::RelationTargetMode;
    match mode {
        RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText => {
            column_link(schemas, kind, path)?;
            validate_path(plans, kind, path, ScalarType::Text)
        }
        RelationTargetMode::CanonicalId => Ok(()),
        RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText => {
            validate_request_path(plans, kind, path)
        }
    }
}

fn linked(
    kind: &Kind,
    schemas: &[Arc<KindSchema>],
    coverage: &[KindCoverage],
) -> Result<(), RuntimeCatalogError> {
    if schemas.iter().any(|schema| schema.kind() == kind)
        && coverage
            .iter()
            .any(|entry| entry.mapped && entry.kind.as_ref() == Some(kind))
    {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn column_link(
    schemas: &[Arc<KindSchema>],
    kind: &Kind,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    if schemas
        .iter()
        .find(|schema| schema.kind() == kind)
        .is_some_and(|schema| {
            schema
                .columns()
                .iter()
                .any(|column| column.path() == path && column.scalar() == ScalarType::Text)
        })
    {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn emits_resource(plan: &CallablePlan<'_>) -> bool {
    !matches!(
        plan.purpose(),
        PlanPurpose::TagEndpoint(_) | PlanPurpose::EndpointDiscovery
    ) && if plan.branches().is_empty() {
        plan.emission() == Emission::EmitMetadata
    } else {
        plan.branches()
            .iter()
            .any(|branch| plan.effective_emission(Some(branch)) == Emission::EmitMetadata)
    }
}

fn validates_model_paths(plan: &CallablePlan<'_>) -> bool {
    !matches!(
        plan.purpose(),
        PlanPurpose::TagEndpoint(_) | PlanPurpose::EndpointDiscovery
    ) && if plan.branches().is_empty() {
        plan.emission() != Emission::DiscoveryOnly
    } else {
        plan.branches()
            .iter()
            .any(|branch| plan.effective_emission(Some(branch)) != Emission::DiscoveryOnly)
    }
}

fn emitters<'p, 'a>(
    plans: &'p [CallablePlan<'a>],
    kind: &'p Kind,
) -> impl Iterator<Item = (&'p CallablePlan<'a>, &'p ResourceEmissionPlan)> {
    plans
        .iter()
        .filter(|plan| validates_model_paths(plan))
        .flat_map(move |plan| {
            plan.resource_emissions()
                .iter()
                .filter(move |emission| emission.emits_kind(kind))
                .map(move |emission| (plan, emission))
        })
}

fn validate_path(
    plans: &[CallablePlan<'_>],
    kind: &Kind,
    path: &[FieldId],
    scalar: ScalarType,
) -> Result<(), RuntimeCatalogError> {
    let mut found = false;
    for (plan, emission) in emitters(plans, kind) {
        if matches!(emission.projection(), Some(ResourceProjectionPlan::IamTrustSummary { path: protected }) if protected.as_ref() == path)
        {
            return Err(RuntimeCatalogError::Capability);
        }
        let item = tags::selected_item(plan.service(), plan.operation(), emission.items())?;
        if path.is_empty() && tags::target_kind(plan.service(), item)? == "structure" {
            continue;
        }
        if let Some(target) = optional_target(plan.service(), item, path)? {
            found = true;
            if !scalar_matches(tags::target_kind(plan.service(), target)?, scalar) {
                return Err(RuntimeCatalogError::Capability);
            }
        }
    }
    if found {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn validate_request_path(
    plans: &[CallablePlan<'_>],
    kind: &Kind,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    let found =
        emitters(plans, kind).try_fold(false, |found, (plan, _)| {
            match optional_target(plan.service(), plan.operation().input()?, path)? {
                None => Ok(found),
                Some(target)
                    if scalar_matches(
                        tags::target_kind(plan.service(), target)?,
                        ScalarType::Text,
                    ) =>
                {
                    Ok(true)
                }
                Some(_) => Err(RuntimeCatalogError::Capability),
            }
        })?;
    if found {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn relationship_text(service: Service<'_>, target: Target) -> Result<bool, RuntimeCatalogError> {
    if scalar_matches(tags::target_kind(service, target)?, ScalarType::Text) {
        return Ok(true);
    }
    let shape = tags::shape(service, target)?;
    if shape.kind()? != "list" {
        return Ok(false);
    }
    let element = shape.element()?.ok_or(RuntimeCatalogError::Capability)?;
    Ok(scalar_matches(
        tags::target_kind(service, element)?,
        ScalarType::Text,
    ))
}

fn optional_target(
    service: Service<'_>,
    mut target: Target,
    path: &[FieldId],
) -> Result<Option<Target>, RuntimeCatalogError> {
    for id in path {
        match tags::target_kind(service, target)? {
            "string" | "enum" | "intEnum" | "boolean" | "byte" | "short" | "integer" | "long"
            | "float" | "double" | "bigInteger" | "bigDecimal" | "timestamp" | "blob" => {
                return Ok(None);
            }
            "structure" | "union" => {}
            _ => return Err(RuntimeCatalogError::Capability),
        }
        let shape = tags::shape(service, target)?;
        let mut member = None;
        for candidate in shape.members()? {
            let candidate = candidate?;
            if candidate.id == *id && member.replace(candidate.target).is_some() {
                return Err(RuntimeCatalogError::Capability);
            }
        }
        let Some(next) = member else {
            return Ok(None);
        };
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
