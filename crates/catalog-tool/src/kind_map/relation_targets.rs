use super::validation::{row_error, valid_kind, valid_label, valid_path};
use super::{KindMapError, RelationPath, RelationSource, RelationTargetsMapping, ReviewedRelation};
use std::collections::BTreeSet;

pub(super) fn fold(
    source: &[RelationTargetsMapping<'_>],
    known: &BTreeSet<&str>,
    relations: &mut [ReviewedRelation],
) -> Result<(), KindMapError> {
    let mut keys = BTreeSet::new();
    for (index, row) in source.iter().enumerate() {
        let path_valid = match row.source {
            RelationSource::SelectedOutput(RelationPath::Fields(path))
            | RelationSource::Request(RelationPath::Fields(path)) => valid_path(path),
            RelationSource::SelectedOutput(RelationPath::RootText)
            | RelationSource::Request(RelationPath::RootText) => true,
        };
        if !valid_kind(row.source_kind)
            || !path_valid
            || !valid_label(row.relation)
            || !(2..=16).contains(&row.target_kinds.len())
        {
            return Err(row_error(index, "invalid reviewed reference target group"));
        }
        if !keys.insert((row.source_kind, row.source, row.relation)) {
            return Err(KindMapError::Duplicate(row.relation.to_owned()));
        }
        let mut matching = relations.iter_mut().filter(|relation| {
            relation.source_kind == row.source_kind
                && row.source.matches(&relation.source)
                && relation.relation == row.relation
        });
        let relation = matching
            .next()
            .ok_or_else(|| row_error(index, "reference target group has no relation"))?;
        if matching.next().is_some() || relation.guard.is_some() {
            return Err(row_error(
                index,
                "reference target group requires one unguarded relation",
            ));
        }
        for target in row.target_kinds {
            if !known.contains(target) {
                return Err(KindMapError::UnknownRelationKind {
                    relation: row.relation.to_owned(),
                    kind: (*target).to_owned(),
                });
            }
        }
        relation.target_kinds.expand(row.target_kinds, index)?;
    }
    Ok(())
}
