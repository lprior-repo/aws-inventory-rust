use super::*;
use std::collections::BTreeSet;
use toml::map::Map;

type Table = Map<String, toml::Value>;

const STEP_KEYS: &[&str] = &[
    "id",
    "operation",
    "kind",
    "identity",
    "repeat",
    "input",
    "check",
    "scope",
];

pub(super) struct Document<'a> {
    pub service: &'a str,
    pub steps: Vec<RawStep<'a>>,
    pub relationships: Vec<RawRelationship<'a>>,
}

pub(super) struct RawStep<'a> {
    pub id: &'a str,
    pub operation: &'a str,
    pub kind: &'a str,
    pub identity: Option<&'a str>,
    pub scope: SeedScope,
    pub repeat: u16,
    pub inputs: &'a Table,
    pub checks: &'a Table,
}

pub(super) struct RawRelationship<'a> {
    pub source: &'a str,
    pub target: &'a str,
    pub relation: &'a str,
    pub observed: &'a str,
}

impl<'a> Document<'a> {
    pub fn parse(value: &'a toml::Value) -> Result<Self, SeedError> {
        let root = table(value)?;
        keys(root, &["format", "service", "step", "relationship"])?;
        if root.get("format").and_then(toml::Value::as_integer) != Some(1) {
            return Err(SeedError::Invalid("seed file format must be version 1"));
        }
        let steps = array(root, "step")?
            .iter()
            .map(RawStep::parse)
            .collect::<Result<Vec<_>, _>>()?;
        let relationships = array(root, "relationship")?
            .iter()
            .map(RawRelationship::parse)
            .collect::<Result<Vec<_>, _>>()?;
        validate_ids(&steps)?;
        Ok(Self {
            service: field(root, "service")?,
            steps,
            relationships,
        })
    }
}

impl<'a> RawStep<'a> {
    fn parse(value: &'a toml::Value) -> Result<Self, SeedError> {
        let step = table(value)?;
        keys(step, STEP_KEYS)?;
        let kind = field(step, "kind")?;
        let identity = optional_field(step, "identity")?;
        if kind.is_empty() != identity.is_none() {
            return Err(SeedError::Invalid(
                "seed resource requires identity; auxiliary step cannot declare it",
            ));
        }
        Ok(Self {
            id: field(step, "id")?,
            operation: field(step, "operation")?,
            kind,
            identity,
            repeat: repeat(step)?,
            scope: scope(step)?,
            inputs: fields(step, "input")?,
            checks: fields(step, "check")?,
        })
    }
}

impl<'a> RawRelationship<'a> {
    fn parse(value: &'a toml::Value) -> Result<Self, SeedError> {
        let rel = table(value)?;
        keys(rel, &["source", "target", "relation", "observed"])?;
        Ok(Self {
            source: field(rel, "source")?,
            target: field(rel, "target")?,
            relation: field(rel, "relation")?,
            observed: field(rel, "observed")?,
        })
    }
}

pub(super) fn table(value: &toml::Value) -> Result<&Table, SeedError> {
    value
        .as_table()
        .ok_or(SeedError::Invalid("seed descriptor must be a table"))
}

pub(super) fn field<'a>(value: &'a Table, key: &str) -> Result<&'a str, SeedError> {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| SeedError::Message(format!("seed field {key} must be a string")))
}

pub(super) fn keys(value: &Table, allowed: &[&str]) -> Result<(), SeedError> {
    match value.keys().find(|key| !allowed.contains(&key.as_str())) {
        Some(key) => Err(SeedError::Message(format!("unknown seed field: {key}"))),
        None => Ok(()),
    }
}

fn optional_field<'a>(value: &'a Table, key: &str) -> Result<Option<&'a str>, SeedError> {
    value.get(key).map(|_| field(value, key)).transpose()
}

fn array<'a>(value: &'a Table, key: &str) -> Result<&'a [toml::Value], SeedError> {
    match value.get(key) {
        None => Ok(&[]),
        Some(toml::Value::Array(items)) if items.len() <= 4096 => Ok(items),
        _ => Err(SeedError::Invalid(
            "seed descriptors require an array of at most 4096 entries",
        )),
    }
}

fn fields<'a>(value: &'a Table, key: &str) -> Result<&'a Table, SeedError> {
    static EMPTY: std::sync::LazyLock<Table> = std::sync::LazyLock::new(Table::new);
    let result = match value.get(key) {
        Some(value) => table(value)?,
        None => &EMPTY,
    };
    if result.len() > 64 {
        return Err(SeedError::Invalid("seed step exceeds its field limit"));
    }
    Ok(result)
}

fn repeat(value: &Table) -> Result<u16, SeedError> {
    match value.get("repeat") {
        None => Ok(1),
        Some(toml::Value::Integer(count)) if (1..=4096).contains(count) => {
            u16::try_from(*count).map_err(|_| SeedError::Invalid("seed repeat is invalid"))
        }
        _ => Err(SeedError::Invalid(
            "seed repeat must be an integer from 1 through 4096",
        )),
    }
}

fn scope(value: &Table) -> Result<SeedScope, SeedError> {
    match optional_field(value, "scope")? {
        None | Some("region") => Ok(SeedScope::Regional),
        Some("account") => Ok(SeedScope::Account),
        _ => Err(SeedError::Invalid("seed scope must be account or region")),
    }
}

fn validate_ids(steps: &[RawStep<'_>]) -> Result<(), SeedError> {
    if steps.is_empty() {
        return Err(SeedError::Invalid("seed file requires at least one step"));
    }
    steps
        .iter()
        .try_fold(BTreeSet::new(), |mut seen, step| {
            if step.id.is_empty() || step.id.len() > 128 || !seen.insert(step.id) {
                return Err(SeedError::Invalid(
                    "seed step id is empty, oversized or duplicated",
                ));
            }
            Ok(seen)
        })
        .map(|_| ())
}
