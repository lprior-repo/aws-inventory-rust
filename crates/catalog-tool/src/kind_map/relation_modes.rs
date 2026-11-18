use super::KindMapError;
use super::relation_selectors::{
    RelationSource, RelationTarget, ReviewedRelationSource, ReviewedRelationTarget,
};
use super::target_kinds::ReviewedTargetKinds;
use super::validation::{row_error, valid_kind, valid_label};
use std::collections::BTreeSet;

const MAX_COMPONENTS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentRelationMapping<'a> {
    pub source_kind: &'a str,
    pub target_kinds: &'a [&'a str],
    pub relation: &'a str,
    pub scope: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationComponentMapping<'a> {
    pub source: RelationSource<'a>,
    pub target: RelationTarget<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompoundRelationMapping<'a> {
    pub source_kind: &'a str,
    pub target_kind: &'a str,
    pub relation: &'a str,
    pub scope: &'a str,
    pub components: &'a [RelationComponentMapping<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewedParentRelation {
    pub source_kind: String,
    pub target_kinds: ReviewedTargetKinds,
    pub relation: String,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewedRelationComponent {
    pub source: ReviewedRelationSource,
    pub target: ReviewedRelationTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewedCompoundRelation {
    pub source_kind: String,
    pub target_kinds: ReviewedTargetKinds,
    pub relation: String,
    pub scope: String,
    pub components: Vec<ReviewedRelationComponent>,
}

pub(super) fn compile_parents(
    source: &[ParentRelationMapping<'_>],
    known: &BTreeSet<&str>,
) -> Result<Vec<ReviewedParentRelation>, KindMapError> {
    let mut rows = source
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let target_kinds = ReviewedTargetKinds::from_members(row.target_kinds, index)?;
            validate_ends(
                row.source_kind,
                target_kinds.primary(),
                row.relation,
                row.scope,
                known,
                index,
            )?;
            for target in target_kinds.iter().skip(1) {
                if !known.contains(target) {
                    return Err(KindMapError::UnknownRelationKind {
                        relation: row.relation.to_owned(),
                        kind: target.to_owned(),
                    });
                }
            }
            Ok(ReviewedParentRelation {
                source_kind: row.source_kind.to_owned(),
                target_kinds,
                relation: row.relation.to_owned(),
                scope: row.scope.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, KindMapError>>()?;
    rows.sort();
    if rows
        .iter()
        .zip(rows.iter().skip(1))
        .any(|(left, right)| left == right)
    {
        return Err(KindMapError::Duplicate(
            "reviewed parent relation".to_owned(),
        ));
    }
    Ok(rows)
}

pub(super) fn compile_compounds(
    source: &[CompoundRelationMapping<'_>],
    known: &BTreeSet<&str>,
) -> Result<Vec<ReviewedCompoundRelation>, KindMapError> {
    let mut rows = source
        .iter()
        .enumerate()
        .map(|(index, row)| {
            validate_ends(
                row.source_kind,
                row.target_kind,
                row.relation,
                row.scope,
                known,
                index,
            )?;
            if row.components.len() < 2 || row.components.len() > MAX_COMPONENTS {
                return Err(row_error(
                    index,
                    "compound relation requires two through eight AND components",
                ));
            }
            let mut components = row
                .components
                .iter()
                .map(|component| {
                    Ok(ReviewedRelationComponent {
                        source: component.source.owned(index)?,
                        target: component.target.owned(index)?,
                    })
                })
                .collect::<Result<Vec<_>, KindMapError>>()?;
            for (position, component) in components.iter().enumerate() {
                if components
                    .iter()
                    .take(position)
                    .any(|previous| previous.target == component.target)
                {
                    return Err(row_error(
                        index,
                        "compound relation repeats a target selector",
                    ));
                }
            }
            components.sort();
            Ok(ReviewedCompoundRelation {
                source_kind: row.source_kind.to_owned(),
                target_kinds: ReviewedTargetKinds::singleton(row.target_kind, index)?,
                relation: row.relation.to_owned(),
                scope: row.scope.to_owned(),
                components,
            })
        })
        .collect::<Result<Vec<_>, KindMapError>>()?;
    rows.sort();
    if rows
        .iter()
        .zip(rows.iter().skip(1))
        .any(|(left, right)| left == right)
    {
        return Err(KindMapError::Duplicate(
            "reviewed compound relation".to_owned(),
        ));
    }
    Ok(rows)
}

fn validate_ends(
    source: &str,
    target: &str,
    relation: &str,
    scope: &str,
    known: &BTreeSet<&str>,
    index: usize,
) -> Result<(), KindMapError> {
    if !valid_kind(source) || !valid_kind(target) || !valid_label(relation) || !valid_label(scope) {
        return Err(row_error(
            index,
            "invalid reviewed relation kind, label, or scope",
        ));
    }
    for kind in [source, target] {
        if !known.contains(kind) {
            return Err(KindMapError::UnknownRelationKind {
                relation: relation.to_owned(),
                kind: kind.to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::relation_selectors::RelationPath;
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const CHILD: &str = "AWS::Route53::KeySigningKey";
    const PARENT: &str = "AWS::Route53::HostedZone";
    const PARENT_ROW: ParentRelationMapping<'static> = ParentRelationMapping {
        source_kind: CHILD,
        target_kinds: &[PARENT],
        relation: "FOR_HOSTED_ZONE",
        scope: "account",
    };
    const COMPONENTS: &[RelationComponentMapping<'static>] = &[
        RelationComponentMapping {
            source: RelationSource::Request(RelationPath::Fields("farmId")),
            target: RelationTarget::BoundContext(RelationPath::Fields("farmId")),
        },
        RelationComponentMapping {
            source: RelationSource::SelectedOutput(RelationPath::Fields("queueId")),
            target: RelationTarget::SelectedOutput(RelationPath::Fields("queueId")),
        },
    ];

    #[test]
    fn leased_parent_metadata_contains_only_the_exact_reviewed_kinds_and_scope() -> TestResult {
        let known = [CHILD, PARENT].into_iter().collect();
        let rows = compile_parents(&[PARENT_ROW], &known)?;
        assert_eq!(
            rows,
            [ReviewedParentRelation {
                source_kind: CHILD.to_owned(),
                target_kinds: ReviewedTargetKinds::singleton(PARENT, 0)?,
                relation: "FOR_HOSTED_ZONE".to_owned(),
                scope: "account".to_owned(),
            }]
        );
        assert!(compile_parents(&[PARENT_ROW, PARENT_ROW], &known).is_err());
        assert!(
            compile_parents(
                &[ParentRelationMapping {
                    target_kinds: &["AWS::Missing::Kind"],
                    ..PARENT_ROW
                }],
                &known
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn compound_metadata_preserves_independent_context_and_selected_terms() -> TestResult {
        // Given a parent identifier available only from bound input, and a selected queue identifier.
        let known = [CHILD, PARENT].into_iter().collect();
        let row = compound_row(COMPONENTS);
        let compiled = compile_compounds(&[row], &known)?;
        let [first, second] = COMPONENTS else {
            return Err("fixture components".into());
        };
        let reordered = [*second, *first];
        assert_eq!(
            compiled,
            compile_compounds(
                &[CompoundRelationMapping {
                    components: &reordered,
                    ..row
                }],
                &known
            )?
        );
        let result = compiled.first().ok_or("missing reviewed compound")?;
        assert_eq!(result.components.len(), 2);
        assert_eq!(result.target_kinds.primary(), PARENT);
        assert!(
            result
                .components
                .iter()
                .any(|term| matches!(term.target, ReviewedRelationTarget::BoundContext(_)))
        );
        Ok(())
    }

    fn compound_row<'a>(
        components: &'a [RelationComponentMapping<'a>],
    ) -> CompoundRelationMapping<'a> {
        CompoundRelationMapping {
            source_kind: CHILD,
            target_kind: PARENT,
            relation: "FOR_QUEUE",
            scope: "account_region",
            components,
        }
    }

    #[test]
    fn compound_bounds_reject_zero_one_and_nine_distinct_components_but_accept_eight() -> TestResult
    {
        let known = [CHILD, PARENT].into_iter().collect();
        let components =
            ["a", "b", "c", "d", "e", "f", "g", "h", "i"].map(|path| RelationComponentMapping {
                source: RelationSource::SelectedOutput(RelationPath::Fields(path)),
                target: RelationTarget::SelectedOutput(RelationPath::Fields(path)),
            });
        let one = components.get(..1).ok_or("fixture bounds")?;
        let eight = components.get(..8).ok_or("fixture bounds")?;
        let accepted = compile_compounds(&[compound_row(eight)], &known)?;
        assert_eq!(
            accepted
                .first()
                .ok_or("missing upper-bound compound")?
                .components
                .len(),
            8
        );
        for invalid in [&[][..], one, &components[..]] {
            assert!(matches!(
                compile_compounds(&[compound_row(invalid)], &known),
                Err(KindMapError::InvalidRow {
                    line: 0,
                    reason: "compound relation requires two through eight AND components"
                })
            ));
        }
        Ok(())
    }

    #[test]
    fn compound_target_selectors_must_be_unique_even_when_the_sources_differ() -> TestResult {
        let known = [CHILD, PARENT].into_iter().collect();
        let first = COMPONENTS.first().copied().ok_or("fixture component")?;
        let other_source = RelationComponentMapping {
            source: RelationSource::SelectedOutput(RelationPath::Fields("otherId")),
            ..first
        };
        for duplicate_targets in [[first, first], [first, other_source]] {
            assert!(matches!(
                compile_compounds(&[compound_row(&duplicate_targets)], &known),
                Err(KindMapError::InvalidRow {
                    line: 0,
                    reason: "compound relation repeats a target selector"
                })
            ));
        }
        Ok(())
    }

    #[test]
    fn reviewed_parent_groups_are_known_bounded_unique_and_permutation_canonical() -> TestResult {
        const OTHER_PARENT: &str = "AWS::Route53::HealthCheck";
        let known = [CHILD, PARENT, OTHER_PARENT].into_iter().collect();
        let group = ParentRelationMapping {
            target_kinds: &[PARENT, OTHER_PARENT],
            ..PARENT_ROW
        };
        let rows = compile_parents(&[group], &known)?;
        let reordered = ParentRelationMapping {
            target_kinds: &[OTHER_PARENT, PARENT],
            ..PARENT_ROW
        };
        assert_eq!(rows, compile_parents(&[reordered], &known)?);
        let row = rows.first().ok_or("missing reviewed parent group")?;
        assert_eq!(row.target_kinds.primary(), OTHER_PARENT);
        assert_eq!(
            row.target_kinds.iter().collect::<Vec<_>>(),
            [OTHER_PARENT, PARENT]
        );
        assert!(compile_parents(&[group, reordered], &known).is_err());
        for members in [
            &[][..],
            &[PARENT, PARENT][..],
            &[PARENT, "AWS::ZZZ::Missing"][..],
            &["bad kind"][..],
        ] {
            assert!(
                compile_parents(
                    &[ParentRelationMapping {
                        target_kinds: members,
                        ..PARENT_ROW
                    }],
                    &known
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn maximum_reviewed_parent_group_retains_every_known_member() -> TestResult {
        let owned = (0..16)
            .map(|index| format!("AWS::ApiOnly::ParentKind{index:02}"))
            .collect::<Vec<_>>();
        let members = owned.iter().map(String::as_str).collect::<Vec<_>>();
        let known = members.iter().copied().chain([CHILD]).collect();
        let rows = compile_parents(
            &[ParentRelationMapping {
                target_kinds: &members,
                ..PARENT_ROW
            }],
            &known,
        )?;
        let row = rows
            .first()
            .ok_or("missing maximum reviewed parent group")?;
        assert_eq!(row.target_kinds.len(), 16);
        assert_eq!(row.target_kinds.iter().collect::<Vec<_>>(), members);
        Ok(())
    }

    #[test]
    fn oversized_reviewed_parent_group_rejects_distinct_known_members() {
        let owned = (0..17)
            .map(|index| format!("AWS::ApiOnly::ParentKind{index:02}"))
            .collect::<Vec<_>>();
        let members = owned.iter().map(String::as_str).collect::<Vec<_>>();
        let known = members.iter().copied().chain([CHILD]).collect();
        let result = compile_parents(
            &[ParentRelationMapping {
                target_kinds: &members,
                ..PARENT_ROW
            }],
            &known,
        );
        assert!(matches!(
            result,
            Err(KindMapError::InvalidRow {
                line: 0,
                reason: "reviewed target group exceeds sixteen kinds"
            })
        ));
    }
}
