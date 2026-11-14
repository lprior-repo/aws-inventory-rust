use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::layout;
use crate::model::{DeletionOracle, DeletionRecord, MultiDigest, OBSERVED_AT, canonical_deletion};
use crate::options::Options;
use crate::rng::Rng;

const DELETION_SALT: u64 = 0x94D0_49BB_1331_11EB;

pub fn write_deletions(
    path: &Path,
    options: &Options,
    sizes: &[usize],
) -> Result<DeletionOracle, Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(path)?);
    let mut state = DeletionState::new(options.seed);
    for burst in 0..options.deletion_bursts {
        write_burst(&mut writer, &mut state, options, sizes, burst)?;
    }
    writer.flush()?;
    Ok(state.into_oracle())
}

struct DeletionState {
    rng: Rng,
    digest: MultiDigest,
    per_account: BTreeMap<String, usize>,
    count: usize,
}

impl DeletionState {
    fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed ^ DELETION_SALT),
            digest: MultiDigest::default(),
            per_account: BTreeMap::new(),
            count: 0,
        }
    }

    fn record(
        &mut self,
        writer: &mut impl Write,
        record: &DeletionRecord,
    ) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(&mut *writer, record)?;
        writer.write_all(b"\n")?;
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

fn write_burst(
    writer: &mut impl Write,
    state: &mut DeletionState,
    options: &Options,
    sizes: &[usize],
    burst: usize,
) -> Result<(), Box<dyn Error>> {
    let accounts = u64::try_from(options.accounts)?;
    let account = usize::try_from(state.rng.below(accounts))?;
    let size = sizes.get(account).copied().ok_or("deletion account size")?;
    let start = usize::try_from(state.rng.below(u64::try_from(size)?))?;
    let global = layout::is_global_account(options.seed, account, options.global_share)?;
    let deleted_at = i64::try_from(burst)?.saturating_add(OBSERVED_AT);
    for step in 0..options.deletion_burst_size {
        let record = deletion_record(account, start, step, size, global, deleted_at)?;
        state.record(writer, &record)?;
    }
    Ok(())
}

fn deletion_record(
    account: usize,
    start: usize,
    step: usize,
    size: usize,
    global: bool,
    deleted_at: i64,
) -> Result<DeletionRecord, Box<dyn Error>> {
    let index = start
        .checked_add(step)
        .ok_or("deletion index overflow")?
        .checked_rem(size)
        .ok_or("deletion index remainder")?;
    let (kind, id) = layout::kind_and_id(account, index, global)?;
    Ok(DeletionRecord {
        account_id: format!("{:012}", account.checked_add(1).ok_or("account overflow")?),
        resource_id: id,
        kind: kind.to_owned(),
        deleted_at,
    })
}
