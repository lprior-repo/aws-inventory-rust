use super::*;

pub(super) fn cfn(
    entry: CfnMapping<'_>,
    exceptions: &Exceptions,
    index: usize,
) -> Result<KindEntry, KindMapError> {
    let (cfn_type, target, display_name_path) = match entry {
        CfnMapping::Mapped {
            cfn_type,
            kind,
            display_name_path,
        } => {
            if !valid_cfn_type(cfn_type)
                || !valid_kind(kind)
                || display_name_path.is_some_and(|path| !valid_path(path))
            {
                return Err(row_error(index, "invalid canonical kind or display path"));
            }
            let target = KindTarget::Mapped {
                kind: kind.to_owned(),
                table_name: table(cfn_type, exceptions)?,
            };
            (cfn_type, target, display_name_path.map(str::to_owned))
        }
        CfnMapping::Unmapped { cfn_type, reason } => {
            if !valid_cfn_type(cfn_type) || !valid_reason(reason) {
                return Err(row_error(index, "unmapped CFN type requires a reason"));
            }
            (
                cfn_type,
                KindTarget::Unmapped {
                    reason: reason.to_owned(),
                },
                None,
            )
        }
    };
    Ok(KindEntry {
        cfn_type: cfn_type.to_owned(),
        target,
        display_name_path,
    })
}

pub(super) fn api(
    entry: ApiOnlyMapping<'_>,
    exceptions: &Exceptions,
    index: usize,
) -> Result<ApiOnlyKind, KindMapError> {
    if !valid_cfn_type(entry.kind) || !valid_reason(entry.reason) {
        return Err(row_error(
            index,
            "API-only kind requires an explicit reason",
        ));
    }
    Ok(ApiOnlyKind {
        kind: entry.kind.to_owned(),
        reason: entry.reason.to_owned(),
        table_name: table(entry.kind, exceptions)?,
        display_name_path: None,
    })
}

fn table(kind: &str, exceptions: &Exceptions) -> Result<SqlName, KindMapError> {
    crate::names::sql_name(kind, exceptions).map_err(|source| KindMapError::Name {
        kind: kind.to_owned(),
        source,
    })
}

pub(super) fn operation(
    entry: OperationMapping<'_>,
    index: usize,
) -> Result<RuntimeOperation, KindMapError> {
    if !valid_service(entry.service) || !valid_operation(entry.operation) || !valid_kind(entry.kind)
    {
        return Err(row_error(index, "invalid typed runtime operation mapping"));
    }
    Ok(RuntimeOperation {
        service: entry.service.to_owned(),
        operation: entry.operation.to_owned(),
        kind: entry.kind.to_owned(),
    })
}

pub(super) fn relation(
    entry: RelationMapping<'_>,
    index: usize,
) -> Result<ReviewedRelation, KindMapError> {
    if !valid_kind(entry.source_kind)
        || !valid_kind(entry.target_kind)
        || !valid_label(entry.relation)
        || !valid_label(entry.scope)
    {
        return Err(row_error(index, "invalid reviewed scalar relation"));
    }
    let guard = entry
        .guard
        .map(|guard| {
            if !valid_reason(guard.equals) {
                return Err(row_error(
                    index,
                    "relation guard requires a bounded nonempty enum literal",
                ));
            }
            guard.owned(index)
        })
        .transpose()?;
    Ok(ReviewedRelation {
        source_kind: entry.source_kind.to_owned(),
        source: entry.source.owned(index)?,
        target_kinds: ReviewedTargetKinds::singleton(entry.target_kind, index)?,
        target: entry.target.owned(index)?,
        relation: entry.relation.to_owned(),
        scope: entry.scope.to_owned(),
        guard,
    })
}

pub(super) fn validate_canonical_kinds(
    entries: &[KindEntry],
    api_only: &[ApiOnlyKind],
) -> Result<(), KindMapError> {
    let mapped = entries.iter().filter_map(|entry| match &entry.target {
        KindTarget::Mapped { kind, .. } => Some(kind.as_str()),
        KindTarget::Unmapped { .. } => None,
    });
    let kinds = mapped.chain(api_only.iter().map(|entry| entry.kind.as_str()));
    let mut seen = BTreeSet::new();
    kinds.into_iter().try_for_each(|kind| {
        if seen.insert(kind) {
            return Ok(());
        }
        Err(KindMapError::DuplicateCanonicalKind(kind.to_owned()))
    })
}
pub(super) fn validate_collisions(
    entries: &[KindEntry],
    api_only: &[ApiOnlyKind],
) -> Result<(), KindMapError> {
    validate_canonical_kinds(entries, api_only)?;
    let mapped = entries.iter().filter_map(|entry| match &entry.target {
        KindTarget::Mapped { table_name, .. } => Some(table_name.as_str()),
        KindTarget::Unmapped { .. } => None,
    });
    let names = mapped.chain(api_only.iter().map(|entry| entry.table_name.as_str()));
    let mut seen = BTreeSet::new();
    names.into_iter().try_for_each(|name| {
        if seen.insert(name) {
            Ok(())
        } else {
            Err(KindMapError::Duplicate(name.to_owned()))
        }
    })
}
