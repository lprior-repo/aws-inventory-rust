//! Bounded network observation page assembly.
use super::*;

impl NetworkPage {
    pub(super) fn components(&mut self, counts: &[usize]) -> Result<(), AwsScanError> {
        let count = counts
            .iter()
            .try_fold(0usize, |total, count| total.checked_add(*count))
            .ok_or_else(|| AwsScanError::InvalidData("network component count overflow".into()))?;
        if count > MAX_COMPONENTS_PER_RESOURCE {
            return Err(AwsScanError::InvalidData(format!(
                "network resource exceeds {MAX_COMPONENTS_PER_RESOURCE} nested components"
            )));
        }
        self.components = self.components.checked_add(count).ok_or_else(|| {
            AwsScanError::InvalidData("network page component count overflow".into())
        })?;
        if self.components > MAX_COMPONENTS_PER_PAGE {
            return Err(AwsScanError::InvalidData(format!(
                "network page exceeds {MAX_COMPONENTS_PER_PAGE} nested components"
            )));
        }
        Ok(())
    }

    pub(super) fn link(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        reference: Option<Reference<'_>>,
        relation: &str,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let Some(reference) = reference else {
            return Ok(());
        };
        let (target, properties) = reference.resolve(ctx);
        self.link_resolved(ctx, resource, target, relation, properties, evidence)
    }

    pub(super) fn arn_link(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        arn: Option<&str>,
        relation: &str,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let Some(arn) = arn.filter(|arn| !arn.is_empty()) else {
            return Ok(());
        };
        if !arn.starts_with("arn:") {
            return Err(AwsScanError::InvalidData(
                "AWS returned a non-ARN network target in an ARN field".into(),
            ));
        }
        self.link_resolved(
            ctx,
            resource,
            arn.to_owned(),
            relation,
            json!({
                "target_resource_id": arn,
                "target_account_id": arn.split(':').nth(4),
                "target_region": arn.split(':').nth(3),
                "owner_resolution": "aws_returned_arn",
                "region_resolution": "aws_returned_arn",
            }),
            evidence,
        )
    }

    pub(super) fn link_resolved(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        target: String,
        relation: &str,
        properties: Value,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let key = (resource.id.clone(), target.clone(), relation.to_owned());
        if let Some(edge) = self.edges.get_mut(&key) {
            let observations = edge
                .properties
                .get_mut("evidence")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| {
                    AwsScanError::InvalidData("network edge evidence is not an array".into())
                })?;
            if observations.len() >= MAX_COMPONENTS_PER_RESOURCE {
                return Err(AwsScanError::InvalidData(
                    "network edge evidence capacity exceeded".into(),
                ));
            }
            observations.push(evidence);
        } else {
            if self.edges.len() >= MAX_PAGE_EDGES {
                return Err(AwsScanError::InvalidData(
                    "network page edge capacity exceeded".into(),
                ));
            }
            let mut edge = ctx.edge(resource, target, relation);
            edge.properties = properties;
            let object = edge.properties.as_object_mut().ok_or_else(|| {
                AwsScanError::InvalidData("network reference evidence is not an object".into())
            })?;
            object.insert("evidence".into(), json!([evidence]));
            self.edges.insert(key, edge);
        }
        Ok(())
    }

    pub(super) fn finish(
        self,
        next_token: Option<&str>,
        listed_count: usize,
    ) -> Result<PageBody, AwsScanError> {
        {
            let mut resource_ids = BTreeSet::new();
            for resource in &self.resources {
                if !resource_ids.insert(resource.id.as_str()) {
                    return Err(AwsScanError::InvalidData(
                        "network page repeats a resource identity".into(),
                    ));
                }
            }
        }
        let edges = self.edges.into_values().collect::<Vec<_>>();
        for edge in &edges {
            edge.validate()?;
        }
        Ok(PageBody {
            resources: self.resources,
            edges,
            next_token: next_token
                .filter(|token| !token.is_empty())
                .map(str::to_owned),
            listed_count,
        })
    }
}
