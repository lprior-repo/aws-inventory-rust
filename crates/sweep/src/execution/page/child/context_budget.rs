#[cfg(test)]
use super::Record;
use super::{DiscoveryContext, SweepError};

pub(crate) const MAX_PAGE_CONTEXT_BYTES: usize = 32 * 1024 * 1024;

#[cfg(test)]
pub(crate) struct Rows<'a> {
    pub items: &'a [Record],
    pub offset: usize,
}

pub(crate) struct PageContextBudget {
    used: usize,
}

impl PageContextBudget {
    pub(crate) fn new(ancestry: &DiscoveryContext, total_rows: usize) -> Result<Self, SweepError> {
        let inherited = ancestry
            .encoded_len()
            .map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?;
        inherited
            .checked_mul(total_rows)
            .filter(|total| *total <= MAX_PAGE_CONTEXT_BYTES)
            .ok_or(SweepError::Operation(
                "page discovery contexts exceed the aggregate byte bound",
            ))?;
        Ok(Self { used: 0 })
    }

    pub(crate) fn charge(&mut self, bytes: usize) -> Result<(), SweepError> {
        let next = self
            .used
            .checked_add(bytes)
            .filter(|total| *total <= MAX_PAGE_CONTEXT_BYTES)
            .ok_or(SweepError::Operation(
                "page discovery contexts exceed the aggregate byte bound",
            ))?;
        self.used = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
