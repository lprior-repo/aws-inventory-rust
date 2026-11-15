use aws_wire::{request::Protocol, response::{self, DecodeLimits, ResponseContext}};
use http::{HeaderMap, StatusCode};
use inventory_core::{
    catalog::runtime::{CallablePlan, IdentityPart, IdentityPlan, ItemPlan, RuntimeProtocol},
    inventory::{AccountId, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey, ServiceId, Tag, TagCoverage, TagObservation},
    record::{FieldId, MapRef, Record, ValueRef, scalar_item, scalar_record},
};
use std::fmt::Write;
use super::pipeline::{BenchResult, Phase};

pub(super) const REGION: &str = "us-east-1";

pub(super) fn scope(shard: usize) -> BenchResult<ScopeKey> {
    let account = shard.checked_add(1).ok_or("account overflow")?;
    Ok(ScopeKey::new(
        AccountId::parse(&format!("{account:012}"))?, Region::parse(REGION)?,
        ServiceId::parse("dynamodb")?, OperationId::parse("ListTables")?, None,
    )?)
}

/// Typed, deterministic transport fixture, not a service/app configuration.
/// At most 1,000 short table names and one response body are live per page.
pub(super) struct TablePage {
    pub start: usize,
    pub rows: usize,
}

impl TablePage {
    pub fn body(&self) -> BenchResult<String> {
        let mut body = String::new();
        body.try_reserve(self.rows.checked_mul(24).ok_or("body size overflow")?.saturating_add(32))?;
        body.push_str("{\"TableNames\":[");
        (0..self.rows).try_for_each(|offset| -> BenchResult<()> {
            if offset != 0 { body.push(','); }
            let index = self.start.checked_add(offset).ok_or("fixture index overflow")?;
            write!(body, "\"bench-{index:012}\"")?;
            Ok(())
        })?;
        body.push_str("]}");
        Ok(body)
    }
}

pub(super) fn decode(
    plan: &CallablePlan<'_>, scope: &ScopeKey, fixture: &TablePage,
) -> BenchResult<Vec<Observation>> {
    if plan.protocol() != RuntimeProtocol::AwsJson10 {
        return Err("ListTables benchmark requires its current AWS JSON 1.0 protocol".into());
    }
    let body = fixture.body()?;
    let headers = HeaderMap::new();
    let decoded = response::decode(ResponseContext {
        service: plan.service(), operation: plan.operation(), protocol: Protocol::AwsJson10,
        body: body.as_bytes(), headers: &headers, status: StatusCode::OK,
        limits: DecodeLimits::new(64 * 1024, 4096, 16)?,
    })?;
    let ItemPlan::Path(path) = plan.items() else {
        return Err("ListTables must select the modeled TableNames list".into());
    };
    let Some(ValueRef::List(items)) = value_at(decoded.root(), path)? else {
        return Err("decoder omitted the modeled table-name list".into());
    };
    if items.count()? != fixture.rows { return Err("decoder table count mismatch".into()); }
    items.iter()?.map(|item| {
        let ValueRef::Text(name) = item? else { return Err("decoded table name is not text".into()); };
        let record = scalar_record(ValueRef::Text(name))?;
        let key = ResourceKey {
            account: scope.account().clone(), region: scope.region().clone(),
            kind: plan.kind().clone(), id: identity(&record, plan.identity(), scope)?,
        };
        Ok(Observation::new(key, record, plan.hash_exclusions(), Some(name.into()))?)
    }).collect()
}

fn value_at<'a>(root: MapRef<'a>, path: &[FieldId]) -> BenchResult<Option<ValueRef<'a>>> {
    let Some((leaf, parents)) = path.split_last() else { return Ok(scalar_item(root)?); };
    let parent = parents.iter().try_fold(Some(root), |current, field| -> BenchResult<_> {
        match current.map(|map| map.get_field(*field)).transpose()?.flatten() {
            Some(ValueRef::Map(child)) => Ok(Some(child)),
            None => Ok(None),
            _ => Err("identity parent is not a structure".into()),
        }
    })?;
    Ok(parent.map(|map| map.get_field(*leaf)).transpose()?.flatten())
}

fn text_at<'a>(record: &'a Record, path: &[FieldId]) -> BenchResult<&'a str> {
    match value_at(record.root(), path)? {
        Some(ValueRef::Text(text)) => Ok(text),
        _ => Err("table identity field is missing or is not text".into()),
    }
}

// The runtime normalizer is private. This fixture adapter deliberately uses the
// checked catalog identity rather than claiming to benchmark root drain_work.
fn identity(record: &Record, plan: &IdentityPlan, scope: &ScopeKey) -> BenchResult<ResourceId> {
    let text = match plan {
        IdentityPlan::Arn(path) => text_at(record, path)?.to_owned(),
        IdentityPlan::Key(paths) => match paths.as_ref() {
            [path] => text_at(record, path)?.to_owned(),
            _ => return Err("table benchmark requires a scalar identity or ARN template".into()),
        },
        IdentityPlan::Template(parts) => parts.iter().try_fold(String::new(), |mut text, part| -> BenchResult<_> {
            let value = match part {
                IdentityPart::Literal(value) => value.as_str(),
                IdentityPart::Field(path) => text_at(record, path)?,
                IdentityPart::Partition => "aws",
                IdentityPart::Account => scope.account().as_str(),
                IdentityPart::Region => scope.region().as_str(),
            };
            if text.len().saturating_add(value.len()) > 4096 { return Err("identity exceeds bound".into()); }
            text.try_reserve(value.len())?;
            text.push_str(value);
            Ok(text)
        })?,
    };
    Ok(ResourceId::parse(&text)?)
}

pub(super) fn tags(observations: &[Observation], start: usize, phase: Phase) -> BenchResult<Vec<TagObservation>> {
    observations.iter().enumerate().map(|(offset, observation)| {
        let index = start.checked_add(offset).ok_or("tag index overflow")?;
        let (coverage, tags) = tag_state(index, phase)?;
        Ok(TagObservation::new(observation.key().clone(), coverage, tags)?)
    }).collect()
}

fn tag_state(index: usize, phase: Phase) -> BenchResult<(TagCoverage, Vec<Tag>)> {
    if phase != Phase::Initial && index % 4 == 0 { return Ok((TagCoverage::Complete, Vec::new())); }
    if phase != Phase::Initial && index % 4 == 2 {
        return Ok((TagCoverage::Partial, vec![Tag::new("environment", "must-not-replace")?]));
    }
    let environment = if phase != Phase::Initial && index % 4 == 1 { "bench-updated" } else { "bench" };
    Ok((TagCoverage::Complete, vec![Tag::new("environment", environment)?, Tag::new("owner", "inventory")?]))
}

pub(super) fn expected_tags(rows_per_scope: usize, phase: Phase) -> BenchResult<usize> {
    let removed = if phase == Phase::Initial { 0 } else { rows_per_scope.div_ceil(4) };
    rows_per_scope.checked_sub(removed).and_then(|rows| rows.checked_mul(2)).ok_or_else(|| "tag count overflow".into())
}

pub(super) fn expected_tag_vertices(rows_per_scope: usize, phase: Phase) -> usize {
    if rows_per_scope == 0 { return 0; }
    if phase == Phase::Initial { return 2; }
    match rows_per_scope { 1 => 0, 2 => 2, _ => 3 }
}
