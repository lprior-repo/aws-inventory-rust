use super::{
    KindMap, KindMapError, RelationMapping, RelationPath, RelationSource, RelationTarget,
    fingerprint, operations, rows, validation::row_error,
};
use crate::smithy::fields::publish::read_bytes;
use std::{collections::BTreeSet, path::Path};

pub(super) fn load(root: &Path, map: KindMap) -> Result<KindMap, KindMapError> {
    let bytes = read_bytes(&root.join(crate::relations::ROWS_PATH), 2_097_152)?
        .ok_or(KindMapError::MissingGeneratedRelations)?;
    map.with_generated_relations(&bytes)
}

impl KindMap {
    pub fn with_generated_relations(mut self, bytes: &[u8]) -> Result<Self, KindMapError> {
        if bytes.len() > super::MAX_BYTES {
            return Err(KindMapError::TooLarge);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| row_error(0, "invalid UTF-8"))?;
        let hand = self
            .relations
            .iter()
            .map(|row| (row.source_kind.clone(), row.source.clone()))
            .collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::new();
        text.lines().enumerate().try_for_each(|(index, line)| {
            if index >= super::MAX_ROWS {
                return Err(KindMapError::TooManyRows);
            }
            if line.is_empty() {
                return Ok(());
            }
            let Some(row) = parse(line, index)? else {
                return Ok(());
            };
            if hand.contains(&(row.source_kind.clone(), row.source.clone())) {
                return Ok(());
            }
            if !seen.insert((
                row.source_kind.clone(),
                row.source.clone(),
                row.target_kinds.clone(),
                row.target.clone(),
                row.relation.clone(),
                row.scope.clone(),
            )) {
                return Err(row_error(index, "duplicate generated relation"));
            }
            if self.relations.len() >= super::MAX_ROWS {
                return Err(KindMapError::TooManyRows);
            }
            self.relations.push(row);
            Ok(())
        })?;
        let known = operations::known_kinds(&self.entries, &self.api_only);
        operations::validate(
            &known,
            &self.api_only,
            &self.operations,
            &self.relations,
            &self.bootstraps,
        )?;
        sort_relations(&mut self.relations);
        self.source_sha256 = fingerprint::digest(&self);
        Ok(self)
    }
}

fn sort_relations(relations: &mut [super::ReviewedRelation]) {
    relations.sort_by(|a, b| {
        (
            &a.source_kind,
            &a.source,
            a.target_kinds.primary(),
            &a.target,
            &a.relation,
            &a.scope,
            &a.guard,
        )
            .cmp(&(
                &b.source_kind,
                &b.source,
                b.target_kinds.primary(),
                &b.target,
                &b.relation,
                &b.scope,
                &b.guard,
            ))
    });
}

fn parse(line: &str, index: usize) -> Result<Option<super::ReviewedRelation>, KindMapError> {
    let mut cells = line.split('\t');
    let source = (cells.next(), cells.next(), cells.next());
    let target = (cells.next(), cells.next());
    let labels = (cells.next(), cells.next(), cells.next());
    match (source, target, labels) {
        (
            (Some(tag @ ("relation" | "relation-held")), Some(kind), Some(path)),
            (Some(target), Some(target_path)),
            (Some(relation), Some(scope), None),
        ) => {
            let row = rows::relation(
                RelationMapping {
                    source_kind: kind,
                    source: RelationSource::SelectedOutput(RelationPath::Fields(path)),
                    target_kind: target,
                    target: RelationTarget::SelectedOutput(RelationPath::Fields(target_path)),
                    relation,
                    scope,
                    guard: None,
                },
                index,
            )?;
            Ok((tag == "relation").then_some(row))
        }
        _ => Err(row_error(
            index,
            "generated relation requires seven tab-separated columns",
        )),
    }
}

#[cfg(test)]
mod tests;
