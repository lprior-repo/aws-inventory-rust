use super::Error;
use std::collections::BTreeMap;

pub(super) const FIELDS: &[&str] = &[
    "operations",
    "enumerate",
    "expand",
    "denied",
    "unclassified",
    "expand_modeled",
    "expand_modeled_with_list_source",
    "expand_needs_overlay",
    "expand_needs_overlay_not_in_resource",
    "expand_needs_overlay_required_member_not_identifier",
    "ambiguous_resource",
    "list_describe",
    "list_describe_denied",
    "list_describe_paginated",
    "list_describe_without_paginator",
    "unpaginated",
    "pagination_needs_overlay",
    "pagination_needs_overlay_token_both_sides",
    "pagination_needs_overlay_token_input_only",
    "pagination_needs_overlay_token_output_only",
    "pagination_needs_overlay_truncation_flag",
    "paginated",
    "service_tokens_completed",
    "page_size_known",
    "page_size_unbounded",
    "paginated_without_page_size",
    "resources",
    "resources_with_list",
    "resources_with_read",
    "resources_with_list_and_read",
    "resources_with_arn",
    "batch_reads",
    "batch_reads_with_single_counterpart",
];
pub(super) const SERVICE_FIELDS: &[&str] = &[
    "enumerate",
    "expand",
    "expand_modeled",
    "expand_needs_overlay",
    "list_describe_paginated",
    "unpaginated",
    "pagination_needs_overlay",
];
const SPLITS: &[(&str, &[&str])] = &[
    (
        "operations",
        &["enumerate", "expand", "denied", "unclassified"],
    ),
    ("expand", &["expand_modeled", "expand_needs_overlay"]),
    (
        "expand_needs_overlay",
        &[
            "expand_needs_overlay_not_in_resource",
            "expand_needs_overlay_required_member_not_identifier",
        ],
    ),
    (
        "list_describe",
        &["enumerate", "expand", "list_describe_denied"],
    ),
    (
        "list_describe",
        &["list_describe_paginated", "list_describe_without_paginator"],
    ),
    (
        "list_describe_without_paginator",
        &["unpaginated", "pagination_needs_overlay"],
    ),
    (
        "pagination_needs_overlay",
        &[
            "pagination_needs_overlay_token_both_sides",
            "pagination_needs_overlay_token_input_only",
            "pagination_needs_overlay_token_output_only",
            "pagination_needs_overlay_truncation_flag",
        ],
    ),
    (
        "paginated",
        &[
            "page_size_known",
            "page_size_unbounded",
            "paginated_without_page_size",
        ],
    ),
];
const SUBSETS: &[(&str, &str)] = &[
    ("expand_modeled_with_list_source", "expand_modeled"),
    ("ambiguous_resource", "expand_modeled"),
    ("resources_with_list", "resources"),
    ("resources_with_read", "resources"),
    ("resources_with_arn", "resources"),
    ("resources_with_list_and_read", "resources_with_list"),
    ("resources_with_list_and_read", "resources_with_read"),
    ("batch_reads_with_single_counterpart", "batch_reads"),
    ("service_tokens_completed", "paginated"),
];

#[derive(Debug, Default)]
pub(super) struct Counts(BTreeMap<&'static str, u64>);
impl Counts {
    pub fn count(&self, key: &str) -> u64 {
        self.0.get(key).map_or(0, |count| *count)
    }
    pub fn increment(&mut self, key: &'static str) -> Result<(), Error> {
        self.add(key, 1)
    }
    pub fn add(&mut self, key: &'static str, amount: u64) -> Result<(), Error> {
        let next = self
            .count(key)
            .checked_add(amount)
            .ok_or(Error::CountMismatch)?;
        self.0.insert(key, next);
        Ok(())
    }
    pub fn merge(&mut self, other: &Self) -> Result<(), Error> {
        FIELDS
            .iter()
            .try_for_each(|key| self.add(key, other.count(key)))
    }
    pub fn validate(&self) -> Result<(), Error> {
        SPLITS.iter().try_for_each(|(total, parts)| {
            let sum = parts
                .iter()
                .try_fold(0_u64, |n, key| n.checked_add(self.count(key)))
                .ok_or(Error::CountMismatch)?;
            if sum != self.count(total) {
                return Err(Error::CountMismatch);
            }
            Ok(())
        })?;
        if SUBSETS
            .iter()
            .any(|(part, whole)| self.count(part) > self.count(whole))
        {
            return Err(Error::CountMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn violated_split_is_a_typed_error() -> Result<(), Error> {
        // Given an expansion without its evidence split, when validated, then it is rejected.
        let mut counts = Counts::default();
        counts.increment("expand")?;
        assert!(matches!(counts.validate(), Err(Error::CountMismatch)));
        Ok(())
    }
    #[test]
    fn overflowing_counter_is_a_typed_error() -> Result<(), Error> {
        // Given a full counter, when incremented, then overflow never wraps.
        let mut counts = Counts::default();
        counts.add("operations", u64::MAX)?;
        assert!(matches!(
            counts.increment("operations"),
            Err(Error::CountMismatch)
        ));
        Ok(())
    }
}
