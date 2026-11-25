use super::bootstrap::BootstrapEntry;
use super::validation::{
    row_error, valid_date, valid_operation, valid_path, valid_reason, valid_service,
};
use super::{
    ApiOnlyKind, BootstrapMapping, BootstrapPurpose, KindEntry, KindMap, KindMapError, MAX_ROWS,
    MappingSource, ReviewedRelation, RuntimeOperation, rows, validate_operations,
};
use crate::names::Exceptions;
use std::collections::BTreeSet;

struct ParsedRows {
    entries: Vec<KindEntry>,
    api_only: Vec<ApiOnlyKind>,
    operations: Vec<RuntimeOperation>,
    relations: Vec<ReviewedRelation>,
    bootstraps: Vec<BootstrapEntry>,
}

pub(super) fn compile(
    source: &MappingSource<'_>,
    exceptions: &Exceptions,
) -> Result<KindMap, KindMapError> {
    validate_metadata(source)?;
    let parsed = parse_rows(source, exceptions)?;
    validate_keys(&parsed)?;
    build_kind_map(source, parsed)
}

fn parse_rows(
    source: &MappingSource<'_>,
    exceptions: &Exceptions,
) -> Result<ParsedRows, KindMapError> {
    let entries = source
        .cfn
        .iter()
        .enumerate()
        .map(|(index, entry)| rows::cfn(*entry, exceptions, index))
        .collect::<Result<Vec<_>, _>>()?;
    let api_only = source
        .api_only
        .iter()
        .enumerate()
        .map(|(index, entry)| rows::api(*entry, exceptions, index))
        .collect::<Result<Vec<_>, _>>()?;
    let operations = source
        .operations
        .iter()
        .enumerate()
        .map(|(index, entry)| rows::operation(*entry, index))
        .collect::<Result<Vec<_>, _>>()?;
    let relations = source
        .relations
        .iter()
        .enumerate()
        .map(|(index, entry)| rows::relation(*entry, index))
        .collect::<Result<Vec<_>, KindMapError>>()?;
    let bootstraps = source
        .bootstraps
        .iter()
        .copied()
        .map(BootstrapMapping::owned)
        .collect::<Vec<_>>();
    Ok(ParsedRows {
        entries,
        api_only,
        operations,
        relations,
        bootstraps,
    })
}

fn validate_keys(parsed: &ParsedRows) -> Result<(), KindMapError> {
    validate_bootstrap_fields(&parsed.bootstraps)?;
    rows::validate_collisions(&parsed.entries, &parsed.api_only)?;
    validate_duplicates(parsed.entries.iter().map(|entry| entry.cfn_type.as_str()))?;
    validate_operation_keys(&parsed.operations)?;
    validate_bootstrap_keys(&parsed.bootstraps)?;
    validate_relation_keys(&parsed.relations)
}

fn build_kind_map(source: &MappingSource<'_>, parsed: ParsedRows) -> Result<KindMap, KindMapError> {
    let known = super::operations::known_kinds(&parsed.entries, &parsed.api_only);
    validate_operations(
        &known,
        &parsed.api_only,
        &parsed.operations,
        &parsed.relations,
        &parsed.bootstraps,
    )?;
    let mut relations = parsed.relations;
    super::relation_targets::fold(source.relation_targets, &known, &mut relations)?;
    let parent_relations = super::relation_modes::compile_parents(source.parent_relations, &known)?;
    let compound_relations =
        super::relation_modes::compile_compounds(source.compound_relations, &known)?;
    let additional_emissions =
        super::emissions::compile(source.additional_emissions, &parsed.operations, &known)?;
    let mut map = KindMap {
        source_version: source.version,
        source_sha256: [0; 32],
        reviewer: source.reviewer.to_owned(),
        reviewed: source.reviewed.to_owned(),
        default_unmapped_reason: source.default_unmapped_reason.map(str::to_owned),
        entries: parsed.entries,
        api_only: parsed.api_only,
        relations,
        parent_relations,
        compound_relations,
        additional_emissions,
        bootstraps: parsed.bootstraps,
        operations: parsed.operations,
    };
    sort(&mut map);
    map.source_sha256 = super::fingerprint::digest(&map);
    Ok(map)
}

