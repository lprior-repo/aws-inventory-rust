use super::*;
use inventory_core::{catalog::runtime::ExecutionBranch, record::Record};

pub(super) struct Page<'a, 'catalog> {
    pub(super) context: &'a Context<'a, 'catalog>,
    pub(super) lease: &'a WorkLease,
    pub(super) plan: &'a CallablePlan<'catalog>,
    pub(super) request: &'a PageRequest,
    pub(super) response: call::DecodedResponse,
    pub(super) ancestry: &'a DiscoveryContext,
    pub(super) branch: Option<&'a ExecutionBranch>,
    pub(super) emission: Emission,
}

impl<'a, 'catalog> Page<'a, 'catalog> {
    pub(super) fn new(
        context: &'a Context<'a, 'catalog>,
        lease: &'a WorkLease,
        plan: &'a CallablePlan<'catalog>,
        response: call::DecodedResponse,
        request: &'a PageRequest,
        ancestry: &'a DiscoveryContext,
    ) -> Result<Self, SweepError> {
        let branch = super::super::super::routing::execution_branch(plan, lease.scope().branch())?;
        let emission = plan.effective_emission(branch);
        Ok(Self { context, lease, plan, request, response, ancestry, branch, emission })
    }

    pub(super) async fn prepare(self, cursor: cursor::Cursor) -> Result<PageParts, SweepError> {
        if matches!(self.plan.purpose(), PlanPurpose::TagEndpoint(_)) {
            return tag_endpoint::prepare_page(
                self.context.catalog, self.context.account, self.context.max_pages,
                self.lease, self.plan, cursor, self.response,
            );
        }
        self.enumerate(cursor).await
    }

    async fn enumerate(&self, cursor: cursor::Cursor) -> Result<PageParts, SweepError> {
        let (total_rows, mut selected) = record::item_selection(&self.response.record, self.plan.items())?;
        if total_rows > MAX_PAGE_ROWS {
            return Err(SweepError::Operation("page exceeds the atomic observation bound"));
        }
        let (mut parts, mut tag_pairs) = self.collect(&mut selected, total_rows).await?;
        checkpoint(self.context.executor).await?;
        let additional = resource_emissions::additional(
            self.context, self.lease, self.plan, &self.response.record, self.request, self.ancestry,
        ).await?;
        append(&mut parts, additional, &mut tag_pairs)?;
        parts.next = cursor.next(
            self.plan.pagination(), &self.response.record, &self.response.headers, self.context.max_pages,
        )?;
        parts.rows = u64::try_from(parts.observations.len())
            .map_err(|_| SweepError::Operation("row count exceeds summary range"))?;
        Ok(parts)
    }

    async fn collect(
        &self,
        selected: &mut impl Iterator<Item = Result<Record, SweepError>>,
        total_rows: usize,
    ) -> Result<(PageParts, usize), SweepError> {
        let mut collected = Collection::new(self.ancestry, total_rows)?;
        let child_context = self.child_context();
        loop {
            checkpoint(self.context.executor).await?;
            let chunk = selection::select(self, selected, collected.offset)?;
            if chunk.raw_count == 0 { break; }
            collected.offset = selection::add_ordinal(collected.offset, chunk.raw_count)?;
            if chunk.items.is_empty() { continue; }
            let next = self.parts(&child_context, &chunk, &mut collected.budget)?;
            append(&mut collected.parts, next, &mut collected.tag_pairs)?;
        }
        Ok((collected.parts, collected.tag_pairs))
    }

    fn child_context(&self) -> child::Context<'_, 'catalog> {
        child::Context::new(self.plan, self.lease.sweep(), self.context.account,
            self.context.home, self.context.consumers)
    }
}

struct Collection {
    parts: PageParts,
    tag_pairs: usize,
    budget: child::PageContextBudget,
    offset: usize,
}

impl Collection {
    fn new(ancestry: &DiscoveryContext, total_rows: usize) -> Result<Self, SweepError> {
        let budget = child::PageContextBudget::new(ancestry, total_rows)?;
        Ok(Self { parts: PageParts::default(), tag_pairs: 0, budget, offset: 0 })
    }
}
