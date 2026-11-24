use super::*;
use inventory_core::inventory::{ResourceKey, UnresolvedReference};

impl page::Page<'_, '_> {
    pub(super) fn parts(
        &self,
        child_context: &child::Context<'_, '_>,
        chunk: selection::Chunk,
        budget: &mut child::PageContextBudget,
    ) -> Result<PageParts, SweepError> {
        let contexts = self.contexts(&chunk, budget)?;
        let identities = self.identities(&chunk.items, &contexts)?;
        let references = self.references(&chunk.items, &identities)?;
        let tags = if self.emission == Emission::DiscoveryOnly {
            Vec::new()
        } else {
            rows::tags(self.plan, &chunk.items, &identities)?
        };
        let children = self.work(child_context, &chunk, &identities, &contexts, budget)?;
        let observations = if self.emission == Emission::DiscoveryOnly {
            Vec::new()
        } else {
            rows::observations(
                chunk.items,
                identities,
                self.plan.hash_exclusions(),
                self.plan.facet_precedence(),
            )?
        };
        Ok(PageParts {
            observations,
            references,
            tags,
            children,
            ..PageParts::default()
        })
    }

    fn work(
        &self,
        child_context: &child::Context<'_, '_>,
        chunk: &selection::Chunk,
        observations: &[impl AsRef<ResourceKey>],
        contexts: &[DiscoveryContext],
        budget: &mut child::PageContextBudget,
    ) -> Result<Vec<inventory_engine::inventory::WorkSpec>, SweepError> {
        child::work(
            child_context,
            &chunk.items,
            observations,
            &self.request.record,
            contexts,
            self.lease.scope(),
            budget,
        )
    }

    fn contexts(
        &self,
        chunk: &selection::Chunk,
        budget: &mut child::PageContextBudget,
    ) -> Result<Vec<DiscoveryContext>, SweepError> {
        child::contexts_at(
            self.plan,
            chunk
                .items
                .iter()
                .zip(chunk.ordinals.iter())
                .map(|(row, ordinal)| (*ordinal, row)),
            &self.request.record,
            &self.response.record,
            self.ancestry,
            self.lease.scope(),
            budget,
        )
    }

    fn identities(
        &self,
        items: &[inventory_core::record::Record],
        contexts: &[DiscoveryContext],
    ) -> Result<Vec<rows::RowIdentity>, SweepError> {
        if self.emission == Emission::DiscoveryOnly {
            return Ok(Vec::new());
        }
        rows::identities(
            self.context.catalog,
            record::IdentityContext {
                account: self.context.account,
                region: &self.request.region,
            },
            self.plan,
            self.lease.scope().region(),
            items,
            &self.request.record,
            contexts,
        )
    }

    fn references(
        &self,
        items: &[inventory_core::record::Record],
        observations: &[impl AsRef<ResourceKey>],
    ) -> Result<Vec<UnresolvedReference>, SweepError> {
        rows::references(
            self.context.catalog,
            self.plan,
            &self.request.region,
            items,
            observations,
            &self.request.record,
            self.lease.scope().parent(),
        )
    }
}
