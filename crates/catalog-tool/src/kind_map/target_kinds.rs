use super::KindMapError;
use super::validation::{row_error, valid_kind};

pub(super) const MAX_TARGET_KINDS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewedTargetKinds {
    primary: String,
    additional: Vec<String>,
}

impl ReviewedTargetKinds {
    pub fn primary(&self) -> &str {
        &self.primary
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.primary.as_str()).chain(self.additional.iter().map(String::as_str))
    }

    pub(super) fn iter_sorted(&self) -> impl Iterator<Item = &str> {
        let mut additional = self.additional.iter().map(String::as_str).peekable();
        let mut primary = Some(self.primary.as_str());
        std::iter::from_fn(move || {
            if primary.is_some_and(|kind| additional.peek().is_none_or(|next| kind < *next)) {
                primary.take()
            } else {
                additional.next()
            }
        })
    }

    pub fn len(&self) -> usize {
        self.additional.len().saturating_add(1)
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub(super) fn singleton(primary: &str, index: usize) -> Result<Self, KindMapError> {
        if !valid_kind(primary) {
            return Err(row_error(index, "invalid relation target kind"));
        }
        Ok(Self {
            primary: primary.to_owned(),
            additional: Vec::new(),
        })
    }

    pub(super) fn from_members(declared: &[&str], index: usize) -> Result<Self, KindMapError> {
        if declared.len() > MAX_TARGET_KINDS {
            return Err(row_error(
                index,
                "reviewed target group exceeds sixteen kinds",
            ));
        }
        let primary = declared
            .iter()
            .copied()
            .min()
            .ok_or_else(|| row_error(index, "reviewed target group must not be empty"))?;
        Self::from_declared(primary, declared, index)
    }

    pub(super) fn from_declared(
        primary: &str,
        declared: &[&str],
        index: usize,
    ) -> Result<Self, KindMapError> {
        let additional = validated_additional(primary, declared, index)?;
        Ok(Self {
            primary: primary.to_owned(),
            additional,
        })
    }

    pub(super) fn expand(&mut self, declared: &[&str], index: usize) -> Result<(), KindMapError> {
        self.additional = validated_additional(&self.primary, declared, index)?;
        Ok(())
    }
}

