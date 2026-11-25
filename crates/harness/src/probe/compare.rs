mod shapes;
#[cfg(test)]
mod tests;
mod values;

use super::*;
use catalog_tool::overlay::{OperationName, OperationOverlay, ServiceOverlay, SkipEvidence};
use futures_util::TryStreamExt;
use inventory_sweep::{ProbeScope, ProbeSession, probe_pair};
use std::collections::{BTreeMap, BTreeSet};

struct Evidence {
    service: String,
    parent: String,
    child: String,
    inputs_sha256: String,
    compared: String,
    resources: u32,
}

enum Comparison {
    Equal(u32),
    Different(String),
    Unverified(u32, UnverifiedReason),
}

enum UnverifiedReason {
    Empty,
    Unbound,
}

impl<'catalog, C: CredentialSource> Probe<'_, 'catalog, C> {
    pub(super) async fn compare(
        &self,
        pairs: &[String],
        when: &str,
    ) -> Result<(String, String), ProbeError> {
        if when.trim().is_empty() || when.len() > 4096 || when.chars().any(char::is_control) {
            return Err(ProbeError::Invalid(
                "comparison date must be bounded nonempty text",
            ));
        }
        let mut distinct = BTreeSet::new();
        if let Some(duplicate) = pairs.iter().find(|pair| !distinct.insert(pair.as_str())) {
            return Err(ProbeError::DuplicatePair(duplicate.clone()));
        }
        let (_, account) = self.bootstrap().await;
        let account = account.ok_or(ProbeError::Invalid(
            "comparison requires successful account identity probe",
        ))?;
        let (report, evidence) = stream::iter(pairs)
            .then(|pair| self.compare_pair(pair, &account))
            .try_fold(
                (String::new(), Vec::new()),
                |(mut report, mut evidence), (row, result)| async move {
                    report.push_str(&row);
                    if let Some(result) = result {
                        evidence.push(result);
                    }
                    Ok::<_, ProbeError>((report, evidence))
                },
            )
            .await?;
        Ok((report, overlays(evidence, when)?))
    }

    async fn compare_pair(
        &self,
        pair: &str,
        account: &AccountId,
    ) -> Result<(String, Option<Evidence>), ProbeError> {
        let parts: Vec<_> = pair.split('/').collect();
        let [service, parent, child] = parts.as_slice() else {
            return Err(ProbeError::Invalid(
                "compare-pair requires service/Parent/Child",
            ));
        };
        let parent = self
            .catalog
            .lookup(service, parent)
            .ok_or(ProbeError::Invalid("comparison parent is not callable"))?;
        let child = self
            .catalog
            .lookup(service, child)
            .ok_or(ProbeError::Invalid("comparison child is not callable"))?;
        if let Some(path) = shapes::missing(parent, child)? {
            return Ok((format!("{pair}\tdifferent\t{path}\n"), None));
        }
        let result = self.compare_variants(parent, child, account).await?;
        result_row(self.catalog, parent, child, result)
    }

    async fn compare_variants(
        &self,
        parent: &CallablePlan<'catalog>,
        child: &CallablePlan<'catalog>,
        account: &AccountId,
    ) -> Result<Comparison, ProbeError> {
        let mut count = 0u32;
        for parent_branch in branches(parent) {
            for child_branch in branches(child) {
                let session = ProbeSession::new(
                    self.catalog,
                    self.credentials,
                    self.client,
                    self.executor,
                    parent,
                    ProbeScope {
                        region: self.region,
                        account: Some(account),
                        endpoint_override: Some(self.endpoint),
                    },
                )?
                .for_branches(parent_branch, child_branch);
                if !session.is_bound_child(child) {
                    return Ok(Comparison::Unverified(count, UnverifiedReason::Unbound));
                }
                let (resources, difference) = compare_resources(&session, parent, child).await?;
                if resources == 0 {
                    return Ok(Comparison::Unverified(count, UnverifiedReason::Empty));
                }
                count = count
                    .checked_add(resources)
                    .ok_or(ProbeError::TotalOverflow)?;
                if let Some(path) = difference {
                    return Ok(Comparison::Different(path));
                }
            }
        }
        Ok(Comparison::Equal(count))
    }
}

fn branches<'a>(
    plan: &'a CallablePlan<'_>,
) -> impl Iterator<Item = Option<&'a inventory_core::catalog::runtime::ExecutionBranch>> {
    std::iter::once(None)
        .filter(move |_| plan.branches().is_empty())
        .chain(plan.branches().iter().map(Some))
}

