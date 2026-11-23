mod item;
mod match_cfn;
mod run;
pub use run::run;
pub(crate) use item::record as record_item;
use super::{KindMap, KindMapError, KindTarget};
use crate::{names::Exceptions, overlay::CheckedOverlays, plans::Model};
use serde_json::Value;
use std::{collections::{BTreeMap, BTreeSet}, fmt::Write};

type Failure = Box<dyn std::error::Error>;
pub const CONFLICT: &str = "kind: another enumeration already produces this resource";
const REASON: &str = "generated: no exact CloudFormation type";

pub struct Candidate<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub shape_id: &'a str,
    pub resource: &'a str,
    pub sdk_id: &'a str,
    pub cfn_service: Option<&'a str>,
    pub display: Option<&'a str>,
}

#[derive(Default)]
pub struct Generated {
    pub text: String,
    pub conflicts: BTreeSet<(String, String)>,
}

pub fn generate<'a>(
    values: &'a BTreeMap<String, Value>,
    checked: &'a CheckedOverlays,
    hand: &KindMap,
    cfn_source: &[u8],
) -> Result<Generated, Failure> {
    let mut candidates = Vec::new();
    for service in checked.services() {
        let value = values.get(service.service()).ok_or("kind: service model absent")?;
        let model = Model::parse(value, service.service())?;
        for operation in service.callable() {
            let candidate = item::candidate(&model, operation.service(), operation.operation(), operation.plan());
            if covered(hand, operation.service(), operation.operation()) && candidate.is_err() {
                continue;
            }
            if candidates.len() >= super::MAX_ROWS {
                return Err(KindMapError::TooManyRows.into());
            }
            candidates.push(candidate?);
        }
    }
    generate_candidates(candidates, hand, cfn_source)
}

pub fn generate_candidates(
    mut candidates: Vec<Candidate<'_>>,
    hand: &KindMap,
    cfn_source: &[u8],
) -> Result<Generated, Failure> {
    if candidates.len() > super::MAX_ROWS {
        return Err(KindMapError::TooManyRows.into());
    }
    candidates.sort_by_key(|candidate| (candidate.service, candidate.operation));
    let cfn = super::cfn_types(cfn_source)?;
    let mut state = State::new(&candidates, hand);
    let mut generated = Generated::default();
    for candidate in &candidates {
        if !covered(hand, candidate.service, candidate.operation) {
            state.add(candidate, hand, &cfn, &mut generated)?;
        }
    }
    generated.text = state.finish()?;
    hand.clone().with_generated(generated.text.as_bytes(), &Exceptions::builtin())?;
    Ok(generated)
}

fn covered(hand: &KindMap, service: &str, operation: &str) -> bool {
    hand.operations().iter().any(|row| row.service == service && row.operation == operation)
}

struct State<'a> {
    producers: BTreeMap<String, Option<(&'a str, &'a str)>>,
    kinds: BTreeMap<String, String>,
    operations: BTreeMap<(&'a str, &'a str), String>,
    bytes: usize,
}

impl<'a> State<'a> {
    fn new(candidates: &[Candidate<'a>], hand: &KindMap) -> Self {
        let mut producers = BTreeMap::new();
        for operation in hand.operations() {
            let shape = candidates.iter().find(|candidate| {
                candidate.service == operation.service && candidate.operation == operation.operation
            }).map(|candidate| (candidate.service, candidate.shape_id));
            producers.entry(operation.kind.clone()).or_insert(shape);
        }
        Self { producers, kinds: BTreeMap::new(), operations: BTreeMap::new(), bytes: 0 }
    }

    fn add(
        &mut self,
        candidate: &Candidate<'a>,
        hand: &KindMap,
        cfn: &BTreeSet<String>,
        generated: &mut Generated,
    ) -> Result<(), Failure> {
        let kind = match_cfn::choose(candidate, cfn, hand);
        let identity = Some((candidate.service, candidate.shape_id));
        if self.producers.get(&kind).is_some_and(|previous| *previous != identity) {
            generated.conflicts.insert((candidate.service.to_owned(), candidate.operation.to_owned()));
            return Ok(());
        }
        self.producers.insert(kind.clone(), identity);
        if !known(hand, &kind) && !self.kinds.contains_key(&kind) {
            let row = kind_row(candidate, &kind, cfn);
            self.reserve(row.len())?;
            self.kinds.insert(kind.clone(), row);
        }
        let row = format!("operation\t{}\t{}\t{kind}\n", candidate.service, candidate.operation);
        self.reserve(row.len())?;
        if self.operations.insert((candidate.service, candidate.operation), row).is_some() {
            return Err("kind: duplicate callable operation".into());
        }
        Ok(())
    }

    fn reserve(&mut self, length: usize) -> Result<(), KindMapError> {
        self.bytes = self.bytes.checked_add(length).ok_or(KindMapError::TooLarge)?;
        if self.bytes > super::MAX_BYTES {
            return Err(KindMapError::TooLarge);
        }
        Ok(())
    }

    fn finish(self) -> Result<String, Failure> {
        let mut text = String::new();
        text.try_reserve(self.bytes)?;
        for row in self.kinds.into_values().chain(self.operations.into_values()) {
            text.push_str(&row);
        }
        Ok(text)
    }
}

fn known(hand: &KindMap, kind: &str) -> bool {
    hand.api_only().iter().any(|entry| entry.kind == kind)
        || hand.entries().iter().any(|entry| {
            matches!(&entry.target, KindTarget::Mapped { kind: mapped, .. } if mapped == kind)
        })
}

fn kind_row(candidate: &Candidate<'_>, kind: &str, cfn: &BTreeSet<String>) -> String {
    let display = candidate.display.map_or("-", |path| path);
    if cfn.contains(kind) {
        format!("cfn\t{kind}\tmap\t{kind}\t{display}\n")
    } else {
        format!("api\t{kind}\tkind\t{REASON}\t{display}\n")
    }
}

impl Generated {
    pub fn report(&self) -> Result<String, std::fmt::Error> {
        let mut report = String::new();
        for (service, operation) in &self.conflicts {
            writeln!(report, "defer {service} {operation} {CONFLICT}")?;
        }
        writeln!(report, "kinds generated_rows={} conflicts={}", self.text.lines().count(), self.conflicts.len())?;
        Ok(report)
    }
}
