use super::{DiscoveryContext, Record, SweepError};

pub(in crate::sweep_runtime) const MAX_PAGE_CONTEXT_BYTES: usize = 32 * 1024 * 1024;

pub(in crate::sweep_runtime) struct Rows<'a> {
    pub items: &'a [Record],
    pub offset: usize,
}

pub(in crate::sweep_runtime) struct PageContextBudget {
    used: usize,
}

impl PageContextBudget {
    pub(in crate::sweep_runtime) fn new(ancestry: &DiscoveryContext, total_rows: usize) -> Result<Self, SweepError> {
        let inherited = ancestry.encoded_len().map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?;
        inherited.checked_mul(total_rows).filter(|total| *total <= MAX_PAGE_CONTEXT_BYTES)
            .ok_or(SweepError::Operation("page discovery contexts exceed the aggregate byte bound"))?;
        Ok(Self { used: 0 })
    }

    pub(in crate::sweep_runtime) fn charge(&mut self, bytes: usize) -> Result<(), SweepError> {
        let next = self.used.checked_add(bytes).filter(|total| *total <= MAX_PAGE_CONTEXT_BYTES)
            .ok_or(SweepError::Operation("page discovery contexts exceed the aggregate byte bound"))?;
        self.used = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
