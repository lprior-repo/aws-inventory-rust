use super::*;
use crate::execution::page::{child, consumer_index::ConsumerIndex, cursor::Cursor};
use inventory_core::{
    catalog::runtime::{BindingSource, ExecutionBranch},
    inventory::{DiscoveryContext, DiscoveryFrame, OperationId, ScopeKey, ServiceId},
};
use inventory_engine::inventory::SweepId;

/// Raw, unprojected items for an explicit equivalence probe; never persisted by sweep.
#[derive(Debug)]
#[non_exhaustive]
pub struct ProbePage {
    request: Record,
    response: Record,
    pub items: Vec<Record>,
    continuation: Option<Record>,
}

/// Bounded read-only probe transport reusing production routing and child bindings.
pub struct ProbeSession<'a, 'catalog, C> {
    probe: Probe<'a, 'catalog, C>,
    parent_branch: Option<&'a ExecutionBranch>,
    child_branch: Option<&'a ExecutionBranch>,
    consumers: ConsumerIndex<'a, 'catalog>,
    sweep: SweepId,
}

impl<'a, 'catalog, C: CredentialSource> ProbeSession<'a, 'catalog, C> {
    pub fn new(
        catalog: &'a RuntimeCatalog<'catalog>,
        credentials: &'a C,
        client: &'a Client,
        executor: &'a ApiExecutor,
        plan: &'a CallablePlan<'catalog>,
        scope: ProbeScope<'a>,
    ) -> Result<Self, SweepError> {
        Ok(Self {
            probe: Probe {
                catalog,
                credentials,
                client,
                executor,
                plan,
                scope,
            },
            parent_branch: plan.branches().first(),
            child_branch: None,
            consumers: ConsumerIndex::build(catalog).map_err(|_| {
                SweepError::Operation("comparison consumer index construction failed")
            })?,
            sweep: SweepId::new("equivalence-probe".to_owned())?,
        })
    }

    pub fn for_branches(
        mut self,
        parent: Option<&'a ExecutionBranch>,
        child: Option<&'a ExecutionBranch>,
    ) -> Self {
        self.parent_branch = parent;
        self.child_branch = child;
        self
    }

    pub fn is_bound_child(&self, child: &CallablePlan<'_>) -> bool {
        self.validate_child(child).is_ok()
    }

