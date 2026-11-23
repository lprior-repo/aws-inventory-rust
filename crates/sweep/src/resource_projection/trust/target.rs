use super::*;
use inventory_core::record::FieldId;

pub(super) fn target(
    catalog: &RuntimeCatalog<'_>,
    observation: &Observation,
    arn: &str,
) -> Result<Option<ReferenceTarget>, SweepError> {
    let Some((account, kind)) = parsed_target(observation, arn)? else {
        return Ok(None);
    };
    let Some(path) = identity_path(catalog, &kind)? else {
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

fn parsed_target(
    observation: &Observation,
    arn: &str,
) -> Result<Option<(AccountId, Kind)>, SweepError> {
    if !arn.starts_with("arn:") {
        return Ok(None);
    }
    let partition = source_partition(observation, arn)?;
    let Some((account, resource)) = principal_arn(arn, partition)? else {
        return Ok(None);
    };
    let kind = inventory_core::inventory::iam_principal_resource_kind(resource)
        .ok_or(SweepError::Operation("invalid reviewed IAM principal ARN"))?
        .canonical_kind();
    Ok(Some((account, Kind::parse(kind)?)))
}

fn source_partition<'a>(observation: &'a Observation, arn: &str) -> Result<&'a str, SweepError> {
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
    Ok(source_partition)
}

fn principal_arn<'a>(
    arn: &'a str,
    partition: &str,
) -> Result<Option<(AccountId, &'a str)>, SweepError> {
    let mut parts = arn.splitn(6, ':');
    if parts.next() != Some("arn")
        || parts.next() != Some(partition)
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
    if !supported_resource(resource) {
        return Ok(None);
    }
    Ok(Some((account, resource)))
}

fn supported_resource(resource: &str) -> bool {
    let Some((prefix, _)) = resource.split_once('/') else {
        return false;
    };
    matches!(prefix, "role" | "user" | "saml-provider" | "oidc-provider")
}

fn identity_path(
    catalog: &RuntimeCatalog<'_>,
    kind: &Kind,
) -> Result<Option<Vec<FieldId>>, SweepError> {
    let Some(schema) = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind() == kind)
    else {
        return Ok(None);
    };
    schema.columns().iter().try_fold(None, |found, column| {
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
    })
}
