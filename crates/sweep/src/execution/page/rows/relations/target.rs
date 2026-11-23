use super::*;
use inventory_core::inventory::{AccountId, Kind, Region};

impl Context<'_> {
    pub(super) fn target(
        &self,
        relation: &RelationshipPlan,
        value: &str,
    ) -> Result<Option<ReferenceTarget>, SweepError> {
        let Some(components) = self.components(relation)? else { return Ok(None); };
        let target = self.typed_target(relation, value, components)?;
        let target = if relation.target_kinds.len() > 1 {
            target.with_candidate_kinds(relation.target_kinds.to_vec())?
        } else { target };
        Ok(Some(target))
    }

    fn components(&self, relation: &RelationshipPlan) -> Result<Option<Vec<ReferenceComponent>>, SweepError> {
        let mut components = Vec::new();
        components.try_reserve(relation.components.len())?;
        for component in relation.components.iter() {
            let Some(value) = self.value(component.source, &component.source_path)? else { return Ok(None); };
            let Some(text) = text(value, component.source)? else { return Ok(None); };
            self.validate_request_arn(component.source, relation, &text)?;
            components.push(ReferenceComponent {
                mode: component.target_mode,
                path: component.target_path.clone(),
                value: ResourceId::parse(&text)?,
            });
        }
        Ok(Some(components))
    }

    fn typed_target(
        &self,
        relation: &RelationshipPlan,
        value: &str,
        components: Vec<ReferenceComponent>,
    ) -> Result<ReferenceTarget, SweepError> {
        self.validate_request_arn(relation.source, relation, value)?;
        let (account, region, kind) = self.target_owner(relation, value)?;
        ReferenceTarget::typed(
            account, region, kind, relation.target_path.to_vec(), ResourceId::parse(value)?, relation.target_mode,
        )?.with_components(components).map_err(SweepError::from)
    }

    fn target_owner(
        &self,
        relation: &RelationshipPlan,
        value: &str,
    ) -> Result<(AccountId, Region, Kind), SweepError> {
        if relation.source == RelationSource::Parent {
            let parent = self.parent.ok_or(SweepError::Operation("parent relationship has no authoritative parent"))?;
            Ok((parent.account.clone(), parent.region.clone(), parent.kind.clone()))
        } else {
            let (account, region) = resource_projection::reference_owner(
                relation, self.observation, value, self.home,
            )?;
            record::trusted_arn::validate_kind_arn(&relation.target_kind, value)?;
            Ok((account, region, relation.target_kind.clone()))
        }
    }
}
