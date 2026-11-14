use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use inventory_core::{CollectionEvent, CollectionPage};

use crate::model::{
    AccountStats, DeletionOracle, DeletionRecord, MultiDigest, Oracle, canonical_deletion,
    canonical_edge, canonical_resource,
};

pub fn verify(root: &Path) -> Result<Oracle, Box<dyn Error>> {
    let expected = load_oracle(root)?;
    let events = scan_events(&root.join("events.jsonl"), &expected)?;
    let deletions = scan_deletions(&root.join("deletions.jsonl"))?;
    let actual = assemble(&expected, events, deletions);
    if actual != expected {
        return Err(format!("workload does not match its oracle: {actual:?}").into());
    }
    Ok(actual)
}

fn load_oracle(root: &Path) -> Result<Oracle, Box<dyn Error>> {
    let bytes = fs::read(root.join("oracle.json"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn assemble(expected: &Oracle, events: EventScan, deletions: DeletionOracle) -> Oracle {
    Oracle {
        seed: expected.seed,
        mode: expected.mode,
        accounts: expected.accounts,
        page_size: expected.page_size,
        resources: events.resources,
        edges: events.edges,
        kinds: events.kinds,
        regions: events.regions,
        per_account: events.per_account,
        empty_regions: expected.empty_regions.clone(),
        global_services: expected.global_services,
        duplicates: events.duplicates,
        reorders: expected.reorders,
        deletions,
        digest: events.digest.finish(),
    }
}

struct EventScan {
    resources: usize,
    edges: usize,
    kinds: BTreeMap<String, usize>,
    regions: BTreeMap<String, usize>,
    per_account: BTreeMap<String, AccountStats>,
    duplicates: usize,
    digest: MultiDigest,
}

impl EventScan {
    fn new(expected: &Oracle) -> Self {
        Self {
            resources: 0,
            edges: 0,
            kinds: zero_keys(&expected.kinds),
            regions: zero_keys(&expected.regions),
            per_account: expected
                .per_account
                .keys()
                .map(|key| (key.clone(), AccountStats::default()))
                .collect(),
            duplicates: 0,
            digest: MultiDigest::default(),
        }
    }
}

fn zero_keys(map: &BTreeMap<String, usize>) -> BTreeMap<String, usize> {
    map.keys().map(|key| (key.clone(), 0)).collect()
}

fn scan_events(path: &Path, expected: &Oracle) -> Result<EventScan, Box<dyn Error>> {
    let mut scan = EventScan::new(expected);
    let mut reader = BufReader::new(File::open(path)?);
    scan_stream(&mut reader, &mut scan)?;
    Ok(scan)
}

fn scan_stream(reader: &mut impl BufRead, scan: &mut EventScan) -> Result<(), Box<dyn Error>> {
    let mut previous = String::new();
    let mut has_previous = false;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        if has_previous && line == previous {
            scan.duplicates = scan.duplicates.checked_add(1).ok_or("duplicate overflow")?;
            continue;
        }
        std::mem::swap(&mut previous, &mut line);
        has_previous = true;
        let event: CollectionEvent = serde_json::from_str(&previous)?;
        if let CollectionEvent::Page(page) = event {
            scan_page(scan, &page)?;
        }
    }
    Ok(())
}

fn scan_page(scan: &mut EventScan, page: &CollectionPage) -> Result<(), Box<dyn Error>> {
    let stats = scan
        .per_account
        .entry(page.scope.account_id.clone())
        .or_default();
    for resource in &page.resources {
        scan.resources = scan.resources.checked_add(1).ok_or("resource overflow")?;
        stats.resources = stats
            .resources
            .checked_add(1)
            .ok_or("account resource overflow")?;
        let entry = scan.kinds.entry(resource.kind.clone()).or_insert(0);
        *entry = entry.saturating_add(1);
        let entry = scan.regions.entry(resource.region.clone()).or_insert(0);
        *entry = entry.saturating_add(1);
        scan.digest.add(canonical_resource(resource).as_bytes());
    }
    for edge in &page.edges {
        scan.edges = scan.edges.checked_add(1).ok_or("edge overflow")?;
        stats.edges = stats.edges.checked_add(1).ok_or("account edge overflow")?;
        scan.digest.add(canonical_edge(edge).as_bytes());
    }
    Ok(())
}

#[derive(Default)]
struct DeletionScan {
    count: usize,
    per_account: BTreeMap<String, usize>,
    digest: MultiDigest,
}

impl DeletionScan {
    fn record(&mut self, record: &DeletionRecord) -> Result<(), Box<dyn Error>> {
        self.digest.add(canonical_deletion(record).as_bytes());
        self.count = self.count.checked_add(1).ok_or("deletion overflow")?;
        let entry = self
            .per_account
            .entry(record.account_id.clone())
            .or_insert(0);
        *entry = entry.saturating_add(1);
        Ok(())
    }

    fn into_oracle(self) -> DeletionOracle {
        DeletionOracle {
            count: self.count,
            digest: self.digest.finish(),
            per_account: self.per_account,
        }
    }
}

fn scan_deletions(path: &Path) -> Result<DeletionOracle, Box<dyn Error>> {
    let mut scan = DeletionScan::default();
    if path.exists() {
        let mut reader = BufReader::new(File::open(path)?);
        scan_deletion_stream(&mut reader, &mut scan)?;
    }
    Ok(scan.into_oracle())
}

fn scan_deletion_stream(
    reader: &mut impl BufRead,
    scan: &mut DeletionScan,
) -> Result<(), Box<dyn Error>> {
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let record: DeletionRecord = serde_json::from_str(&line)?;
        scan.record(&record)?;
    }
    Ok(())
}