    fn selected_child_branch<'b>(
        &'b self,
        plan: &'b CallablePlan<'_>,
    ) -> Option<&'b ExecutionBranch> {
        self.child_branch.or_else(|| plan.branches().first())
    }

    pub async fn root_page(&self) -> Result<ProbePage, SweepError> {
        self.probe.validate()?;
        if !matches!(self.probe.plan.purpose(), PlanPurpose::Enumeration) {
            return Err(SweepError::InvalidConfig(
                "comparison parent must enumerate resources",
            ));
        }
        if !valid_branch(self.probe.plan, self.parent_branch) {
            return Err(SweepError::InvalidConfig(
                "comparison parent branch is not from this plan",
            ));
        }
        let scope = self.probe.scope();
        let branch = self.parent_branch;
        let base = input::root_bindings_branch(
            self.probe.plan,
            branch,
            self.probe.scope.account,
            Some(&scope),
        )?;
        self.page(self.probe.plan, branch, base, Cursor::default())
            .await
    }

    pub async fn next_page(&self, page: &ProbePage) -> Result<Option<ProbePage>, SweepError> {
        let Some(token) = page.continuation.as_ref() else {
            return Ok(None);
        };
        let cursor = Cursor::decode(Some(token.as_bytes()))?;
        let fields = cursor.request_fields(self.probe.plan.pagination())?;
        let base = input::merge_input(&page.request, &fields)?;
        self.page(self.probe.plan, self.parent_branch, base, cursor)
            .await
            .map(Some)
    }
    async fn describe(
        &self,
        plan: &CallablePlan<'catalog>,
        page: &ProbePage,
        row: &Record,
    ) -> Result<ProbePage, SweepError> {
        self.validate_child(plan)?;
        let account = self.probe.scope.account.ok_or(SweepError::InvalidConfig(
            "comparison requires account identity",
        ))?;
        let context = child::Context::new(
            self.probe.plan,
            &self.sweep,
            account,
            self.probe.scope.region,
            &self.consumers,
        );
        let scope = self.parent_scope(account)?;
        let ancestry = self.ancestry(page, row, &scope)?;
        let row = child::RowContext {
            row,
            parent: None,
            scope: &scope,
            ancestry: &ancestry,
        };
        let branch = self.selected_child_branch(plan);
        let fields = plan
            .effective_bindings(branch)
            .map(|binding| {
                let (_, value) = child::resolve_binding(
                    &context,
                    self.probe.scope.region,
                    &row,
                    &binding.source,
                )?;
                Ok(input::InputField {
                    path: binding.path.clone(),
                    value: value
                        .ok_or(SweepError::Operation("comparison child binding is missing"))?,
                })
            })
            .collect::<Result<Vec<_>, SweepError>>()?;
        let empty = Record::encode(&inventory_core::record::Value::Map(Default::default()))?;
        self.page(
            plan,
            branch,
            input::merge_input(&empty, &fields)?,
            Cursor::default(),
        )
        .await
    }

    fn validate_child(&self, child: &CallablePlan<'_>) -> Result<(), SweepError> {
        let found = self
            .probe
            .catalog
            .lookup(child.service_name(), child.operation_name());
        let child_branch = self.selected_child_branch(child);
        let parent_branch = self.parent_branch.map(|stage| stage.discriminator.as_str());
        if !found.is_some_and(|plan| std::ptr::eq(plan, child))
            || child.service_name() != self.probe.plan.service_name()
            || !valid_branch(child, child_branch)
            || !child.has_exclusive_skip_bindings(self.probe.plan, child_branch)
            // Unlike catalog validation, a probe has only one parent branch in context.
            || child.effective_bindings(child_branch).any(|binding| {
                matches!(&binding.source, BindingSource::ContextFromBranch { branch, .. }
                    if Some(branch.as_ref()) != parent_branch)
            })
        {
            return Err(SweepError::InvalidConfig(
                "comparison requires a bound child from the same catalog",
            ));
        }
        Ok(())
    }

    async fn page(
        &self,
        plan: &CallablePlan<'catalog>,
        branch: Option<&ExecutionBranch>,
        base: Record,
        cursor: Cursor,
    ) -> Result<ProbePage, SweepError> {
        bounded_bytes(&base)?;
        let response = self.raw_request(plan, branch, &base).await?;
        bounded_bytes(&response.record)?;
        let items = record::item_records(&response.record, plan.items())?;
        let total = items.iter().try_fold(0usize, |total, item| {
            total
                .checked_add(item.as_bytes().len())
                .ok_or(SweepError::Operation("comparison item byte count overflow"))
        })?;
        if total > 1_048_576 {
            return Err(SweepError::Operation(
                "comparison selected items exceed one MiB",
            ));
        }
        let continuation = continuation(plan, cursor, &response.record, &response.headers)?;
        Ok(ProbePage {
            request: base,
            response: response.record,
            items,
            continuation,
        })
    }

    async fn raw_request(
        &self,
        plan: &CallablePlan<'catalog>,
        branch: Option<&ExecutionBranch>,
        base: &Record,
    ) -> Result<call::DecodedResponse, SweepError> {
        let probe = &self.probe;
        let scope = probe.scope();
        let region = routing::request_region(
            plan,
            &scope,
            probe.scope.region,
            branch.map(|branch| branch.discriminator.as_str()),
        )?;
        let endpoint = routing::resolve(plan, &region)?;
        let fields = super::first_page_fields(plan);
        let metrics = metrics::Accumulators::default();
        call::Call {
            client: probe.client,
            executor: probe.executor,
            credentials: probe.credentials,
            plan,
            endpoint: &endpoint,
            url: probe
                .scope
                .endpoint_override
                .map_or(endpoint.url.as_str(), |url| url),
            base,
            fields: &fields,
            metrics: &metrics,
            account: probe
                .scope
                .account
                .map_or(crate::bucket::BOOTSTRAP_ACCOUNT, AccountId::as_str),
            scope: &scope,
        }
        .run_raw()
        .await
    }
    fn parent_scope(&self, account: &AccountId) -> Result<ScopeKey, SweepError> {
        let scope = ScopeKey::new(
            account.clone(),
            self.probe.scope(),
            ServiceId::parse(self.probe.plan.service_name())?,
            OperationId::parse(self.probe.plan.operation_name())?,
            None,
        )?;
        match self.parent_branch {
            Some(branch) => Ok(scope.with_branch(branch.discriminator.to_string())?),
            None => Ok(scope),
        }
    }

    fn ancestry(
        &self,
        page: &ProbePage,
        row: &Record,
        scope: &ScopeKey,
    ) -> Result<DiscoveryContext, SweepError> {
        let frame = DiscoveryFrame::project(
            scope.service().clone(),
            scope.operation().clone(),
            0,
            self.probe.plan.context_projection(),
            row,
            &page.response,
            &page.request,
        )
        .and_then(|frame| frame.with_branch(scope.branch()))
        .map_err(|_| SweepError::Operation("comparison discovery projection failed"))?;
        DiscoveryContext::empty()
            .push(frame)
            .map_err(|_| SweepError::Operation("comparison discovery context exceeds bound"))
    }
}

