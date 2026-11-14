use std::error::Error;

use crate::options::{Options, SizeSpec};
use crate::rng::Rng;

const TAIL_SEED_SALT: u64 = 0xD1B5_4A32_D192_ED03;
const WEIGHT_SCALE: u128 = 1_000_000_000_000_000_000;

type Fractions = Vec<(u128, usize)>;
type Apportioned = (Vec<usize>, Fractions, usize);

fn error(message: &'static str) -> Box<dyn Error> {
    message.into()
}

pub fn account_sizes(options: &Options) -> Result<Vec<usize>, Box<dyn Error>> {
    match options.size {
        SizeSpec::Uniform { per_account } => Ok(vec![per_account; options.accounts]),
        SizeSpec::AssetsTotal { assets_total } => {
            skewed_sizes(options.accounts, assets_total, options.seed)
        }
    }
}

pub fn skewed_sizes(
    accounts: usize,
    total: usize,
    seed: u64,
) -> Result<Vec<usize>, Box<dyn Error>> {
    if accounts == 0 {
        return Err("accounts must be positive".into());
    }
    let base_total = total
        .checked_sub(accounts)
        .ok_or("assets total below account count")?;
    let weights = shuffled_weights(accounts, seed)?;
    apportion(accounts, base_total, &weights)
}

fn shuffled_weights(accounts: usize, seed: u64) -> Result<Vec<u128>, Box<dyn Error>> {
    let mut weights = harmonic_weights(accounts)?;
    let mut rng = Rng::new(seed ^ TAIL_SEED_SALT);
    for upper in (1..accounts).rev() {
        let span = u64::try_from(upper)?
            .checked_add(1)
            .ok_or("shuffle span overflow")?;
        weights.swap(upper, usize::try_from(rng.below(span))?);
    }
    Ok(weights)
}

fn harmonic_weights(accounts: usize) -> Result<Vec<u128>, Box<dyn Error>> {
    let mut weights = Vec::with_capacity(accounts);
    for rank in 0..accounts {
        let divisor = u128::try_from(rank)?
            .checked_add(1)
            .ok_or("rank divisor overflow")?;
        weights.push(
            WEIGHT_SCALE
                .checked_div(divisor)
                .ok_or_else(|| error("weight divisor"))?,
        );
    }
    Ok(weights)
}

fn apportion(
    accounts: usize,
    base_total: usize,
    weights: &[u128],
) -> Result<Vec<usize>, Box<dyn Error>> {
    let total_weight = sum_weights(weights)?;
    let residual = u128::try_from(base_total)?;
    let (mut sizes, fractions, allocated) = base_shares(accounts, residual, total_weight, weights)?;
    let remaining = base_total
        .checked_sub(allocated)
        .ok_or("allocation exceeds residual")?;
    apply_remainder(&mut sizes, fractions, remaining)?;
    Ok(sizes)
}

fn sum_weights(weights: &[u128]) -> Result<u128, Box<dyn Error>> {
    weights
        .iter()
        .copied()
        .try_fold(0u128, |sum, weight| sum.checked_add(weight))
        .ok_or_else(|| error("weight sum overflow"))
}

fn base_shares(
    accounts: usize,
    residual: u128,
    total_weight: u128,
    weights: &[u128],
) -> Result<Apportioned, Box<dyn Error>> {
    let mut sizes = Vec::with_capacity(accounts);
    let mut fractions = Vec::with_capacity(accounts);
    let mut allocated = 0u128;
    for (index, weight) in weights.iter().copied().enumerate() {
        let (share, fraction) = allocate(residual, weight, total_weight)?;
        fractions.push((fraction, index));
        allocated = allocated
            .checked_add(share)
            .ok_or("allocation sum overflow")?;
        let size = usize::try_from(share)?
            .checked_add(1)
            .ok_or("account size overflow")?;
        sizes.push(size);
    }
    Ok((sizes, fractions, usize::try_from(allocated)?))
}

fn allocate(
    residual: u128,
    weight: u128,
    total_weight: u128,
) -> Result<(u128, u128), Box<dyn Error>> {
    let product = residual.checked_mul(weight).ok_or("allocation overflow")?;
    let share = product
        .checked_div(total_weight)
        .ok_or("allocation divisor")?;
    let fraction = product
        .checked_rem(total_weight)
        .ok_or("allocation remainder")?;
    Ok((share, fraction))
}

fn apply_remainder(
    sizes: &mut [usize],
    mut fractions: Fractions,
    mut remaining: usize,
) -> Result<(), Box<dyn Error>> {
    fractions.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    for (_, index) in &fractions {
        if remaining == 0 {
            break;
        }
        if let Some(size) = sizes.get_mut(*index) {
            *size = size.saturating_add(1);
            remaining = remaining.saturating_sub(1);
        }
    }
    if remaining == 0 {
        Ok(())
    } else {
        Err("distribution remainder not fully assigned".into())
    }
}
