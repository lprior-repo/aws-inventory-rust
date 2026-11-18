use super::*;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{EdgeSource, ReferenceTarget, Relation, ResourceId, UnresolvedReference},
};

pub(in crate::sweep_runtime) fn trust_references(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    observation: &Observation,
) -> Result<Vec<UnresolvedReference>, SweepError> {
    let Some(ResourceProjectionPlan::IamTrustSummary { path }) = plan.projection() else {
        return Ok(Vec::new());
    };
    let relation = Relation::parse("DECLARES_TRUST")?;
    let Some(ValueRef::Map(summary)) = record::path_value(observation.record().root(), path)?
    else {
        return Ok(Vec::new());
    };
    let Some(ValueRef::List(statements)) = summary.get_name("trust_statements")? else {
        return Err(SweepError::Operation(
            "trust projection lacks bounded statements",
        ));
    };
    if statements.count()? > 128 {
        return Err(SweepError::Operation("trust statement bound exceeded"));
    }
    statements
        .iter()?
        .try_fold(Vec::new(), |output, statement| {
            let ValueRef::Map(statement) = statement? else {
                return Err(SweepError::Operation("invalid trust summary statement"));
            };
            if !matches!(statement.get_name("effect")?, Some(ValueRef::Text("Allow"))) {
                return Ok(output);
            }
            let Some(ValueRef::Map(principals)) = statement.get_name("principals")? else {
                return Ok(output);
            };
            principals.iter()?.try_fold(output, |output, principal| {
                let (principal_type, values) = principal?;
                if !matches!(
                    principal_type,
                    inventory_core::record::KeyRef::Name("AWS" | "Federated")
                ) {
                    return Ok(output);
                }
                let ValueRef::List(values) = values else {
                    return Err(SweepError::Operation("invalid trust principal list"));
                };
                values.iter()?.try_fold(output, |mut output, value| {
                    let ValueRef::Text(value) = value? else {
                        return Err(SweepError::Operation("invalid trust principal"));
                    };
                    if let Some(target) = target(catalog, observation, value)? {
                        if output.len() >= 256 {
                            return Err(SweepError::Operation("trust principal bound exceeded"));
                        }
                        output.try_reserve(1)?;
                        output.push(UnresolvedReference {
                            from: observation.key().clone(),
                            relation: relation.clone(),
                            source: EdgeSource::ReviewedRule,
                            target,
                        });
                    }
                    Ok(output)
                })
            })
        })
}

fn target(
    catalog: &RuntimeCatalog<'_>,
    observation: &Observation,
    arn: &str,
) -> Result<Option<ReferenceTarget>, SweepError> {
    if !arn.starts_with("arn:") {
        return Ok(None);
    }
    let source_partition = observation
        .key()
        .id
        .as_str()
        .split(':')
        .nth(1)
        .ok_or(SweepError::Operation("trust source identity is not an ARN"))?;
    if !matches!(source_partition, "aws" | "aws-cn" | "aws-us-gov")
        || !arn.is_ascii()
        || arn.len() > 4096
    {
        return Err(SweepError::Operation(
            "trust principal ARN exceeds reviewed grammar",
        ));
    }
    let mut parts = arn.splitn(6, ':');
    if parts.next() != Some("arn")
        || parts.next() != Some(source_partition)
        || parts.next() != Some("iam")
        || parts.next() != Some("")
    {
        return Ok(None);
    }
    let account = AccountId::parse(
        parts
            .next()
            .ok_or(SweepError::Operation("trust ARN lacks account"))?,
    )?;
    let resource = parts
        .next()
        .ok_or(SweepError::Operation("trust ARN lacks resource"))?;
    let Some((prefix, _)) = resource.split_once('/') else {
        return Ok(None);
    };
    if !matches!(prefix, "role" | "user" | "saml-provider" | "oidc-provider") {
        return Ok(None);
    }
    let kind = inventory_core::inventory::iam_principal_resource_kind(resource)
        .ok_or(SweepError::Operation("invalid reviewed IAM principal ARN"))?
        .canonical_kind();
    let kind = Kind::parse(kind)?;
    let Some(schema) = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &kind)
    else {
        return Ok(None);
    };
    let path = schema.columns().iter().try_fold(None, |found, column| {
        if column.path().len() != 1 {
            return Ok::<_, SweepError>(found);
        }
        let Some(field) = column.path().first() else {
            return Ok(found);
        };
        if catalog.catalog().field_name(*field)? == "Arn" {
            Ok(Some(column.path().to_vec()))
        } else {
            Ok(found)
        }
    })?;
    let Some(path) = path else {
        return Ok(None);
    };
    Ok(Some(ReferenceTarget::new(
        account,
        Region::Global,
        kind,
        path,
        ResourceId::parse(arn)?,
    )?))
}
