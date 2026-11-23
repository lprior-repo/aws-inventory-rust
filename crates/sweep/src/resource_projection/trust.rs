mod target;
use target::target;
use super::*;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{EdgeSource, ReferenceTarget, Relation, ResourceId, UnresolvedReference},
};

pub(crate) fn trust_references(
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
    Trust { catalog, observation, relation }.collect(summary)
}

struct Trust<'a, 'catalog> {
    catalog: &'a RuntimeCatalog<'catalog>,
    observation: &'a Observation,
    relation: Relation,
}

impl Trust<'_, '_> {
    fn collect(&self, summary: MapRef<'_>) -> Result<Vec<UnresolvedReference>, SweepError> {
        let Some(ValueRef::List(statements)) = summary.get_name("trust_statements")? else {
            return Err(SweepError::Operation("trust projection lacks bounded statements"));
        };
        if statements.count()? > 128 {
            return Err(SweepError::Operation("trust statement bound exceeded"));
        }
        statements.iter()?.try_fold(Vec::new(), |output, statement| {
            self.statement(output, statement?)
        })
    }

    fn statement(&self, output: Vec<UnresolvedReference>, statement: ValueRef<'_>) -> Result<Vec<UnresolvedReference>, SweepError> {
        let ValueRef::Map(statement) = statement else {
            return Err(SweepError::Operation("invalid trust summary statement"));
        };
        if !matches!(statement.get_name("effect")?, Some(ValueRef::Text("Allow"))) { return Ok(output); }
        let Some(ValueRef::Map(principals)) = statement.get_name("principals")? else { return Ok(output); };
        principals.iter()?.try_fold(output, |output, principal| {
            let (principal_type, values) = principal?;
            if !matches!(principal_type, inventory_core::record::KeyRef::Name("AWS" | "Federated")) {
                return Ok(output);
            }
            self.principals(output, values)
        })
    }

    fn principals(&self, output: Vec<UnresolvedReference>, values: ValueRef<'_>) -> Result<Vec<UnresolvedReference>, SweepError> {
        let ValueRef::List(values) = values else {
            return Err(SweepError::Operation("invalid trust principal list"));
        };
        values.iter()?.try_fold(output, |output, value| {
            let ValueRef::Text(value) = value? else {
                return Err(SweepError::Operation("invalid trust principal"));
            };
            self.append(output, value)
        })
    }

    fn append(&self, mut output: Vec<UnresolvedReference>, value: &str) -> Result<Vec<UnresolvedReference>, SweepError> {
        if let Some(target) = target(self.catalog, self.observation, value)? {
            if output.len() >= 256 { return Err(SweepError::Operation("trust principal bound exceeded")); }
            output.try_reserve(1)?;
            output.push(UnresolvedReference {
                from: self.observation.key().clone(),
                relation: self.relation.clone(),
                source: EdgeSource::ReviewedRule,
                target,
            });
        }
        Ok(output)
    }
}
