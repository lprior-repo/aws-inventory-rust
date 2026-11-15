use super::StoreError;
use inventory_core::{
    catalog::runtime::{CallablePlan, ResourceProjectionPlan, RuntimeCatalog},
    inventory::{AccountId, Edge, Kind, Observation, Region, ResourceKey, ScalarType, UnresolvedReference},
    record::{FieldId, KeyRef, ValueRef},
};

#[derive(Clone, Debug)]
pub(super) struct TrustRule {
    source: Kind,
    target: Kind,
    path: Box<[FieldId]>,
    summary_path: Box<[FieldId]>,
    prefix: &'static str,
}

pub(super) fn rules(catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>) -> Result<Vec<TrustRule>, StoreError> {
    let Some(ResourceProjectionPlan::IamTrustSummary { path }) = plan.projection() else { return Ok(Vec::new()); };
    let mut rules = Vec::new();
    for (kind, prefix) in [
        ("AWS::IAM::Role", "role"), ("AWS::IAM::User", "user"),
        ("AWS::IAM::SAMLProvider", "saml-provider"), ("AWS::IAM::OIDCProvider", "oidc-provider"),
    ] {
        let Some(schema) = catalog.schemas().iter().find(|schema| schema.kind().as_str() == kind) else { continue; };
        for column in schema.columns() {
            let [field] = column.path() else { continue; };
            if column.scalar() == ScalarType::Text && catalog.catalog().field_name(*field)? == "Arn" {
                rules.push(TrustRule { source: plan.kind().clone(), target: schema.kind().clone(),
                    path: column.path().into(), prefix, summary_path: path.clone() });
            }
        }
    }
    Ok(rules)
}

impl TrustRule {
    pub(super) fn dependency_path(&self) -> &[FieldId] { &self.summary_path }
    pub(super) fn target_path(&self) -> &[FieldId] { &self.path }
    pub(super) fn reference(&self, reference: &UnresolvedReference) -> bool {
        let target = &reference.target;
        reference.relation.as_str() == "DECLARES_TRUST" && reference.from.kind == self.source
            && target.kind() == &self.target && target.candidate_kinds().len() == 1
            && target.path() == self.path.as_ref()
            && self.owner(&reference.from, target.account(), target.region(), target.value().as_str())
    }

    pub(super) fn edge(&self, edge: &Edge) -> bool {
        edge.relation.as_str() == "DECLARES_TRUST" && edge.from.kind == self.source
            && edge.to.kind == self.target && self.owner(&edge.from, &edge.to.account, &edge.to.region, edge.to.id.as_str())
    }

    pub(super) fn declares(&self, source: &Observation, expected: &str) -> Result<bool, StoreError> {
        let Some(ValueRef::Map(summary)) = super::scalar(source.record().root(), &self.summary_path)? else { return Ok(false); };
        let Some(ValueRef::List(statements)) = summary.get_name("trust_statements")? else { return Ok(false); };
        if statements.count()? > 128 { return Err(StoreError::Invalid("reviewed trust statement bound exceeded")); }
        let mut principals = 0usize;
        for statement in statements.iter()? {
            let ValueRef::Map(statement) = statement? else { return Err(StoreError::Invalid("invalid reviewed trust statement")); };
            if !matches!(statement.get_name("effect")?, Some(ValueRef::Text("Allow"))) { continue; }
            let Some(ValueRef::Map(values)) = statement.get_name("principals")? else { continue; };
            for entry in values.iter()? {
                let (kind, values) = entry?;
                if !matches!(kind, KeyRef::Name("AWS" | "Federated")) { continue; }
                let ValueRef::List(values) = values else { return Err(StoreError::Invalid("invalid reviewed trust principal list")); };
                principals = principals.checked_add(values.count()?).ok_or(StoreError::Invalid("reviewed trust principal overflow"))?;
                if principals > 256 { return Err(StoreError::Invalid("reviewed trust principal bound exceeded")); }
                for value in values.iter()? {
                    if matches!(value?, ValueRef::Text(value) if value == expected) { return Ok(true); }
                }
            }
        }
        Ok(false)
    }

    pub(super) fn target_matches(&self, target: &Observation, expected: &str) -> Result<bool, StoreError> {
        Ok(matches!(super::scalar(target.record().root(), &self.path)?, Some(ValueRef::Text(value)) if value == expected))
    }

    fn owner(&self, source: &ResourceKey, account: &AccountId, region: &Region, arn: &str) -> bool {
        if *region != Region::Global || !arn.is_ascii() || arn.len() > 4096 { return false; }
        let Some(partition) = source.id.as_str().split(':').nth(1) else { return false; };
        if !matches!(partition, "aws" | "aws-cn" | "aws-us-gov") { return false; }
        let mut parts = arn.splitn(6, ':');
        if parts.next() != Some("arn") || parts.next() != Some(partition)
            || parts.next() != Some("iam") || parts.next() != Some("")
            || parts.next() != Some(account.as_str()) { return false; }
        let Some(resource) = parts.next() else { return false; };
        resource.split_once('/').is_some_and(|(prefix, _)| prefix == self.prefix)
            && inventory_core::inventory::iam_principal_resource_kind(resource)
                .is_some_and(|kind| kind.canonical_kind() == self.target.as_str())
    }
}
