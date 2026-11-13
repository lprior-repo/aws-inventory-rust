//! Resource projection, combinable filters and offset/cursor pagination.

use crate::Snapshot;
use crate::error::ViewError;
use crate::meta::SnapshotMeta;
use std::collections::BTreeMap;

/// Largest page a caller may request.
pub const MAX_PAGE_LIMIT: usize = 10_000;

/// The UI-facing projection of one `resource` record.
///
/// `properties_bytes` is the exact compact-JSON byte length of the wire
/// `properties` value, measured once at parse time; the property tree itself is
/// deliberately not retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceView {
    pub id: String,
    pub account_id: String,
    pub region: String,
    pub kind: String,
    pub name: Option<String>,
    pub tags: BTreeMap<String, String>,
    pub properties_bytes: usize,
}

impl ResourceView {
    /// ASCII case-insensitive containment over identity and tag text.
    ///
    /// Resource properties are not retained, so they are not searchable.
    pub fn matches_text(&self, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        contains_ascii_ci(&self.id, needle)
            || contains_ascii_ci(&self.account_id, needle)
            || contains_ascii_ci(&self.region, needle)
            || contains_ascii_ci(&self.kind, needle)
            || self
                .name
                .as_deref()
                .is_some_and(|name| contains_ascii_ci(name, needle))
            || self.tags.iter().any(|(key, value)| {
                contains_ascii_ci(key, needle) || contains_ascii_ci(value, needle)
            })
    }
}

/// All filters are optional and combined with AND.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceFilter {
    /// Exact account match.
    pub account_id: Option<String>,
    /// Exact region match.
    pub region: Option<String>,
    /// Exact resource kind match.
    pub kind: Option<String>,
    /// Case-insensitive containment over id, account, region, kind, name and tags.
    pub text: Option<String>,
}

impl ResourceFilter {
    /// An unfiltered view.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether one resource survives every configured filter.
    pub fn matches(&self, resource: &ResourceView) -> bool {
        self.account_id
            .as_ref()
            .is_none_or(|value| value == &resource.account_id)
            && self
                .region
                .as_ref()
                .is_none_or(|value| value == &resource.region)
            && self
                .kind
                .as_ref()
                .is_none_or(|value| value == &resource.kind)
            && self
                .text
                .as_deref()
                .is_none_or(|needle| resource.matches_text(needle))
    }
}

/// Offset plus a bounded page size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageRequest {
    offset: usize,
    limit: usize,
}

impl PageRequest {
    /// Validate a page request; `limit` must be in `1..=MAX_PAGE_LIMIT`.
    pub fn new(offset: usize, limit: usize) -> Result<Self, ViewError> {
        if !(1..=MAX_PAGE_LIMIT).contains(&limit) {
            return Err(ViewError::InvalidPageLimit);
        }
        Ok(Self { offset, limit })
    }

    /// Number of matching resources skipped before the page.
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Maximum resources in the page.
    pub const fn limit(self) -> usize {
        self.limit
    }

    /// Resume a page at an opaque cursor's offset.
    pub fn after(cursor: &PageCursor, limit: usize) -> Result<Self, ViewError> {
        Self::new(cursor.offset()?, limit)
    }
}

/// Opaque continuation token: the decimal offset of the next page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor(String);

impl PageCursor {
    /// Parse a cursor spelling persisted by a UI.
    pub fn parse(value: &str) -> Result<Self, ViewError> {
        if value.is_empty() || value.len() > 20 {
            return Err(ViewError::InvalidCursor);
        }
        let cursor = Self(value.to_owned());
        cursor.offset()?;
        Ok(cursor)
    }

    pub(crate) fn from_offset(offset: usize) -> Self {
        Self(offset.to_string())
    }

    /// The cursor spelling a UI can persist.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Parse the opaque spelling back into an offset.
    pub fn offset(&self) -> Result<usize, ViewError> {
        self.0
            .parse::<usize>()
            .map_err(|_| ViewError::InvalidCursor)
    }
}

impl std::fmt::Display for PageCursor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0.as_str())
    }
}

/// One deterministic page over the filtered, sorted resource views.
#[derive(Debug)]
pub struct ResourcePage<'a> {
    /// Borrowed views in `(account, region, kind, id)` order.
    pub resources: Vec<&'a ResourceView>,
    /// Number of resources matching the filter, independent of the page.
    pub total_matching: usize,
    /// Offset requested by the caller.
    pub offset: usize,
    /// Page size requested by the caller.
    pub limit: usize,
    /// Present only when more matching resources follow this page.
    pub next_cursor: Option<PageCursor>,
}

pub(crate) fn page<'a>(
    resources: &'a [ResourceView],
    filter: &ResourceFilter,
    request: PageRequest,
) -> Result<ResourcePage<'a>, ViewError> {
    let end = request
        .offset
        .checked_add(request.limit)
        .ok_or(ViewError::SizeOverflow)?;
    let mut page = ResourcePage {
        resources: Vec::new(),
        total_matching: 0,
        offset: request.offset,
        limit: request.limit,
        next_cursor: None,
    };
    collect_matches(&mut page, resources, filter, end)?;
    set_cursor(&mut page)?;
    Ok(page)
}

fn collect_matches<'a>(
    page: &mut ResourcePage<'a>,
    resources: &'a [ResourceView],
    filter: &ResourceFilter,
    end: usize,
) -> Result<(), ViewError> {
    for resource in resources {
        if !filter.matches(resource) {
            continue;
        }
        if page.total_matching >= page.offset && page.total_matching < end {
            page.resources.push(resource);
        }
        page.total_matching = page
            .total_matching
            .checked_add(1)
            .ok_or(ViewError::SizeOverflow)?;
    }
    Ok(())
}

fn set_cursor(page: &mut ResourcePage<'_>) -> Result<(), ViewError> {
    let consumed = page
        .offset
        .checked_add(page.resources.len())
        .ok_or(ViewError::SizeOverflow)?;
    if page.total_matching > consumed {
        page.next_cursor = Some(PageCursor::from_offset(consumed));
    }
    Ok(())
}

/// Zero-allocation ASCII case-insensitive containment.
pub(crate) fn contains_ascii_ci(haystack: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    if needle.is_empty() {
        return true;
    }
    let haystack = haystack.as_bytes();
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

impl Snapshot {
    /// Identity of the parsed snapshot.
    pub fn meta(&self) -> &SnapshotMeta {
        &self.meta
    }

    /// SHA-256 hex fingerprint suitable for change detection.
    pub fn fingerprint(&self) -> &str {
        self.meta.fingerprint()
    }

    /// Sorted resource projection shared by every query.
    pub fn resources(&self) -> &[ResourceView] {
        &self.resources
    }

    /// Look up one resource by its wire id.
    pub fn resource(&self, id: &str) -> Option<&ResourceView> {
        let position = *self.resource_index.get(id)?;
        self.resources.get(position)
    }

    /// One filtered, deterministically ordered page.
    pub fn page<'a>(
        &'a self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage<'a>, ViewError> {
        page(&self.resources, filter, request)
    }
}