fn validate_metadata(source: &MappingSource<'_>) -> Result<(), KindMapError> {
    let rows = [
        source.cfn.len(),
        source.api_only.len(),
        source.operations.len(),
        source.relations.len(),
        source.parent_relations.len(),
        source.compound_relations.len(),
        source.additional_emissions.len(),
        source.relation_targets.len(),
        source.bootstraps.len(),
    ]
    .into_iter()
    .try_fold(0_usize, |total, count| {
        total.checked_add(count).ok_or(KindMapError::TooManyRows)
    })?;
    if rows > MAX_ROWS {
        return Err(KindMapError::TooManyRows);
    }
    if source.version != 1
        || !valid_reason(source.reviewer)
        || source.reviewer.len() > 256
        || !valid_date(source.reviewed)
    {
        return Err(row_error(0, "invalid typed review metadata"));
    }
    if source
        .default_unmapped_reason
        .is_some_and(|reason| !valid_reason(reason))
    {
        return Err(row_error(0, "invalid default unmapped reason"));
    }
    Ok(())
}

fn validate_duplicates<'a>(keys: impl Iterator<Item = &'a str>) -> Result<(), KindMapError> {
    let mut seen = BTreeSet::new();
    keys.into_iter().try_for_each(|key| {
        if seen.insert(key) {
            Ok(())
        } else {
            Err(KindMapError::Duplicate(key.to_owned()))
        }
    })
}

fn validate_operation_keys(entries: &[RuntimeOperation]) -> Result<(), KindMapError> {
    let mut seen = BTreeSet::new();
    entries.iter().try_for_each(|entry| {
        if seen.insert((&entry.service, &entry.operation)) {
            return Ok(());
        }
        Err(KindMapError::Duplicate(format!(
            "{}.{}",
            entry.service, entry.operation
        )))
    })
}

fn validate_bootstrap_keys(entries: &[BootstrapEntry]) -> Result<(), KindMapError> {
    let mut seen = BTreeSet::new();
    entries.iter().try_for_each(|entry| {
        if seen.insert((&entry.service, &entry.operation)) {
            return Ok(());
        }
        Err(KindMapError::Duplicate(format!(
            "{}.{}",
            entry.service, entry.operation
        )))
    })
}

fn validate_relation_keys(entries: &[ReviewedRelation]) -> Result<(), KindMapError> {
    let mut seen = BTreeSet::new();
    entries.iter().try_for_each(|entry| {
        let key = (
            &entry.source_kind,
            &entry.source,
            &entry.target_kinds,
            &entry.target,
            &entry.relation,
            &entry.scope,
            &entry.guard,
        );
        if seen.insert(key) {
            return Ok(());
        }
        Err(KindMapError::Duplicate(entry.relation.clone()))
    })
}

fn validate_bootstrap_fields(entries: &[BootstrapEntry]) -> Result<(), KindMapError> {
    entries.iter().try_for_each(|entry| {
        let valid = match &entry.purpose {
            BootstrapPurpose::AccountIdentity { account_path } => valid_path(account_path),
            BootstrapPurpose::RegionDiscovery {
                regions_path,
                region_name_path,
                opt_in_status_path,
            } => {
                valid_path(regions_path)
                    && valid_path(region_name_path)
                    && opt_in_status_path.as_deref().is_none_or(valid_path)
            }
            BootstrapPurpose::Enumeration => false,
        };
        if valid_service(&entry.service) && valid_operation(&entry.operation) && valid {
            return Ok(());
        }
        Err(row_error(0, "invalid typed bootstrap mapping"))
    })
}

fn sort(map: &mut KindMap) {
    map.entries.sort_by(|a, b| a.cfn_type.cmp(&b.cfn_type));
    map.api_only.sort_by(|a, b| a.kind.cmp(&b.kind));
    map.operations
        .sort_by(|a, b| (&a.service, &a.operation).cmp(&(&b.service, &b.operation)));
    map.bootstraps
        .sort_by(|a, b| (&a.service, &a.operation).cmp(&(&b.service, &b.operation)));
    map.relations.sort_by(|a, b| {
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
