use super::RuntimeCatalogCompileError;
use crate::{
    kind_map::KindMap,
    overlay::{
        CallableOperation, CheckError, CheckedOverlays, Emission, Identity, ItemSelector,
        KindClassification, MapKeyExclusion, MemberPath, OperationOverlay, ResourceProjection,
        SkipEvidence,
    },
};
use inventory_core::catalog::runtime::{
    CallablePlan, Emission as RuntimeEmission, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan,
    PlanScope, RelationSource, ResourceEmissionPlan, RuntimeCatalog,
};

pub(crate) fn validate(checked: &CheckedOverlays, kinds: &KindMap) -> Result<(), CheckError> {
    checked.services().iter().try_for_each(|service| {
        service.callable().iter().try_for_each(|parent| {
            parent.plan().skip.iter().try_for_each(|target| {
                let name = target.as_str();
                if !enumeration(parent, kinds) {
                    return Err(invalid(
                        parent,
                        name,
                        "only enumeration plans can carry skips",
                    ));
                }
                let child = service.callable().iter().find(|op| op.operation() == name);
                let Some(child) = child else {
                    return Err(invalid(parent, name, "skip child is not callable"));
                };
                if !enumeration(child, kinds) {
                    return Err(invalid(
                        parent,
                        name,
                        "skip child is not an enumeration plan",
                    ));
                }
                if !parent
                    .plan()
                    .skip_evidence
                    .get(target)
                    .is_some_and(|receipt| {
                        receipt.service == parent.service()
                            && receipt.list.as_str() == parent.operation()
                            && receipt.describe.as_str() == name
                    })
                {
                    return Err(invalid(
                        parent,
                        name,
                        "skip receipt is for a different operation pair",
                    ));
                }
                validate_emissions(parent, child, kinds)
            })
        })
    })
}

fn enumeration(operation: &CallableOperation, kinds: &KindMap) -> bool {
    let plan = operation.plan();
    !plan.endpoint_discovery
        && plan.tag_endpoint.is_none()
        && kinds
            .bootstrap_for(operation.service(), operation.operation())
            .is_none()
}

fn invalid(parent: &CallableOperation, value: &str, rule: &'static str) -> CheckError {
    CheckError::Invalid {
        service: parent.service().to_owned(),
        operation: parent.operation().to_owned(),
        key: "skip".to_owned(),
        value: value.to_owned(),
        rule,
    }
}

struct Resource<'a> {
    kind: &'a str,
    items: Option<&'a ItemSelector>,
    identity: Option<&'a Identity>,
    classification: Option<&'a KindClassification>,
    tags: Option<&'a MemberPath>,
    projection: Option<&'a ResourceProjection>,
    exclusions: &'a [MemberPath],
    map_exclusions: &'a [MapKeyExclusion],
    queryable: &'a [MemberPath],
}

fn resources<'a>(
    callable: &'a CallableOperation,
    kinds: &'a KindMap,
) -> impl Iterator<Item = Resource<'a>> {
    let plan = callable.plan();
    let primary = kinds
        .operations()
        .iter()
        .find(|row| row.service == callable.service() && row.operation == callable.operation());
    primary
        .into_iter()
        .map(move |row| Resource {
            kind: &row.kind,
            items: plan.items.as_ref(),
            identity: plan.identity.as_ref(),
            classification: plan.classification.as_ref(),
            tags: plan.tags.as_ref(),
            projection: plan.projection.as_ref(),
            exclusions: &plan.exclude_from_hash,
            map_exclusions: &plan.exclude_map_keys,
            queryable: &plan.queryable,
        })
        .chain(plan.additional_resources.iter().filter_map(move |extra| {
            let kind = kinds.additional_emission_kind(
                callable.service(),
                callable.operation(),
                &extra.key,
            )?;
            Some(Resource {
                kind,
                items: Some(&extra.items),
                identity: Some(&extra.identity),
                classification: extra.classification.as_ref(),
                tags: extra.tags.as_ref(),
                projection: extra.projection.as_ref(),
                exclusions: &extra.exclude_from_hash,
                map_exclusions: &extra.exclude_map_keys,
                queryable: &extra.queryable,
            })
        }))
}

