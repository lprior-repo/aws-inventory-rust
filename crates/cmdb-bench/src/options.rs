//! Workload options plus the bound checks the CLI relies on.

use std::error::Error;

pub const MAX_ACCOUNTS: usize = 1_000_000;
pub const MAX_PER_ACCOUNT: usize = 10_000_000;
pub const MAX_ASSETS: usize = 100_000_000;
pub const MAX_EMPTY_REGIONS: usize = 64;
pub const MAX_PAYLOAD_BYTES: usize = 65_536;
pub const MAX_DELETIONS: usize = 10_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeSpec {
    Uniform { per_account: usize },
    AssetsTotal { assets_total: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub accounts: usize,
    pub size: SizeSpec,
    pub seed: u64,
    pub empty_regions: usize,
    pub global_share: u64,
    pub payload_bytes: usize,
    pub duplicate_pct: u64,
    pub reorder_pct: u64,
    pub deletion_bursts: usize,
    pub deletion_burst_size: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            accounts: 1,
            size: SizeSpec::Uniform { per_account: 1 },
            seed: 1,
            empty_regions: 0,
            global_share: 0,
            payload_bytes: 256,
            duplicate_pct: 0,
            reorder_pct: 0,
            deletion_bursts: 0,
            deletion_burst_size: 0,
        }
    }
}

impl Options {
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        self.validate_accounts()?;
        self.validate_knobs()?;
        self.validate_deletions()
    }

    fn validate_accounts(&self) -> Result<(), Box<dyn Error>> {
        if !(1..=MAX_ACCOUNTS).contains(&self.accounts) {
            return Err(format!("--accounts must be in 1..={MAX_ACCOUNTS}").into());
        }
        let total = self.total_assets()?;
        if total > MAX_ASSETS {
            return Err(format!("workload assets must be <= {MAX_ASSETS}").into());
        }
        Ok(())
    }

    fn total_assets(&self) -> Result<usize, Box<dyn Error>> {
        match self.size {
            SizeSpec::Uniform { per_account } => {
                if !(1..=MAX_PER_ACCOUNT).contains(&per_account) {
                    return Err(format!("--per-account must be in 1..={MAX_PER_ACCOUNT}").into());
                }
                Ok(self
                    .accounts
                    .checked_mul(per_account)
                    .ok_or("account/per-account product overflows")?)
            }
            SizeSpec::AssetsTotal { assets_total } => {
                if assets_total < self.accounts {
                    return Err("--assets-total must be at least --accounts".into());
                }
                Ok(assets_total)
            }
        }
    }

    fn validate_knobs(&self) -> Result<(), Box<dyn Error>> {
        if self.empty_regions > MAX_EMPTY_REGIONS {
            return Err(format!("--empty-regions must be <= {MAX_EMPTY_REGIONS}").into());
        }
        if self.global_share > 100 {
            return Err("--global-share must be a percentage in 0..=100".into());
        }
        if self.payload_bytes > MAX_PAYLOAD_BYTES {
            return Err(format!("--payload-bytes must be <= {MAX_PAYLOAD_BYTES}").into());
        }
        if self.duplicate_pct > 100 {
            return Err("--duplicate-pct must be a percentage in 0..=100".into());
        }
        if self.reorder_pct > 100 {
            return Err("--reorder-pct must be a percentage in 0..=100".into());
        }
        Ok(())
    }

    fn validate_deletions(&self) -> Result<(), Box<dyn Error>> {
        if (self.deletion_bursts == 0) != (self.deletion_burst_size == 0) {
            return Err(
                "--deletion-bursts and --deletion-burst-size must both be positive or both zero"
                    .into(),
            );
        }
        let deletions = self
            .deletion_bursts
            .checked_mul(self.deletion_burst_size)
            .ok_or("deletion burst product overflows")?;
        if deletions > MAX_DELETIONS {
            return Err(format!("deletions must be <= {MAX_DELETIONS}").into());
        }
        Ok(())
    }
}
