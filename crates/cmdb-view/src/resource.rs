use crate::Snapshot;
use crate::error::ViewError;
use crate::meta::SnapshotMeta;
use std::collections::BTreeMap;

pub const MAX_PAGE_LIMIT: usize = 10_000;

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceFilter {
    pub account_id: Option<String>,

    pub region: Option<String>,

    pub kind: Option<String>,

    pub text: Option<String>,
}

impl ResourceFilter {
    pub fn new() -> Self {
        Self::default()
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageRequest {
    offset: usize,
    limit: usize,
}

impl PageRequest {
    pub fn new(offset: usize, limit: usize) -> Result<Self, ViewError> {
        if !(1..=MAX_PAGE_LIMIT).contains(&limit) {
            return Err(ViewError::InvalidPageLimit);
        }
        Ok(Self { offset, limit })
    }

    pub fn after(cursor: &PageCursor, limit: usize) -> Result<Self, ViewError> {
        Self::new(cursor.offset()?, limit)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor(String);

impl PageCursor {
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

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

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

#[derive(Debug)]
pub struct ResourcePage<'a> {
    pub resources: Vec<&'a ResourceView>,

    pub total_matching: usize,

    pub offset: usize,

    pub limit: usize,

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
    pub fn meta(&self) -> &SnapshotMeta {
        &self.meta
    }

    pub fn fingerprint(&self) -> &str {
        self.meta.fingerprint()
    }

    pub fn resources(&self) -> &[ResourceView] {
        &self.resources
    }

    pub fn resource(&self, id: &str) -> Option<&ResourceView> {
        let position = *self.resource_index.get(id)?;
        self.resources.get(position)
    }

    pub fn page<'a>(
        &'a self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage<'a>, ViewError> {
        page(&self.resources, filter, request)
    }
}
