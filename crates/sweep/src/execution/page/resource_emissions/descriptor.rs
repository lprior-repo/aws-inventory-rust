use super::*;

pub(super) struct Descriptor<'a, 'catalog> {
    context: &'a assembly::Context<'a, 'catalog>,
    lease: &'a WorkLease,
    plan: &'a CallablePlan<'catalog>,
    emission: &'a ResourceEmissionPlan,
    request: &'a PageRequest,
    ancestry: &'a DiscoveryContext,
}

impl<'a, 'catalog> Descriptor<'a, 'catalog> {
    pub(super) fn new(
        context: &'a assembly::Context<'a, 'catalog>,
        lease: &'a WorkLease,
        plan: &'a CallablePlan<'catalog>,
        emission: &'a ResourceEmissionPlan,
        request: &'a PageRequest,
        ancestry: &'a DiscoveryContext,
    ) -> Self {
        Self {
            context,
            lease,
            plan,
            emission,
            request,
            ancestry,
        }
    }

    pub(super) async fn collect(&self, response: &Record) -> Result<PageParts, SweepError> {
        let (count, selected) = record::item_selection(response, self.emission.items())?;
        if count > assembly::MAX_PAGE_ROWS {
            return Err(SweepError::Operation(
                "page exceeds the atomic observation bound",
            ));
        }
        self.collect_chunks(selected, count).await
    }

    async fn collect_chunks(
        &self,
        selected: impl Iterator<Item = Result<Record, SweepError>>,
        count: usize,
    ) -> Result<PageParts, SweepError> {
        stream::iter(0..count.div_ceil(assembly::ROWS_PER_CHECKPOINT))
            .map(Ok::<_, SweepError>)
            .try_fold(
                (PageParts::default(), 0, selected),
                |(mut parts, mut tags, mut selected), _| async move {
                    assembly::checkpoint(self.context.executor).await?;
                    let raw = selected
                        .by_ref()
                        .take(assembly::ROWS_PER_CHECKPOINT)
                        .collect::<Result<Vec<_>, _>>()?;
                    let next = self.chunk(raw)?;
                    assembly::append(&mut parts, next, &mut tags)?;
                    Ok((parts, tags, selected))
                },
            )
            .await
            .map(|(parts, _, _)| parts)
    }

    pub(super) fn chunk(&self, raw: Vec<Record>) -> Result<PageParts, SweepError> {
        let items = selected(self.emission, raw)?;
        let identities = self.identities(&items)?;
        let references = rows::references(
            self.context.catalog,
            self.plan,
            &self.request.region,
            &items,
            &identities,
            &self.request.record,
            self.lease.scope().parent(),
        )?;
        let tags = rows::emission_tags(self.emission, &items, &identities)?;
        let observations = rows::observations(
            items,
            identities,
            self.emission.hash_exclusions(),
            self.plan.facet_precedence(),
        )?;
        Ok(PageParts {
            observations,
            references,
            tags,
            ..PageParts::default()
        })
    }

    fn scope(&self) -> rows::EmissionScope<'_> {
        rows::EmissionScope {
            catalog: self.context.catalog,
            identity_context: record::IdentityContext {
                account: self.context.account,
                region: &self.request.region,
            },
            plan: self.plan,
            emission: self.emission,
            region: self.lease.scope().region(),
            request: &self.request.record,
            ancestry: self.ancestry,
        }
    }

    fn identities(&self, items: &[Record]) -> Result<Vec<rows::RowIdentity>, SweepError> {
        items
            .iter()
            .map(|row| rows::emission_identity(self.scope(), row))
            .collect()
    }
}
