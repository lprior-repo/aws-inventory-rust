use super::{
    AdditionalEmissionMapping, ApiOnlyMapping, CfnMapping, CompoundRelationMapping, KindMap,
    KindMapError, MappingSource, OperationMapping, ParentRelationMapping, RelationMapping,
    RelationTargetsMapping,
};
use crate::names::Exceptions;
use std::collections::BTreeMap;

pub(super) fn compile<'a>(
    sources: impl Iterator<Item = MappingSource<'a>> + Clone,
    cfn_source: &[u8],
) -> Result<KindMap, KindMapError> {
    let mut map = reviewed(sources)?;
    super::complete_cfn(&mut map, cfn_source)?;
    map.source_sha256 = super::fingerprint::digest(&map);
    Ok(map)
}

pub(super) fn reviewed<'a>(
    sources: impl Iterator<Item = MappingSource<'a>> + Clone,
) -> Result<KindMap, KindMapError> {
    let cfn = unique(
        sources
            .clone()
            .flat_map(|source| source.cfn.iter().copied()),
        cfn_key,
    )?;
    let api_only = collect_api_only(sources.clone())?;
    let operations = collect_operations(sources.clone())?;
    let relation_targets = collect_relation_targets(sources.clone())?;
    let relations = collect_relations(sources.clone())?;
    let parent_relations = collect_parent_relations(sources.clone())?;
    let compound_relations = collect_compound_relations(sources.clone())?;
    let additional_emissions = collect_additional_emissions(sources)?;
    let source = MappingSource {
        cfn: &cfn,
        api_only: &api_only,
        operations: &operations,
        relations: &relations,
        relation_targets: &relation_targets,
        parent_relations: &parent_relations,
        compound_relations: &compound_relations,
        additional_emissions: &additional_emissions,
        reviewer: "Automated pinned-model review; embedded native service families; no human approval",
        reviewed: "2026-11-15",
        ..super::builtin::SOURCE
    };
    source.compile(&Exceptions::builtin())
}

fn collect_api_only<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<ApiOnlyMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.api_only.iter().copied()),
        |row: &ApiOnlyMapping<'_>| row.kind,
    )
}

fn collect_operations<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<OperationMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.operations.iter().copied()),
        |row: &OperationMapping<'_>| (row.service, row.operation),
    )
}

fn collect_relation_targets<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<RelationTargetsMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.relation_targets.iter().copied()),
        |row: &RelationTargetsMapping<'_>| (row.source_kind, row.source, row.relation),
    )
}

fn collect_relations<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<RelationMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.relations.iter().copied()),
        |row: &RelationMapping<'_>| {
            (
                row.source_kind,
                row.source,
                row.target_kind,
                row.target,
                row.relation,
                row.scope,
                row.guard.map(|guard| (guard.source, guard.equals)),
            )
        },
    )
}

fn collect_parent_relations<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<ParentRelationMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.parent_relations.iter().copied()),
        |row: &ParentRelationMapping<'_>| {
            (row.source_kind, row.target_kinds, row.relation, row.scope)
        },
    )
}

fn collect_compound_relations<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<CompoundRelationMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.compound_relations.iter().copied()),
        |row: &CompoundRelationMapping<'_>| {
            (row.source_kind, row.target_kind, row.relation, row.scope)
        },
    )
}

fn collect_additional_emissions<'a>(
    sources: impl Iterator<Item = MappingSource<'a>>,
) -> Result<Vec<AdditionalEmissionMapping<'a>>, KindMapError> {
    unique(
        sources.flat_map(|source| source.additional_emissions.iter().copied()),
        |row: &AdditionalEmissionMapping<'_>| (row.service, row.operation, row.emitter_key),
    )
}

fn cfn_key<'a>(row: &CfnMapping<'a>) -> &'a str {
    match row {
        CfnMapping::Mapped { cfn_type, .. } | CfnMapping::Unmapped { cfn_type, .. } => cfn_type,
    }
}

fn unique<T: Copy + PartialEq, K: Ord + std::fmt::Debug>(
    rows: impl Iterator<Item = T>,
    key: impl Fn(&T) -> K,
) -> Result<Vec<T>, KindMapError> {
    let mut entries = BTreeMap::new();
    rows.into_iter().try_for_each(|row| {
        let key = key(&row);
        if let Some(previous) = entries.get(&key) {
            return if *previous == row {
                Ok(())
            } else {
                Err(KindMapError::Duplicate(format!("{key:?}")))
            };
        }
        entries.insert(key, row);
        Ok(())
    })?;
    Ok(entries.into_values().collect())
}
