//! pagination responsibilities for global_inventory.
use super::*;

pub(super) fn marker(truncated: bool, token: Option<&str>) -> Result<Option<String>, AwsScanError> {
    if truncated {
        token
            .filter(|token| !token.is_empty())
            .map(|token| Some(token.to_owned()))
            .ok_or_else(|| {
                AwsScanError::InvalidData("truncated response lacks continuation marker".into())
            })
    } else {
        Ok(None)
    }
}

/// Hashes only, with fixed cardinality. The mutex never crosses an await; one
/// list cursor owns each instance. It permits Fn request closures without a
/// shared account-wide cache of resource models or raw identifiers.
#[derive(Default)]
pub(super) struct SeenIds(Mutex<BTreeSet<[u8; 32]>>);

impl SeenIds {
    pub(super) fn record<'a>(
        &self,
        ids: impl Iterator<Item = &'a str>,
        budget: usize,
    ) -> Result<usize, AwsScanError> {
        let mut seen = self
            .0
            .lock()
            .map_err(|_| AwsScanError::InvalidData("identity ledger poisoned".into()))?;
        let mut page = BTreeSet::new();
        let mut count = 0usize;
        for id in ids.take(MAX_LIST_ITEMS.saturating_add(1)) {
            checked_text(id, "listed resource identifier")?;
            count = count.saturating_add(1);
            if count > MAX_LIST_ITEMS {
                return Err(AwsScanError::ChildLimit);
            }
            let digest: [u8; 32] = Sha256::digest(id.as_bytes()).into();
            if seen.contains(&digest) || !page.insert(digest) {
                return Err(AwsScanError::InvalidData(
                    "list response repeated a resource identifier".into(),
                ));
            }
        }
        require_budget(count, budget)?;
        if seen.len().saturating_add(count) > MAX_CHILDREN {
            return Err(AwsScanError::ChildLimit);
        }
        seen.extend(page);
        Ok(count)
    }
}
