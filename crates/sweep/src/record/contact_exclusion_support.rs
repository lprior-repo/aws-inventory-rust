use super::item_records;
use crate::execution::test_catalog;
use aws_wire::{
    request::Protocol,
    response::{DecodeLimits, ResponseContext, decode},
};
use catalog_tool::smithy::Registry;
use http::{HeaderMap, StatusCode};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    record::{MapRef, Record, ValueRef},
};
use serde_json::Value;

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub(super) const ADDRESS: &str = "owner@example.test";
pub(super) const PHONE: &str = "+15555550100";

pub(super) type Path = &'static [&'static str];

pub(super) struct Case {
    pub(super) service: &'static str,
    pub(super) date: &'static str,
    pub(super) protocol: Protocol,
    pub(super) operation: &'static str,
    pub(super) companions: &'static [&'static str],
    pub(super) body: Value,
    pub(super) rows: usize,
    pub(super) removed: &'static [Path],
    pub(super) kept: &'static [Path],
}

struct Stored {
    filtered: Vec<Record>,
    unfiltered: Vec<Record>,
    fields: Registry,
}

fn stored(case: &Case) -> TestResult<Stored> {
    let mut operations = vec![case.operation];
    operations.extend_from_slice(case.companions);
    let bytes = test_catalog::compile(case.service, case.date, &operations, None, |_, _| Ok(()))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(case.service, case.operation)
        .ok_or("plan absent")?;
    let payload = match &case.body {
        Value::String(raw) => raw.clone().into_bytes(),
        other => serde_json::to_vec(other)?,
    };
    let rows = |limits: DecodeLimits| -> TestResult<Vec<Record>> {
        let response = decode(ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol: case.protocol,
            body: &payload,
            headers: &HeaderMap::new(),
            status: StatusCode::OK,
            limits,
        })?;
        Ok(item_records(&response, plan.items())?)
    };
    let filtered = rows(DecodeLimits::default().excluding_members(plan.excluded_members()))?;
    let unfiltered = rows(DecodeLimits::default())?;
    let fields = Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    Ok(Stored {
        filtered,
        unfiltered,
        fields,
    })
}

fn map_has(map: MapRef<'_>, fields: &Registry, path: &[&str]) -> TestResult<bool> {
    let Some((first, rest)) = path.split_first() else {
        return Err("empty member path".into());
    };
    let id = fields.id(first).ok_or("member field absent")?;
    let Some(value) = map.get_field(id)? else {
        return Ok(false);
    };
    if rest.is_empty() {
        return Ok(true);
    }
    value_has(value, fields, rest)
}

fn value_has(value: ValueRef<'_>, fields: &Registry, path: &[&str]) -> TestResult<bool> {
    match value {
        ValueRef::Map(inner) => map_has(inner, fields, path),
        ValueRef::List(list) => {
            for element in list.iter()? {
                if value_has(element?, fields, path)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

pub(super) fn assert_contact_dropped(case: &Case) -> TestResult {
    let stored = stored(case)?;
    assert_eq!(stored.filtered.len(), case.rows);
    assert_eq!(stored.unfiltered.len(), case.rows);
    for row in &stored.unfiltered {
        for path in case.removed.iter().chain(case.kept) {
            assert!(
                map_has(row.root(), &stored.fields, path)?,
                "fixture lacks {path:?}"
            );
        }
    }
    for row in &stored.filtered {
        for path in case.removed {
            assert!(
                !map_has(row.root(), &stored.fields, path)?,
                "{path:?} was stored"
            );
        }
        for path in case.kept {
            assert!(
                map_has(row.root(), &stored.fields, path)?,
                "{path:?} was lost"
            );
        }
    }
    Ok(())
}
