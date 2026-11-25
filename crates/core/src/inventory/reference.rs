use super::{
    AccountId, EdgeSource, InventoryError, Kind, KindSchema, Region, Relation, ResourceId,
    ResourceKey, ScalarType,
};
use crate::catalog::runtime::RelationTargetMode;
use crate::record::FieldId;

#[derive(Debug, PartialEq, Eq)]
pub struct ReferenceComponent {
    pub mode: RelationTargetMode,
    pub path: Box<[FieldId]>,
    pub value: ResourceId,
}

fn valid_path(mode: RelationTargetMode, path: &[FieldId]) -> bool {
    match mode {
        RelationTargetMode::SelectedFields | RelationTargetMode::BoundContextFields => {
            !path.is_empty() && path.len() <= 8
        }
        RelationTargetMode::SelectedRootText
        | RelationTargetMode::BoundContextRootText
        | RelationTargetMode::CanonicalId => path.is_empty(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReferenceTarget {
    account: AccountId,
    region: Region,
    kind: Kind,
    candidate_kinds: Box<[Kind]>,
    path: Box<[FieldId]>,
    value: ResourceId,
    mode: RelationTargetMode,
    components: Box<[ReferenceComponent]>,
}

impl ReferenceTarget {
    pub fn new(
        account: AccountId,
        region: Region,
        kind: Kind,
        path: Vec<FieldId>,
        value: ResourceId,
    ) -> Result<Self, InventoryError> {
        Self::typed(
            account,
            region,
            kind,
            path,
            value,
            RelationTargetMode::SelectedFields,
        )
    }

    pub fn typed(
        account: AccountId,
        region: Region,
        kind: Kind,
        path: Vec<FieldId>,
        value: ResourceId,
        mode: RelationTargetMode,
    ) -> Result<Self, InventoryError> {
        if !valid_path(mode, &path) {
            return Err(InventoryError::Path);
        }
        Ok(Self {
            account,
            region,
            kind,
            candidate_kinds: Box::new([]),
            path: path.into_boxed_slice(),
            value,
            mode,
            components: Box::new([]),
        })
    }

    pub fn mode(&self) -> RelationTargetMode {
        self.mode
    }
    pub fn components(&self) -> &[ReferenceComponent] {
        &self.components
    }
    pub fn with_components(
        mut self,
        components: Vec<ReferenceComponent>,
    ) -> Result<Self, InventoryError> {
        if components.len() > 7
            || components
                .iter()
                .any(|term| !valid_path(term.mode, &term.path))
        {
            return Err(InventoryError::Path);
        }
        self.components = components.into_boxed_slice();
        Ok(self)
    }

    pub fn root_text(
        account: AccountId,
        region: Region,
        schema: &KindSchema,
        value: ResourceId,
    ) -> Result<Self, InventoryError> {
        if !schema
            .columns()
            .iter()
            .any(|column| column.path().is_empty() && column.scalar() == ScalarType::Text)
        {
            return Err(InventoryError::Path);
        }
        Self::typed(
            account,
            region,
            schema.kind().clone(),
            Vec::new(),
            value,
            RelationTargetMode::SelectedRootText,
        )
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }
    pub fn region(&self) -> &Region {
        &self.region
    }
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
    pub fn with_candidate_kinds(mut self, mut kinds: Vec<Kind>) -> Result<Self, InventoryError> {
        if kinds.is_empty() || kinds.len() > 64 || !kinds.contains(&self.kind) {
            return Err(InventoryError::ReferenceKinds);
        }
        kinds.sort_unstable();
        if kinds
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left == right))
        {
            return Err(InventoryError::ReferenceKinds);
        }
        self.candidate_kinds = if kinds.len() == 1 {
            Box::new([])
        } else {
            kinds.into_boxed_slice()
        };
        Ok(self)
    }

    pub fn candidate_kinds(&self) -> &[Kind] {
        if self.candidate_kinds.is_empty() {
            std::slice::from_ref(&self.kind)
        } else {
            &self.candidate_kinds
        }
    }
    pub fn path(&self) -> &[FieldId] {
        &self.path
    }
    pub fn value(&self) -> &ResourceId {
        &self.value
    }
}

#[derive(Debug)]
pub struct UnresolvedReference {
    pub from: ResourceKey,
    pub relation: Relation,
    pub source: EdgeSource,
    pub target: ReferenceTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Result<ReferenceTarget, InventoryError> {
        ReferenceTarget::new(
            AccountId::parse("123456789012")?,
            Region::parse("us-east-1")?,
            Kind::parse("AWS::MSK::Cluster")?,
            vec![FieldId::new(1)?],
            ResourceId::parse("cluster-arn")?,
        )
    }

    #[test]
    fn given_single_reference_when_no_alternatives_then_primary_is_the_only_candidate()
    -> Result<(), InventoryError> {
        let target = target()?;
        assert_eq!(
            target.candidate_kinds(),
            std::slice::from_ref(target.kind())
        );
        Ok(())
    }

    #[test]
    fn given_reviewed_alternatives_when_constructed_then_candidates_are_unique_and_ordered()
    -> Result<(), InventoryError> {
        let cluster = Kind::parse("AWS::MSK::Cluster")?;
        let serverless = Kind::parse("AWS::MSK::ServerlessCluster")?;
        let target = target()?.with_candidate_kinds(vec![serverless.clone(), cluster.clone()])?;
        assert_eq!(target.candidate_kinds(), &[cluster, serverless]);
        Ok(())
    }

    #[test]
    fn given_invalid_candidate_sets_when_constructed_then_domain_rejects_them()
    -> Result<(), InventoryError> {
        let cluster = Kind::parse("AWS::MSK::Cluster")?;
        let other = Kind::parse("AWS::MSK::ServerlessCluster")?;
        let excessive = std::iter::once(cluster.clone()).chain(
            (0..64)
                .map(|index| Kind::parse(&format!("AWS::Fixture::Kind{index}")))
                .collect::<Result<Vec<_>, _>>()?,
        );
        let cases = [
            Vec::new(),
            vec![cluster.clone(), cluster.clone()],
            vec![other],
            excessive.collect(),
        ];
        for kinds in cases {
            assert!(matches!(
                target()?.with_candidate_kinds(kinds),
                Err(InventoryError::ReferenceKinds)
            ));
        }
        Ok(())
    }

    #[test]
    fn given_candidate_boundary_when_one_and_sixty_four_distinct_kinds_then_both_are_accepted()
    -> Result<(), InventoryError> {
        let primary = Kind::parse("AWS::MSK::Cluster")?;
        assert_eq!(
            target()?
                .with_candidate_kinds(vec![primary.clone()])?
                .candidate_kinds(),
            std::slice::from_ref(&primary)
        );
        let maximum = std::iter::once(primary)
            .chain(
                (0..63)
                    .map(|index| Kind::parse(&format!("AWS::Fixture::Kind{index}")))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .collect();
        assert_eq!(
            target()?
                .with_candidate_kinds(maximum)?
                .candidate_kinds()
                .len(),
            64
        );
        Ok(())
    }

    #[test]
    fn given_selected_text_schema_when_targeting_root_then_only_that_text_root_is_accepted()
    -> Result<(), InventoryError> {
        use super::super::{Column, SqlName};

        for scalar in [
            ScalarType::Text,
            ScalarType::Bool,
            ScalarType::Integer,
            ScalarType::Real,
            ScalarType::Blob,
            ScalarType::Timestamp,
        ] {
            let schema = KindSchema::new(
                Kind::parse("AWS::Fixture::Scalar")?,
                SqlName::parse("scalar")?,
                vec![Column::selected_scalar(scalar)],
            )?;
            let result = ReferenceTarget::root_text(
                AccountId::parse("123456789012")?,
                Region::Global,
                &schema,
                ResourceId::parse("selected-value")?,
            );
            if scalar == ScalarType::Text {
                let target = result?;
                assert!(target.path().is_empty());
                assert_eq!(target.kind(), schema.kind());
                assert_eq!(target.value().as_str(), "selected-value");
            } else {
                assert!(matches!(result, Err(InventoryError::Path)));
            }
        }
        for columns in [
            Vec::new(),
            vec![Column::new(vec![FieldId::new(1)?], ScalarType::Text)?],
        ] {
            let schema = KindSchema::new(
                Kind::parse("AWS::Fixture::Object")?,
                SqlName::parse("object")?,
                columns,
            )?;
            assert!(matches!(
                ReferenceTarget::root_text(
                    AccountId::parse("123456789012")?,
                    Region::Global,
                    &schema,
                    ResourceId::parse("selected-value")?
                ),
                Err(InventoryError::Path)
            ));
        }
        assert!(matches!(
            ReferenceTarget::new(
                AccountId::parse("123456789012")?,
                Region::Global,
                Kind::parse("AWS::Fixture::Scalar")?,
                Vec::new(),
                ResourceId::parse("selected-value")?
            ),
            Err(InventoryError::Path)
        ));
        Ok(())
    }

    #[test]
    fn given_typed_target_modes_when_paths_cross_boundaries_then_only_the_matching_path_domain_is_accepted()
    -> Result<(), InventoryError> {
        use RelationTargetMode::*;
        for mode in [
            SelectedFields,
            SelectedRootText,
            BoundContextFields,
            BoundContextRootText,
            CanonicalId,
        ] {
            for length in [0, 1, 8, 9] {
                let path = vec![FieldId::new(1)?; length];
                let result = ReferenceTarget::typed(
                    AccountId::parse("123456789012")?,
                    Region::Global,
                    Kind::parse("AWS::Fixture::Object")?,
                    path,
                    ResourceId::parse("value")?,
                    mode,
                );
                let expected = match mode {
                    SelectedFields | BoundContextFields => (1..=8).contains(&length),
                    _ => length == 0,
                };
                if expected {
                    let target = result?;
                    assert_eq!(
                        (target.mode(), target.path().len(), target.value().as_str()),
                        (mode, length, "value")
                    );
                } else {
                    assert!(
                        matches!(result, Err(InventoryError::Path)),
                        "{mode:?} length {length}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn given_additional_terms_when_count_or_path_exceeds_bounds_then_the_compound_target_is_rejected()
    -> Result<(), InventoryError> {
        for count in [0, 1, 7, 8] {
            for length in [0, 1, 8, 9] {
                let terms = (0..count)
                    .map(|_| {
                        Ok(ReferenceComponent {
                            mode: RelationTargetMode::BoundContextFields,
                            path: vec![FieldId::new(1)?; length].into_boxed_slice(),
                            value: ResourceId::parse("component")?,
                        })
                    })
                    .collect::<Result<Vec<_>, InventoryError>>()?;
                let result = target()?.with_components(terms);
                if count <= 7 && (count == 0 || (1..=8).contains(&length)) {
                    assert_eq!(result?.components().len(), count);
                } else {
                    assert!(
                        matches!(result, Err(InventoryError::Path)),
                        "count {count} length {length}"
                    );
                }
            }
        }
        Ok(())
    }
}