fn validate_emissions(
    parent: &CallableOperation,
    child: &CallableOperation,
    kinds: &KindMap,
) -> Result<(), CheckError> {
    let child_name = child.operation();
    let mut emissions = resources(child, kinds).peekable();
    if emissions.peek().is_none() {
        return Err(invalid(
            parent,
            child_name,
            "skip child has no resource emission",
        ));
    }
    emissions.try_for_each(|emission| {
        let reject = |rule| invalid(parent, &format!("{child_name} -> {}", emission.kind), rule);
        let producer = resources(parent, kinds)
            .find(|candidate| candidate.kind == emission.kind)
            .ok_or_else(|| reject("skip parent does not emit child kind"))?;
        if let Some(reason) = missing_emission(parent.plan(), child.plan(), &producer, &emission) {
            return Err(reject(reason));
        }
        Ok(())
    })?;
    if !metadata_for_all_branches(parent.plan()) {
        return Err(invalid(
            parent,
            child_name,
            "skip parent has a branch without metadata emission",
        ));
    }
    if [parent.plan(), child.plan()]
        .into_iter()
        .any(filters_emissions)
    {
        return Err(invalid(
            parent,
            child_name,
            "skip cannot prove row-policy emission equivalence",
        ));
    }
    if parent.plan().exclude_response_members != child.plan().exclude_response_members {
        return Err(invalid(
            parent,
            child_name,
            "skip response member exclusions differ",
        ));
    }
    Ok(())
}

fn missing_emission(
    parent: &OperationOverlay,
    child: &OperationOverlay,
    producer: &Resource<'_>,
    emission: &Resource<'_>,
) -> Option<&'static str> {
    // The probe compares only the primary selected row, not other response rows.
    if producer.items != parent.items.as_ref() || emission.items != child.items.as_ref() {
        return Some("skip resource selector is outside the compared rows");
    }
    if producer.classification != emission.classification {
        return Some("skip parent does not preserve child classification");
    }
    if producer.identity != emission.identity {
        return Some("skip parent does not preserve child identity");
    }
    if emission.tags.is_some() && producer.tags != emission.tags {
        return Some("skip parent does not preserve child tag source");
    }
    if producer.projection != emission.projection {
        return Some("skip parent does not preserve child projection");
    }
    if producer.exclusions != emission.exclusions
        || producer.map_exclusions != emission.map_exclusions
    {
        return Some("skip parent does not preserve child hash exclusions");
    }
    if !emission
        .queryable
        .iter()
        .all(|path| producer.queryable.contains(path))
    {
        return Some("skip parent does not preserve child queryable fields");
    }
    None
}

fn metadata_for_all_branches(plan: &OperationOverlay) -> bool {
    if plan.branches.is_empty() {
        return plan.emission == Emission::EmitMetadata;
    }
    plan.branches.iter().all(|branch| match branch.emission {
        Some(emission) => emission == Emission::EmitMetadata,
        None => plan.emission == Emission::EmitMetadata,
    })
}

fn filters_emissions(plan: &OperationOverlay) -> bool {
    let filters = |policy: &crate::overlay::RowPolicy| {
        policy.owned.is_some() || !policy.assertions.is_empty()
    };
    filters(&plan.row_policy)
        || plan
            .branches
            .iter()
            .any(|branch| branch.row_policy.as_ref().is_some_and(filters))
}

pub(super) fn validate_receipts(
    catalog: &RuntimeCatalog<'_>,
    checked: &CheckedOverlays,
) -> Result<(), RuntimeCatalogCompileError> {
    checked
        .services()
        .iter()
        .flat_map(|service| service.callable())
        .try_for_each(|parent| {
            parent
                .plan()
                .skip_evidence
                .values()
                .try_for_each(|receipt| {
                    let reject = |path| RuntimeCatalogCompileError::CapabilityContext {
                        service: parent.service().to_owned(),
                        operation: parent.operation().to_owned(),
                        path,
                    };
                    let child = receipt.describe.as_str();
                    let digest = SkipEvidence::fingerprint(
                        catalog,
                        &receipt.service,
                        receipt.list.as_str(),
                        child,
                    )
                    .map_err(|error| reject(format!("skip {child} receipt: {error}")))?;
                    if digest != receipt.inputs_sha256 {
                        return Err(reject(format!(
                            "skip {child} receipt input SHA-256 does not match"
                        )));
                    }
                    validate_compiled_emissions(catalog, receipt).map_err(reject)
                })
        })
}

fn validate_compiled_emissions(
    catalog: &RuntimeCatalog<'_>,
    receipt: &SkipEvidence,
) -> Result<(), String> {
    let lookup = |name| {
        catalog
            .lookup(&receipt.service, name)
            .ok_or_else(|| format!("skip callable {name} is missing"))
    };
    let parent = lookup(receipt.list.as_str())?;
    let child = lookup(receipt.describe.as_str())?;
    let context = child.context_projection();
    if !context.selected.is_empty() || !context.response.is_empty() || !context.request.is_empty() {
        return Err(format!(
            "skip {} produces downstream context",
            child.operation_name()
        ));
    }
    if parent.routing() != child.routing()
        || stages(parent).any(|(scope, region, emission)| {
            emission != RuntimeEmission::EmitMetadata
                || stages(child).any(|(child_scope, child_region, child_emission)| {
                    child_emission != RuntimeEmission::EmitMetadata
                        || child_scope != scope
                        || child_region != region
                })
        })
    {
        return Err(format!(
            "skip {} branch resource scope or emission differs",
            child.operation_name()
        ));
    }
    child.resource_emissions().iter().try_for_each(|emission| {
        let producer = parent
            .resource_emissions()
            .iter()
            .find(|candidate| candidate.kind() == emission.kind())
            .ok_or_else(|| format!("skip child emission {} is missing", emission.kind()))?;
        validate_compiled_emission(catalog, parent, child, producer, emission)
    })
}