fn validated_additional(
    primary: &str,
    declared: &[&str],
    index: usize,
) -> Result<Vec<String>, KindMapError> {
    if declared.is_empty() || declared.len() > MAX_TARGET_KINDS || !declared.contains(&primary) {
        return Err(row_error(
            index,
            "reviewed target group must include its primary kind within bounds",
        ));
    }
    for (position, kind) in declared.iter().enumerate() {
        if !valid_kind(kind) {
            return Err(row_error(index, "invalid reviewed relation target kind"));
        }
        let previous = declared
            .get(..position)
            .ok_or_else(|| row_error(index, "invalid reviewed target group position"))?;
        if previous.contains(kind) {
            return Err(KindMapError::Duplicate((*kind).to_owned()));
        }
    }
    let mut additional = Vec::with_capacity(declared.len().saturating_sub(1));
    additional.extend(
        declared
            .iter()
            .filter(|kind| **kind != primary)
            .map(|kind| (*kind).to_owned()),
    );
    additional.sort();
    Ok(additional)
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const PRIMARY: &str = "AWS::MSK::Cluster";
    const OTHER: &str = "AWS::MSK::ServerlessCluster";

    #[test]
    fn singleton_target_group_is_nonempty_without_a_redundant_vector_allocation() -> TestResult {
        // Given one canonical target kind, when constructing its reviewed domain group.
        let group = ReviewedTargetKinds::singleton(PRIMARY, 0)?;
        // Then the kind is retained exactly once and no expansion storage is allocated.
        assert_eq!(group.primary(), PRIMARY);
        assert_eq!(group.len(), 1);
        assert!(!group.is_empty());
        assert_eq!(group.iter().collect::<Vec<_>>(), [PRIMARY]);
        assert_eq!(group.additional.capacity(), 0);
        Ok(())
    }

    #[test]
    fn expanded_target_groups_preserve_the_anchor_and_canonicalize_other_members() -> TestResult {
        let first = ReviewedTargetKinds::from_declared(PRIMARY, &[OTHER, PRIMARY], 0)?;
        let reversed = ReviewedTargetKinds::from_declared(PRIMARY, &[PRIMARY, OTHER], 0)?;
        assert_eq!(first, reversed);
        assert_eq!(first.primary(), PRIMARY);
        assert_eq!(first.len(), 2);
        assert_eq!(first.iter().collect::<Vec<_>>(), [PRIMARY, OTHER]);
        assert_eq!(first.additional, [OTHER]);
        Ok(())
    }

    #[test]
    fn target_groups_reject_empty_missing_anchor_duplicate_and_malformed_members() {
        for members in [&[][..], &[OTHER][..]] {
            assert!(matches!(
                ReviewedTargetKinds::from_declared(PRIMARY, members, 0),
                Err(KindMapError::InvalidRow {
                    line: 0,
                    reason: "reviewed target group must include its primary kind within bounds"
                })
            ));
        }
        assert!(
            matches!(ReviewedTargetKinds::from_declared(PRIMARY, &[PRIMARY, PRIMARY], 0),
            Err(KindMapError::Duplicate(kind)) if kind == PRIMARY)
        );
        assert!(matches!(
            ReviewedTargetKinds::from_declared(PRIMARY, &[PRIMARY, "bad kind"], 0),
            Err(KindMapError::InvalidRow {
                line: 0,
                reason: "invalid reviewed relation target kind"
            })
        ));
        assert!(matches!(
            ReviewedTargetKinds::singleton("", 0),
            Err(KindMapError::InvalidRow {
                line: 0,
                reason: "invalid relation target kind"
            })
        ));
        let owned = (0..MAX_TARGET_KINDS)
            .map(|index| format!("AWS::Test::Kind{index}"))
            .collect::<Vec<_>>();
        let oversized = std::iter::once(PRIMARY)
            .chain(owned.iter().map(String::as_str))
            .collect::<Vec<_>>();
        assert!(matches!(
            ReviewedTargetKinds::from_declared(PRIMARY, &oversized, 0),
            Err(KindMapError::InvalidRow {
                line: 0,
                reason: "reviewed target group must include its primary kind within bounds"
            })
        ));
    }

    #[test]
    fn lexical_iteration_preserves_legacy_group_order_without_copying_the_anchor() -> TestResult {
        let group = ReviewedTargetKinds::from_declared(OTHER, &[OTHER, PRIMARY], 0)?;
        assert_eq!(group.iter().collect::<Vec<_>>(), [OTHER, PRIMARY]);
        assert_eq!(group.iter_sorted().collect::<Vec<_>>(), [PRIMARY, OTHER]);
        assert_eq!(
            ReviewedTargetKinds::singleton(PRIMARY, 0)?
                .iter_sorted()
                .collect::<Vec<_>>(),
            [PRIMARY]
        );
        Ok(())
    }

    #[test]
    fn folding_expanded_members_preserves_the_owned_anchor_and_is_atomic_on_rejection() -> TestResult
    {
        let mut group = ReviewedTargetKinds::singleton(PRIMARY, 0)?;
        let anchor = group.primary().as_ptr();
        group.expand(&[OTHER, PRIMARY], 0)?;
        assert_eq!(group.primary().as_ptr(), anchor);
        assert_eq!(group.iter().collect::<Vec<_>>(), [PRIMARY, OTHER]);
        let unchanged = group.clone();
        assert!(group.expand(&[PRIMARY, PRIMARY], 0).is_err());
        assert_eq!(group, unchanged);
        assert_eq!(group.primary().as_ptr(), anchor);
        Ok(())
    }
}
