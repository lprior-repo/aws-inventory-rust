use super::KindMapError;
use super::target_kinds::ReviewedTargetKinds;
use super::validation::{row_error, valid_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationPath<'a> {
    RootText,
    Fields(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationSource<'a> {
    SelectedOutput(RelationPath<'a>),
    Request(RelationPath<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationTarget<'a> {
    SelectedOutput(RelationPath<'a>),
    BoundContext(RelationPath<'a>),
    CanonicalId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewedValuePath {
    RootText,
    Fields(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewedRelationSource {
    SelectedOutput(ReviewedValuePath),
    Request(ReviewedValuePath),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewedRelationTarget {
    SelectedOutput(ReviewedValuePath),
    BoundContext(ReviewedValuePath),
    CanonicalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationGuardMapping<'a> {
    pub source: RelationSource<'a>,
    pub equals: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewedRelationGuard {
    pub source: ReviewedRelationSource,
    pub equals: String,
}

impl RelationGuardMapping<'_> {
    pub(super) fn owned(self, index: usize) -> Result<ReviewedRelationGuard, KindMapError> {
        Ok(ReviewedRelationGuard {
            source: self.source.owned(index)?,
            equals: self.equals.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedRelation {
    pub source_kind: String,
    pub source: ReviewedRelationSource,
    pub target_kinds: ReviewedTargetKinds,
    pub target: ReviewedRelationTarget,
    pub relation: String,
    pub scope: String,
    pub guard: Option<ReviewedRelationGuard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationMapping<'a> {
    pub source_kind: &'a str,
    pub source: RelationSource<'a>,
    pub target_kind: &'a str,
    pub target: RelationTarget<'a>,
    pub relation: &'a str,
    pub scope: &'a str,
    pub guard: Option<RelationGuardMapping<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationTargetsMapping<'a> {
    pub source_kind: &'a str,
    pub source: RelationSource<'a>,
    pub relation: &'a str,
    pub target_kinds: &'a [&'a str],
}

impl RelationPath<'_> {
    pub(super) fn owned(self, index: usize) -> Result<ReviewedValuePath, KindMapError> {
        match self {
            Self::RootText => Ok(ReviewedValuePath::RootText),
            Self::Fields(path) if valid_path(path) => {
                Ok(ReviewedValuePath::Fields(path.to_owned()))
            }
            Self::Fields(_) => Err(row_error(
                index,
                "relation fields require a nonempty modeled path",
            )),
        }
    }
}

impl RelationSource<'_> {
    pub(super) fn matches(self, reviewed: &ReviewedRelationSource) -> bool {
        match (self, reviewed) {
            (Self::SelectedOutput(declared), ReviewedRelationSource::SelectedOutput(owned))
            | (Self::Request(declared), ReviewedRelationSource::Request(owned)) => {
                match (declared, owned) {
                    (RelationPath::RootText, ReviewedValuePath::RootText) => true,
                    (RelationPath::Fields(left), ReviewedValuePath::Fields(right)) => left == right,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(super) fn owned(self, index: usize) -> Result<ReviewedRelationSource, KindMapError> {
        match self {
            Self::SelectedOutput(path) => {
                Ok(ReviewedRelationSource::SelectedOutput(path.owned(index)?))
            }
            Self::Request(path) => Ok(ReviewedRelationSource::Request(path.owned(index)?)),
        }
    }
}

impl RelationTarget<'_> {
    pub(super) fn owned(self, index: usize) -> Result<ReviewedRelationTarget, KindMapError> {
        match self {
            Self::SelectedOutput(path) => {
                Ok(ReviewedRelationTarget::SelectedOutput(path.owned(index)?))
            }
            Self::BoundContext(path) => {
                Ok(ReviewedRelationTarget::BoundContext(path.owned(index)?))
            }
            Self::CanonicalId => Ok(ReviewedRelationTarget::CanonicalId),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn root_text_and_canonical_identity_are_not_fictitious_modeled_paths() -> TestResult {
        let root = RelationSource::SelectedOutput(RelationPath::RootText).owned(0)?;
        let canonical = RelationTarget::CanonicalId.owned(0)?;
        assert_eq!(
            root,
            ReviewedRelationSource::SelectedOutput(ReviewedValuePath::RootText)
        );
        assert_eq!(canonical, ReviewedRelationTarget::CanonicalId);
        for path in [
            "",
            "$",
            "$id",
            "nested..field",
            "bad name",
            "a.b.c.d.e.f.g.h.i",
        ] {
            assert!(RelationPath::Fields(path).owned(0).is_err(), "{path}");
        }
        Ok(())
    }

    #[test]
    fn request_and_bound_context_namespaces_remain_distinct_from_selected_output() -> TestResult {
        let path = RelationPath::Fields("farmId");
        let output = RelationSource::SelectedOutput(path).owned(0)?;
        let request = RelationSource::Request(path).owned(0)?;
        let selected_target = RelationTarget::SelectedOutput(path).owned(0)?;
        let context_target = RelationTarget::BoundContext(path).owned(0)?;
        assert_ne!(output, request);
        assert_ne!(selected_target, context_target);
        assert_eq!(
            context_target,
            ReviewedRelationTarget::BoundContext(ReviewedValuePath::Fields("farmId".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn unary_guard_preserves_actual_source_namespace_and_exact_enum_literal() -> TestResult {
        let selected = RelationGuardMapping {
            source: RelationSource::SelectedOutput(RelationPath::Fields("ResourceType")),
            equals: "WORKSPACE",
        };
        let request = RelationGuardMapping {
            source: RelationSource::Request(RelationPath::Fields("ResourceType")),
            ..selected
        };
        let owned = selected.owned(0)?;
        assert_eq!(owned.equals, "WORKSPACE");
        assert_eq!(
            owned.source,
            ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(
                "ResourceType".to_owned()
            ))
        );
        assert_ne!(owned, request.owned(0)?);
        assert_ne!(
            owned,
            RelationGuardMapping {
                equals: "BUNDLE",
                ..selected
            }
            .owned(0)?
        );
        assert!(
            RelationGuardMapping {
                source: RelationSource::SelectedOutput(RelationPath::Fields("$")),
                ..selected
            }
            .owned(0)
            .is_err()
        );
        Ok(())
    }
}
