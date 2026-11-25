use super::{
    ApiOnlyMapping, CfnMapping, KindMap, KindMapError, OperationMapping, fingerprint, operations,
    rows, validation::row_error,
};
use crate::{names::Exceptions, smithy::fields::publish::read_bytes};
use std::{collections::BTreeSet, path::Path};

pub fn load(root: &Path, cfn: &[u8]) -> Result<KindMap, KindMapError> {
    super::generated_relations::load(root, load_base(root, cfn)?)
}

pub(crate) fn load_base(root: &Path, cfn: &[u8]) -> Result<KindMap, KindMapError> {
    let path = root.join("catalog/kinds.generated.tsv");
    let bytes = read_bytes(&path, 2_097_152)?;
    load_with_cfn(cfn, bytes.as_deref().map_or(&[], |bytes| bytes))
}

pub fn load_with_cfn(cfn: &[u8], generated: &[u8]) -> Result<KindMap, KindMapError> {
    let mut map = super::builtin::reviewed()?.with_generated(generated, &Exceptions::builtin())?;
    super::complete_cfn(&mut map, cfn)?;
    map.source_sha256 = fingerprint::digest(&map);
    Ok(map)
}

impl KindMap {
    pub fn with_generated(
        mut self,
        bytes: &[u8],
        exceptions: &Exceptions,
    ) -> Result<Self, KindMapError> {
        if bytes.len() > super::MAX_BYTES {
            return Err(KindMapError::TooLarge);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| row_error(0, "invalid UTF-8"))?;
        let mut seen = hand_keys(&self);
        for (index, line) in text.lines().enumerate() {
            if index >= super::MAX_ROWS {
                return Err(KindMapError::TooManyRows);
            }
            if !line.is_empty() {
                let row = parse(line, exceptions, index)?;
                apply(&mut self, row, &mut seen)?;
            }
        }
        finish(&mut self)?;
        Ok(self)
    }
}

fn hand_keys(map: &KindMap) -> Keys {
    Keys {
        hand: map
            .entries
            .iter()
            .map(|row| Key::Cfn(row.cfn_type.clone()))
            .chain(map.api_only.iter().map(|row| Key::Api(row.kind.clone())))
            .chain(
                map.operations
                    .iter()
                    .map(|row| Key::Operation(row.service.clone(), row.operation.clone())),
            )
            .collect(),
        generated: BTreeSet::new(),
    }
}

struct Keys {
    hand: BTreeSet<Key>,
    generated: BTreeSet<Key>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Cfn(String),
    Api(String),
    Operation(String, String),
}

enum Row {
    Cfn(super::KindEntry),
    Api(super::ApiOnlyKind),
    Operation(super::RuntimeOperation),
}

impl Row {
    fn key(&self) -> Key {
        match self {
            Self::Cfn(row) => Key::Cfn(row.cfn_type.clone()),
            Self::Api(row) => Key::Api(row.kind.clone()),
            Self::Operation(row) => Key::Operation(row.service.clone(), row.operation.clone()),
        }
    }
}

fn parse(line: &str, exceptions: &Exceptions, index: usize) -> Result<Row, KindMapError> {
    let mut cells = line.split('\t');
    let head = (cells.next(), cells.next(), cells.next());
    let tail = (cells.next(), cells.next(), cells.next());
    match (head, tail) {
        ((Some("cfn"), Some(cfn), Some("map")), (Some(kind), Some(display), None)) => {
            parse_cfn(cfn, kind, display, exceptions, index)
        }
        ((Some("api"), Some(kind), Some("kind")), (Some(reason), display, None)) => {
            parse_api(kind, reason, display, exceptions, index)
        }
        ((Some("operation"), Some(service), Some(operation)), (Some(kind), None, None)) => {
            parse_operation(service, operation, kind, index)
        }
        _ => Err(row_error(
            index,
            "generated rows must be cfn, api, or operation",
        )),
    }
}

fn parse_cfn(
    cfn_type: &str,
    kind: &str,
    display: &str,
    exceptions: &Exceptions,
    index: usize,
) -> Result<Row, KindMapError> {
    rows::cfn(
        CfnMapping::Mapped {
            cfn_type,
            kind,
            display_name_path: display_path(display, index)?,
        },
        exceptions,
        index,
    )
    .map(Row::Cfn)
}

fn parse_api(
    kind: &str,
    reason: &str,
    display: Option<&str>,
    exceptions: &Exceptions,
    index: usize,
) -> Result<Row, KindMapError> {
    let mut row = rows::api(ApiOnlyMapping { kind, reason }, exceptions, index)?;
    row.display_name_path = display
        .map(|path| display_path(path, index))
        .transpose()?
        .flatten()
        .map(str::to_owned);
    Ok(Row::Api(row))
}

fn parse_operation(
    service: &str,
    operation: &str,
    kind: &str,
    index: usize,
) -> Result<Row, KindMapError> {
    rows::operation(
        OperationMapping {
            service,
            operation,
            kind,
        },
        index,
    )
    .map(Row::Operation)
}

fn display_path(path: &str, index: usize) -> Result<Option<&str>, KindMapError> {
    if path == "-" {
        return Ok(None);
    }
    if !super::validation::valid_path(path) {
        return Err(row_error(index, "invalid generated display path"));
    }
    Ok(Some(path))
}

fn apply(map: &mut KindMap, row: Row, keys: &mut Keys) -> Result<(), KindMapError> {
    let key = row.key();
    if keys.hand.contains(&key) {
        return Ok(());
    }
    if !keys.generated.insert(key) {
        return Err(row_error(0, "duplicate generated key"));
    }
    match row {
        Row::Cfn(row) => map.entries.push(row),
        Row::Api(row) => map.api_only.push(row),
        Row::Operation(row) => map.operations.push(row),
    }
    Ok(())
}

fn finish(map: &mut KindMap) -> Result<(), KindMapError> {
    rows::validate_collisions(&map.entries, &map.api_only)?;
    let known = operations::known_kinds(&map.entries, &map.api_only);
    operations::validate(
        &known,
        &map.api_only,
        &map.operations,
        &map.relations,
        &map.bootstraps,
    )?;
    map.entries.sort_by(|a, b| a.cfn_type.cmp(&b.cfn_type));
    map.api_only.sort_by(|a, b| a.kind.cmp(&b.kind));
    map.operations
        .sort_by(|a, b| (&a.service, &a.operation).cmp(&(&b.service, &b.operation)));
    map.source_sha256 = fingerprint::digest(map);
    Ok(())
}
