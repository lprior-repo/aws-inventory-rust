use super::*;

#[cfg(test)]
#[path = "oracle/oracle_tests.rs"]
mod oracle_tests;

impl SeedState {
    pub(super) fn generic_oracle(
        self,
        relationships: &[PreparedRelationship],
    ) -> Result<ScenarioSeedOracle, SeedError> {
        let edges = relationships
            .iter()
            .map(|relationship| self.edge(relationship))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ScenarioSeedOracle {
            resources: self.resources(),
            account_id: self.account,
            region: self.region,
            edges,
        })
    }

    pub(super) fn oracle(
        self,
        relationships: &[PreparedRelationship],
    ) -> Result<TypedSeedOracle, SeedError> {
        let edges = relationships
            .iter()
            .map(|relationship| self.edge(relationship))
            .collect::<Result<Vec<_>, _>>()?;
        let pairs = relationships
            .iter()
            .map(|relationship| self.pair(relationship))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let missing = SeedError::Invalid("seed scenario produced no verified VPC relationship");
        let primary = pairs.first().cloned().ok_or(missing)?;
        let parts = (pairs, self.resources(), edges);
        Ok(assemble(self.account, self.region, primary, parts))
    }

    fn resources(&self) -> Vec<super::super::SeedResourceIdentity> {
        self.outputs
            .iter()
            .flatten()
            .filter_map(|output| output.resource.clone())
            .collect()
    }

    pub(super) fn edge(&self, relationship: &PreparedRelationship) -> Result<Edge, SeedError> {
        let source = self.output(relationship.source)?;
        let from = self.resource(relationship.source)?;
        let target = self.resource(relationship.target)?;
        if text_path(source.record.root(), &relationship.observed_path)? != target.id.as_str() {
            return Err(SeedError::Invalid(
                "response did not confirm the modeled relationship",
            ));
        }
        Ok(Edge {
            from: from.key.clone(),
            to: target.key.clone(),
            relation: relationship.relation.clone(),
            source: EdgeSource::ReviewedRule,
        })
    }

    pub(super) fn pair(
        &self,
        relationship: &PreparedRelationship,
    ) -> Result<Option<SeedResourcePair>, SeedError> {
        let source = self.resource(relationship.source)?;
        let target = self.resource(relationship.target)?;
        let subnet_kind = Kind::parse("AWS::EC2::Subnet")?;
        let vpc_kind = Kind::parse("AWS::EC2::VPC")?;
        if source.kind != subnet_kind
            || target.kind != vpc_kind
            || relationship.relation.as_str() != "IN_VPC"
        {
            return Ok(None);
        }
        let edge = self.edge(relationship)?;
        Ok(Some(SeedResourcePair {
            vpc_id: target.id.clone(),
            subnet_id: source.id.clone(),
            vpc_key: target.key.clone(),
            subnet_key: source.key.clone(),
            relationship: relationship.relation.clone(),
            edge,
        }))
    }

    pub(super) fn resource(
        &self,
        step: super::super::StepId,
    ) -> Result<&SeedResourceIdentity, SeedError> {
        self.output(step)?
            .resource
            .as_ref()
            .ok_or(SeedError::Invalid(
                "relationship endpoint must be a resource step",
            ))
    }

    pub(super) fn output(
        &self,
        step: super::super::scenario::StepId,
    ) -> Result<&SeedOutput, SeedError> {
        self.accepted_output(step).ok_or(SeedError::Invalid(
            "relationship resource output is unavailable",
        ))
    }
}

type OracleParts = (
    Vec<SeedResourcePair>,
    Vec<super::super::SeedResourceIdentity>,
    Vec<Edge>,
);

fn assemble(
    account_id: AccountId,
    region: Region,
    primary: SeedResourcePair,
    (pairs, resources, edges): OracleParts,
) -> TypedSeedOracle {
    TypedSeedOracle {
        account_id,
        region,
        vpc_id: primary.vpc_id,
        subnet_id: primary.subnet_id,
        vpc_key: primary.vpc_key,
        subnet_key: primary.subnet_key,
        relationship: primary.relationship,
        edge: primary.edge,
        pairs,
        resources,
        edges,
    }
}

pub(super) fn validate_oracle_plan(models: &SeedModels<'_>) -> Result<(), SeedError> {
    let subnet_kind = Kind::parse("AWS::EC2::Subnet")?;
    let vpc_kind = Kind::parse("AWS::EC2::VPC")?;
    let found = models.relationships.iter().any(|relationship| {
        let source = models.steps.get(usize::from(relationship.source.0));
        let target = models.steps.get(usize::from(relationship.target.0));
        source.is_some_and(|step| {
            step.resource
                .as_ref()
                .is_some_and(|resource| resource.kind == subnet_kind)
        }) && target.is_some_and(|step| {
            step.resource
                .as_ref()
                .is_some_and(|resource| resource.kind == vpc_kind)
        }) && relationship.relation.as_str() == "IN_VPC"
    });
    if !found {
        return Err(SeedError::Invalid(
            "scenario must declare a modeled subnet-to-VPC oracle relationship",
        ));
    }
    Ok(())
}

pub(super) fn text_path(
    map: MapRef<'_>,
    path: &[super::super::model::Selector],
) -> Result<String, SeedError> {
    match value_path(map, path)? {
        ValueRef::Text(text) => Ok(text.to_owned()),
        _ => Err(SeedError::Invalid("modeled identity output is not text")),
    }
}

pub(super) fn value_path<'a>(
    root: MapRef<'a>,
    path: &[super::super::model::Selector],
) -> Result<ValueRef<'a>, SeedError> {
    use super::super::model::Selector;
    if path.is_empty() {
        return Err(SeedError::Invalid("modeled output path is empty"));
    }
    path.iter()
        .try_fold(ValueRef::Map(root), |value, selector| {
            let value = match (value, selector) {
                (ValueRef::Map(map), Selector::Field(field)) => map.get_field(*field)?,
                (ValueRef::Map(map), Selector::Name(name)) => map.get_name(name)?,
                (ValueRef::List(list), Selector::Index(index)) => list.get(*index)?,
                _ => {
                    return Err(SeedError::Invalid(
                        "modeled output selector has the wrong container type",
                    ));
                }
            };
            value.ok_or(SeedError::Invalid(
                "modeled output field or element is missing",
            ))
        })
}
