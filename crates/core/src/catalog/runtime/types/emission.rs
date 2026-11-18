use super::{IdentityPlan, ItemPlan, KindClassificationPlan, ResourceProjectionPlan, TagPlan};
use crate::{inventory::Kind, record::Exclusions};

/// A prevalidated resource selection sharing its operation's request and cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEmissionPlan {
    pub(crate) key: Box<str>,
    pub(crate) kind: Kind,
    pub(crate) items: ItemPlan,
    pub(crate) identity: IdentityPlan,
    pub(crate) tags: Option<TagPlan>,
    pub(crate) exclusions: Exclusions,
    pub(crate) projection: Option<ResourceProjectionPlan>,
    pub(crate) classification: Option<KindClassificationPlan>,
}

impl ResourceEmissionPlan {
    /// The operation's primary selection, keyed `primary`, as decoded from the plan's own fields.
    pub(crate) fn primary(
        kind: &Kind,
        items: &ItemPlan,
        identity: &IdentityPlan,
        tags: Option<&TagPlan>,
        exclusions: &Exclusions,
        projection: Option<&ResourceProjectionPlan>,
        classification: Option<&KindClassificationPlan>,
    ) -> Self {
        Self {
            key: "primary".into(),
            kind: kind.clone(),
            items: items.clone(),
            identity: identity.clone(),
            tags: tags.cloned(),
            exclusions: exclusions.clone(),
            projection: projection.cloned(),
            classification: classification.cloned(),
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
    pub fn items(&self) -> &ItemPlan {
        &self.items
    }
    pub fn identity(&self) -> &IdentityPlan {
        &self.identity
    }
    /// None is unobserved, not an invented empty tag snapshot.
    pub fn tags(&self) -> Option<&TagPlan> {
        self.tags.as_ref()
    }
    pub fn hash_exclusions(&self) -> &Exclusions {
        &self.exclusions
    }
    pub fn projection(&self) -> Option<&ResourceProjectionPlan> {
        self.projection.as_ref()
    }
    pub fn classification(&self) -> Option<&KindClassificationPlan> {
        self.classification.as_ref()
    }
    pub fn emits_kind(&self, kind: &Kind) -> bool {
        self.kind() == kind
            || self.classification().is_some_and(|classification| {
                classification
                    .variants
                    .iter()
                    .any(|(_, candidate)| candidate == kind)
            })
    }

    /// Kind evidence only: stage, source, identity and receipt checks remain separate.
    pub fn matches_observed_kind(
        &self,
        kind: &Kind,
        record: &crate::record::Record,
    ) -> Result<bool, super::super::RuntimeCatalogError> {
        if !self.emits_kind(kind) {
            return Ok(false);
        }
        self.classification().map_or_else(
            || Ok(self.kind() == kind),
            |classification| classification.classify(record).map(|actual| actual == kind),
        )
    }
}

pub const MAX_RESOURCE_EMISSIONS: usize = 8;
