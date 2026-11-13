//! Deterministic per-account layout: region assignment, identities, hubs, page counts.
//!
//! Everything here is a pure function of the account index, its size, the seed, and the
//! options, so generation can stream pages without retaining an account's resources in
//! memory and deletion records can address resources by index.

use std::error::Error;

use crate::model::{KINDS, PAGE_SIZE, REGIONS};
use crate::rng::Rng;

pub const INSTANCE: &str = "aws_ec2_instance";
pub const IAM_ROLE: &str = "aws_iam_role";
pub const GLOBAL_REGION: &str = "global";

const GLOBAL_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

fn error(message: &'static str) -> Box<dyn Error> {
    message.into()
}

#[must_use]
pub fn active_regions() -> [&'static str; 4] {
    REGIONS
}

pub fn empty_region_names(count: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::with_capacity(count);
    for offset in 0..count {
        let number = offset.checked_add(1).ok_or("empty region overflow")?;
        names.push(format!("empty-{number}"));
    }
    Ok(names)
}

pub fn account_region(account: usize) -> Result<&'static str, Box<dyn Error>> {
    let slot = account.checked_rem(REGIONS.len()).ok_or("region slot")?;
    REGIONS
        .get(slot)
        .copied()
        .ok_or_else(|| error("region slot out of range"))
}

/// Fraction of accounts flagged for a global service, derived per account from the seed.
pub fn is_global_account(seed: u64, account: usize, share: u64) -> Result<bool, Box<dyn Error>> {
    if share == 0 {
        return Ok(false);
    }
    let index = u64::try_from(account)?;
    let mut rng = Rng::new(seed ^ index.wrapping_mul(GLOBAL_SALT));
    Ok(rng.below(100) < share)
}

/// Number of leading resources consumed by the injected global service.
#[must_use]
pub fn shift(global: bool) -> usize {
    if global { 1 } else { 0 }
}

pub fn vpc_index(global: bool) -> Result<usize, Box<dyn Error>> {
    shift(global)
        .checked_add(1)
        .ok_or_else(|| error("vpc index overflow"))
}

pub fn security_group_index(global: bool) -> Result<usize, Box<dyn Error>> {
    shift(global)
        .checked_add(2)
        .ok_or_else(|| error("security group index overflow"))
}

pub fn kind_and_id(
    account: usize,
    index: usize,
    global: bool,
) -> Result<(&'static str, String), Box<dyn Error>> {
    if global && index == 0 {
        return Ok((IAM_ROLE, format!("role-global-{account:012}")));
    }
    let effective = index
        .checked_sub(shift(global))
        .ok_or("global index shift underflow")?;
    let slot = effective.checked_rem(KINDS.len()).ok_or("kind slot")?;
    let kind = KINDS.get(slot).copied().ok_or("kind slot out of range")?;
    let id = match kind {
        INSTANCE => format!("i-{account:08x}{effective:08x}"),
        "aws_ec2_vpc" => format!("vpc-{account:08x}{effective:08x}"),
        "aws_ec2_security_group" => format!("sg-{account:08x}{effective:08x}"),
        "aws_s3_bucket" => format!("bucket-{account:08x}-{effective:08x}"),
        _ => format!("role-{account:08x}-{effective:08x}"),
    };
    Ok((kind, id))
}

pub fn hub_id(
    account: usize,
    size: usize,
    global: bool,
    hub: usize,
) -> Result<Option<String>, Box<dyn Error>> {
    if size > hub {
        Ok(Some(kind_and_id(account, hub, global)?.1))
    } else {
        Ok(None)
    }
}

pub fn instance_count(size: usize, global: bool) -> Result<usize, Box<dyn Error>> {
    let patterned = size
        .checked_sub(shift(global))
        .ok_or("instance count underflow")?;
    let padded = patterned
        .checked_add(KINDS.len().checked_sub(1).ok_or("kind span underflow")?)
        .ok_or("instance count overflow")?;
    padded
        .checked_div(KINDS.len())
        .ok_or_else(|| error("instance divisor"))
}

pub fn edge_count(size: usize, global: bool) -> Result<usize, Box<dyn Error>> {
    let instances = instance_count(size, global)?;
    let mut per_instance = 0usize;
    if size > vpc_index(global)? {
        per_instance = per_instance.checked_add(1).ok_or("edge degree overflow")?;
    }
    if size > security_group_index(global)? {
        per_instance = per_instance.checked_add(1).ok_or("edge degree overflow")?;
    }
    instances
        .checked_mul(per_instance)
        .ok_or_else(|| error("edge count overflow"))
}

pub fn page_count(records: usize) -> Result<usize, Box<dyn Error>> {
    records
        .checked_add(PAGE_SIZE.checked_sub(1).ok_or("page span underflow")?)
        .ok_or("page count overflow")?
        .checked_div(PAGE_SIZE)
        .ok_or_else(|| error("page divisor"))
}