fn stages<'a>(
    plan: &'a CallablePlan<'_>,
) -> impl Iterator<Item = (PlanScope, Option<&'a str>, RuntimeEmission)> {
    std::iter::once((plan.scope(), None, plan.emission()))
        .filter(move |_| plan.branches().is_empty())
        .chain(plan.branches().iter().map(move |branch| {
            (
                branch.scope,
                branch.request_region.as_deref(),
                plan.effective_emission(Some(branch)),
            )
        }))
}

fn validate_compiled_emission(
    catalog: &RuntimeCatalog<'_>,
    parent: &CallablePlan<'_>,
    child: &CallablePlan<'_>,
    producer: &ResourceEmissionPlan,
    emission: &ResourceEmissionPlan,
) -> Result<(), String> {
    if emission.tags().is_some() && producer.tags() != emission.tags() {
        return Err(format!("skip {} child tag source differs", emission.kind()));
    }
    if producer.projection() != emission.projection() {
        return Err(format!("skip {} child projection differs", emission.kind()));
    }
    if producer.identity() != emission.identity() || !row_identity(emission.identity()) {
        return Err(format!(
            "skip {} child identity is not preserved from compared rows",
            emission.kind()
        ));
    }
    catalog.relationships().iter().try_for_each(|relation| {
        let rule = || {
            format!(
                "skip {} child relationship {} is not preserved",
                relation.source_kind.as_str(),
                relation.relation.as_str()
            )
        };
        if !emission.emits_kind(&relation.source_kind)
            || !child.supports_relationship(relation).map_err(|_| rule())?
        {
            return Ok(());
        }
        let selected = relation.source == RelationSource::SelectedOutput
            && relation
                .guard
                .as_ref()
                .is_none_or(|guard| guard.source == RelationSource::SelectedOutput)
            && relation
                .components
                .iter()
                .all(|component| component.source == RelationSource::SelectedOutput);
        if !selected || !parent.supports_relationship(relation).map_err(|_| rule())? {
            return Err(rule());
        }
        Ok(())
    })
}

fn row_identity(identity: &IdentityPlan) -> bool {
    use inventory_core::catalog::runtime::{IdentityChoicePlan, IdentityPredicatePlan};
    match identity {
        IdentityPlan::Arn(_) | IdentityPlan::Template(_) | IdentityPlan::Key(_) => true,
        IdentityPlan::CorrelatedKey(fields) => row_fields(fields),
        IdentityPlan::MapArn { request_url, .. } => request_url.is_none(),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => row_leaf(fallback),
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            branches.iter().all(|branch| {
                !matches!(
                    branch.predicate,
                    IdentityPredicatePlan::RequestTextEquals { .. }
                ) && row_leaf(&branch.identity)
            })
        }
    }
}

fn row_leaf(identity: &IdentityLeafPlan) -> bool {
    match identity {
        IdentityLeafPlan::Arn(_) | IdentityLeafPlan::Template(_) | IdentityLeafPlan::Key(_) => true,
        IdentityLeafPlan::Fields(fields) => row_fields(fields),
    }
}

fn row_fields(fields: &[IdentityFieldPlan]) -> bool {
    fields.iter().all(|field| {
        !matches!(
            field,
            IdentityFieldPlan::Request(_)
                | IdentityFieldPlan::RequestArn(_)
                | IdentityFieldPlan::RequestInteger { .. }
                | IdentityFieldPlan::Context { .. }
        )
    })
}

pub(crate) fn check_root(
    root: &std::path::Path,
    checked: &CheckedOverlays,
) -> Result<(), CheckError> {
    if checked
        .services()
        .iter()
        .flat_map(|service| service.callable())
        .all(|plan| plan.plan().skip.is_empty())
    {
        return Ok(());
    }
    let cfn = crate::report::read_text(&root.join("catalog/cloudcontrol-universal.json"))?;
    let kinds =
        crate::kind_map::load(root, cfn.as_bytes()).map_err(|error| CheckError::Invalid {
            service: String::new(),
            operation: String::new(),
            key: "skip.kind_map".to_owned(),
            value: error.to_string(),
            rule: "skip requires a valid kind map",
        })?;
    validate(checked, &kinds)
}