/// Compares one seeded resource using raw selected values, not persisted projections.
/// The list item is borrowed; the describe result is at most one MiB.
pub async fn probe_pair<'row, 'catalog, C: CredentialSource>(
    session: &ProbeSession<'_, 'catalog, C>,
    child: &CallablePlan<'catalog>,
    page: &ProbePage,
    row: &'row Record,
) -> Result<(&'row Record, Record), SweepError> {
    bounded_bytes(row)?;
    if !page.items.iter().any(|item| std::ptr::eq(item, row)) {
        return Err(SweepError::InvalidConfig(
            "comparison row does not belong to its parent page",
        ));
    }
    let response = session.describe(child, page, row).await?;
    if response.items.len() != 1 || response.continuation.is_some() {
        return Err(SweepError::Operation(
            "comparison describe must return exactly one structure",
        ));
    }
    let item = response
        .items
        .into_iter()
        .next()
        .ok_or(SweepError::Operation("comparison describe item is missing"))?;
    Ok((row, item))
}

fn bounded_bytes(record: &Record) -> Result<(), SweepError> {
    if record.as_bytes().len() > 1_048_576 {
        return Err(SweepError::Operation("comparison response exceeds one MiB"));
    }
    Ok(())
}

fn valid_branch(plan: &CallablePlan<'_>, branch: Option<&ExecutionBranch>) -> bool {
    match branch {
        None => plan.branches().is_empty(),
        Some(branch) => plan
            .branches()
            .iter()
            .any(|candidate| std::ptr::eq(candidate, branch)),
    }
}

fn continuation(
    plan: &CallablePlan<'_>,
    mut cursor: Cursor,
    response: &Record,
    headers: &http::HeaderMap,
) -> Result<Option<Record>, SweepError> {
    let previous = cursor.current.take();
    let Some(advance) = cursor.next(plan.pagination(), response, headers, 1024)? else {
        return Ok(None);
    };
    let next = Cursor::decode(Some(advance.cursor.as_bytes()))?;
    if next.partial
        || previous
            .as_ref()
            .zip(next.current.as_ref())
            .is_some_and(|(previous, next)| previous.as_bytes() == next.as_bytes())
    {
        return Err(SweepError::Operation(
            "comparison pagination repeats or exceeds its bound",
        ));
    }
    Ok(Some(advance.cursor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::record::{Key, Value};

    #[test]
    fn oversized_raw_record_is_an_error_not_a_truncation() -> Result<(), Box<dyn std::error::Error>>
    {
        let oversized = Record::encode(&Value::Map(
            [(
                Key::Name("value".into()),
                Value::Text("x".repeat(1_048_576)),
            )]
            .into(),
        ))?;
        assert!(bounded_bytes(&oversized).is_err());
        let small = Record::encode(&Value::Map(Default::default()))?;
        assert!(bounded_bytes(&small).is_ok());
        Ok(())
    }
}
