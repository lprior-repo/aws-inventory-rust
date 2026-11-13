//! Empty oracle skeleton for a workload configuration.

use std::collections::BTreeMap;
use std::error::Error;

use crate::layout;
use crate::model::{DeletionOracle, KINDS, Mode, MultiDigest, Oracle, PAGE_SIZE};
use crate::options::{Options, SizeSpec};

pub fn empty_oracle(options: &Options) -> Result<Oracle, Box<dyn Error>> {
    let empty_regions = layout::empty_region_names(options.empty_regions)?;
    Ok(Oracle {
        seed: options.seed,
        mode: mode(options),
        accounts: options.accounts,
        page_size: PAGE_SIZE,
        resources: 0,
        edges: 0,
        kinds: empty_kinds(),
        regions: empty_regions_map(&empty_regions),
        per_account: BTreeMap::new(),
        empty_regions,
        global_services: 0,
        duplicates: 0,
        reorders: 0,
        deletions: empty_deletions(),
        digest: String::new(),
    })
}

fn mode(options: &Options) -> Mode {
    match options.size {
        SizeSpec::Uniform { .. } => Mode::Uniform,
        SizeSpec::AssetsTotal { .. } => Mode::AssetsTotal,
    }
}

fn empty_kinds() -> BTreeMap<String, usize> {
    KINDS.iter().map(|kind| ((*kind).to_owned(), 0)).collect()
}

fn empty_regions_map(empty_regions: &[String]) -> BTreeMap<String, usize> {
    let mut regions: BTreeMap<String, usize> = layout::active_regions()
        .iter()
        .map(|region| ((*region).to_owned(), 0))
        .collect();
    regions.insert(layout::GLOBAL_REGION.to_owned(), 0);
    for name in empty_regions {
        regions.insert(name.clone(), 0);
    }
    regions
}

fn empty_deletions() -> DeletionOracle {
    DeletionOracle {
        count: 0,
        digest: MultiDigest::default().finish(),
        per_account: BTreeMap::new(),
    }
}
