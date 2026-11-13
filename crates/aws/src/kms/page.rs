//! Page construction and identity de-duplication for KMS listings.

use std::collections::BTreeSet;

use aws_sdk_kms::operation::{list_aliases::ListAliasesOutput, list_keys::ListKeysOutput};
use inventory_core::Resource;

use crate::common::PageBody;

/// KMS list operations accept at most 1000 items per page.
pub(super) const LIST_MAX_RESULTS: i32 = 1_000;

pub(super) fn page_limit(page_size: i32) -> i32 {
    page_size.clamp(1, LIST_MAX_RESULTS)
}

/// `listed_count` counts every listed key, including keys whose enrichment
/// failed, so cursor accounting cannot hide attempted work.
pub(super) fn key_page_body(output: &ListKeysOutput, resources: Vec<Resource>) -> PageBody {
    PageBody {
        resources,
        next_token: output.next_marker().map(str::to_owned),
        listed_count: output.keys().len(),
        ..PageBody::default()
    }
}

pub(super) fn alias_page_body(output: &ListAliasesOutput, resources: Vec<Resource>) -> PageBody {
    PageBody {
        resources,
        next_token: output.next_marker().map(str::to_owned),
        listed_count: output.aliases().len(),
        ..PageBody::default()
    }
}

pub(super) fn dedupe(resources: Vec<Resource>) -> Vec<Resource> {
    let mut seen = BTreeSet::new();
    resources
        .into_iter()
        .filter(|resource| seen.insert(resource.id.clone()))
        .collect()
}
