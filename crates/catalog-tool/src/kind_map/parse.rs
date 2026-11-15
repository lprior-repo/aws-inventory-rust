use super::*;

pub(super) fn parse_rows(
    bytes: &[u8],
    rows: &[&str],
    exceptions: &Exceptions,
) -> Result<KindMap, KindMapError> {
    let mut state = ParseState::default();
    rows.iter().enumerate().try_for_each(|(index, row)| {
        if row.is_empty() {
            return Ok(());
        }
        let line = index.checked_add(1).ok_or(KindMapError::TooManyRows)?;
        parse_row(
            &row.split('\t').collect::<Vec<_>>(),
            line,
            exceptions,
            &mut state,
        )
    })?;
    state.finish(bytes)
}

#[derive(Default)]
pub(super) struct ParseState {
    pub(super) version: Option<u16>,
    pub(super) reviewer: Option<String>,
    pub(super) reviewed: Option<String>,
    pub(super) default_unmapped_reason: Option<String>,
    pub(super) entries: Vec<KindEntry>,
    pub(super) api_only: Vec<ApiOnlyKind>,
    pub(super) relations: Vec<ReviewedRelation>,
    pub(super) bootstraps: Vec<bootstrap::BootstrapEntry>,
    pub(super) operations: Vec<RuntimeOperation>,
    pub(super) operation_keys: BTreeSet<String>,
    pub(super) keys: BTreeSet<String>,
    pub(super) used: Vec<bool>,
}
impl ParseState {
    pub(super) fn finish(self, bytes: &[u8]) -> Result<KindMap, KindMapError> {
        self.validate()?;
        let missing = KindMapError::MissingMetadata;
        Ok(KindMap {
            source_version: self.version.ok_or(missing("version"))?,
            source_sha256: Sha256::digest(bytes).into(),
            reviewer: self.reviewer.ok_or(missing("reviewer"))?,
            reviewed: self.reviewed.ok_or(missing("reviewed"))?,
            default_unmapped_reason: self.default_unmapped_reason,
            entries: self.entries,
            api_only: self.api_only,
            relations: self.relations,
            bootstraps: self.bootstraps,
            operations: self.operations,
        })
    }

    fn validate(&self) -> Result<(), KindMapError> {
        validate_collisions(&self.entries, &self.api_only)?;
        validate_operations(
            &self.entries,
            &self.api_only,
            &self.operations,
            &self.relations,
            &self.bootstraps,
        )
    }
}

pub(super) fn parse_row(
    cells: &[&str],
    line: usize,
    exceptions: &Exceptions,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    match cells.first().copied() {
        Some("meta") => parse_meta(cells, line, state),
        Some("cfn") => parse_cfn(cells, line, exceptions, state),
        Some("api") => parse_api(cells, line, exceptions, state),
        Some("relation") => parse_relation(cells, line, &mut state.relations),
        Some("default-unmapped") => parse_default(cells, line, state),
        Some("bootstrap") => parse_bootstrap(cells, line, state),
        Some("operation") => parse_operation(cells, line, state),
        _ => Err(row_error(line, "unknown row type")),
    }
}

pub(super) fn parse_default(
    cells: &[&str],
    line: usize,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let [_, reason] = cells else {
        return Err(row_error(line, "invalid default unmapped reason"));
    };
    if state.default_unmapped_reason.is_some() || !valid_reason(reason) {
        return Err(row_error(line, "invalid default unmapped reason"));
    }
    state.default_unmapped_reason = Some((*reason).to_owned());
    Ok(())
}

pub(super) fn parse_bootstrap(
    cells: &[&str],
    line: usize,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let entry = bootstrap::parse(cells, line)?;
    if state
        .bootstraps
        .iter()
        .any(|item| item.service == entry.service && item.operation == entry.operation)
    {
        return Err(KindMapError::Duplicate(format!(
            "{}.{}",
            entry.service, entry.operation
        )));
    }
    state.bootstraps.push(entry);
    Ok(())
}

pub(super) fn parse_operation(
    cells: &[&str],
    line: usize,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let [_, service, operation, kind] = cells else {
        return Err(row_error(line, "invalid runtime operation mapping"));
    };
    if !valid_service(service) || !valid_operation(operation) || !valid_kind(kind) {
        return Err(row_error(line, "invalid runtime operation mapping"));
    }
    let key = format!("{service}.{operation}");
    if !state.operation_keys.insert(key.clone()) {
        return Err(KindMapError::Duplicate(key));
    }
    state.operations.push(RuntimeOperation {
        service: (*service).to_owned(),
        operation: (*operation).to_owned(),
        kind: (*kind).to_owned(),
    });
    Ok(())
}