async fn compare_resources<'catalog, C: CredentialSource>(
    session: &ProbeSession<'_, 'catalog, C>,
    parent: &CallablePlan<'catalog>,
    child: &CallablePlan<'catalog>,
) -> Result<(u32, Option<String>), ProbeError> {
    let mut page = Some(session.root_page().await?);
    let mut count = 0u32;
    while let Some(current) = page {
        for row in &current.items {
            let (list, describe) = probe_pair(session, child, &current, row).await?;
            count = count.checked_add(1).ok_or(ProbeError::TotalOverflow)?;
            if let Some(path) = values::difference(parent.service(), list, &describe)? {
                return Ok((count, Some(path)));
            }
        }
        page = session.next_page(&current).await?;
    }
    Ok((count, None))
}

fn result_row(
    catalog: &RuntimeCatalog<'_>,
    parent: &CallablePlan<'_>,
    child: &CallablePlan<'_>,
    comparison: Comparison,
) -> Result<(String, Option<Evidence>), ProbeError> {
    let pair = format!(
        "{}/{}/{}",
        parent.service_name(),
        parent.operation_name(),
        child.operation_name()
    );
    let resources = match comparison {
        Comparison::Equal(resources) => resources,
        Comparison::Different(path) => {
            return Ok((format!("{pair}\tdifferent\t{path}\n"), None));
        }
        Comparison::Unverified(count, reason) => {
            let detail = match (count, reason) {
                (0, UnverifiedReason::Empty) => "no seeded resources".into(),
                (0, UnverifiedReason::Unbound) => {
                    "child binding is not exclusive to the parent branch".into()
                }
                (count, UnverifiedReason::Empty) => {
                    format!("compared {count} before an empty branch variant")
                }
                (count, UnverifiedReason::Unbound) => {
                    format!(
                        "compared {count} before a child branch not bound exclusively to the parent"
                    )
                }
            };
            return Ok((format!("{pair}\tunverified\t{detail}\n"), None));
        }
    };
    let compared = format!(
        "{}.{} == {}.{}; all modeled describe members and raw values",
        parent.operation_name(),
        selector(parent)?,
        child.operation_name(),
        selector(child)?
    );
    let evidence = Evidence {
        service: parent.service_name().into(),
        parent: parent.operation_name().into(),
        child: child.operation_name().into(),
        inputs_sha256: SkipEvidence::fingerprint(
            catalog,
            parent.service_name(),
            parent.operation_name(),
            child.operation_name(),
        )?,
        compared,
        resources,
    };
    Ok((format!("{pair}\tequal\t{resources}\n"), Some(evidence)))
}

fn selector(plan: &CallablePlan<'_>) -> Result<String, ProbeError> {
    use inventory_core::catalog::runtime::ItemPlan;
    match plan.items() {
        ItemPlan::Root => Ok("$".into()),
        ItemPlan::Path(path) | ItemPlan::Select { path, .. } => path
            .iter()
            .map(|id| Ok(plan.service().field_name(*id)?.to_owned()))
            .collect::<Result<Vec<_>, ProbeError>>()
            .map(|names| names.join(".")),
    }
}

fn overlays(evidence: Vec<Evidence>, when: &str) -> Result<String, ProbeError> {
    let groups = evidence
        .iter()
        .fold(BTreeMap::<_, Vec<_>>::new(), |mut groups, proof| {
            groups
                .entry((proof.service.as_str(), proof.parent.as_str()))
                .or_default()
                .push(proof);
            groups
        });
    groups
        .into_iter()
        .try_fold(String::new(), |mut text, ((service, parent), children)| {
            let mut entry = OperationOverlay::default();
            children.into_iter().try_for_each(|proof| {
                let name = OperationName::new(&proof.child)?;
                let receipt = SkipEvidence::new(
                    service,
                    parent,
                    &proof.child,
                    &proof.inputs_sha256,
                    &proof.compared,
                    proof.resources,
                    when,
                )?;
                entry.skip.push(name.clone());
                entry.skip_evidence.insert(name, receipt);
                Ok::<_, ProbeError>(())
            })?;
            entry.skip.sort_unstable();
            let policy = ServiceOverlay::new(service, vec![(parent, entry)])?;
            text.push_str(&catalog_tool::overlay::generate::render_skip_receipts(
                &policy,
            )?);
            Ok(text)
        })
}
