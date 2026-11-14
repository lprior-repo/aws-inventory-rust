use std::collections::BTreeSet;
use std::future::Future;

use futures_util::{StreamExt, future, stream};
use sha2::{Digest, Sha256};

use super::{DiscoveryClient, DiscoveryCoverage, MAX_METADATA_BYTES, data_error, ensure_running};
use crate::AwsScanError;

const MAX_TOKEN_BYTES: usize = 100_000;
const MAX_PAGE_ITEMS: usize = 1_000;

pub(super) trait Identified {
    fn identifier(&self) -> &str;
}

impl Identified for String {
    fn identifier(&self) -> &str {
        self
    }
}

pub(super) struct Page<T> {
    items: Vec<Result<T, AwsScanError>>,
    next_token: Option<String>,
}

impl<T> Page<T> {
    pub(super) fn map<R, F>(
        items: Vec<R>,
        next_token: Option<String>,
        map: F,
    ) -> Result<Self, AwsScanError>
    where
        F: Fn(R) -> Result<T, AwsScanError>,
    {
        if items.len() > MAX_PAGE_ITEMS {
            return Err(data_error("discovery response exceeds 1000 items per page"));
        }
        Ok(Self {
            items: items.into_iter().map(map).collect(),
            next_token,
        })
    }
}

pub(super) struct Listing<T> {
    pub(super) items: Vec<T>,
    pub(super) coverage: DiscoveryCoverage,
}

struct Cursor<T> {
    listing: Listing<T>,
    token: Option<String>,
    seen_tokens: BTreeSet<[u8; 32]>,
    seen_items: BTreeSet<[u8; 32]>,
    done: bool,
}

impl<T: Identified> Cursor<T> {
    fn new() -> Self {
        Self {
            listing: Listing {
                items: Vec::new(),
                coverage: DiscoveryCoverage::success(),
            },
            token: None,
            seen_tokens: BTreeSet::new(),
            seen_items: BTreeSet::new(),
            done: false,
        }
    }

    fn fail(&mut self, operation: &str, parent: Option<&str>, error: &AwsScanError) {
        self.listing.coverage.fail(operation, parent, error);
        self.done = true;
    }

    fn accept(
        &mut self,
        page: Page<T>,
        operation: &str,
        parent: Option<&str>,
        max_pages: u32,
        max_items: usize,
    ) {
        self.listing.coverage.pages = self.listing.coverage.pages.saturating_add(1);
        let item_count = u64::try_from(page.items.len()).unwrap_or(u64::MAX);
        let next_total = self.listing.coverage.items_seen.saturating_add(item_count);
        if next_total > u64::try_from(max_items).unwrap_or(0) {
            self.fail(
                operation,
                parent,
                &data_error("discovery item limit would be exceeded"),
            );
            return;
        }
        self.listing.coverage.items_seen = next_total;
        if self.listing.items.try_reserve(page.items.len()).is_err() {
            self.fail(
                operation,
                parent,
                &data_error("unable to reserve bounded discovery results"),
            );
            return;
        }
        for item in page.items {
            match item {
                Ok(item) => self.accept_item(item, operation, parent),
                Err(error) => self.listing.coverage.fail(operation, parent, &error),
            }
        }
        self.advance(page.next_token, operation, parent, max_pages, max_items);
    }

    fn accept_item(&mut self, item: T, operation: &str, parent: Option<&str>) {
        let id = item.identifier();
        if id.is_empty() || id.len() > MAX_METADATA_BYTES {
            self.listing.coverage.fail(
                operation,
                parent,
                &data_error("invalid discovery identity"),
            );
        } else if !self.seen_items.insert(Sha256::digest(id.as_bytes()).into()) {
            self.listing.coverage.fail(
                operation,
                parent,
                &data_error("discovery repeated an item identity"),
            );
        } else {
            self.listing.items.push(item);
        }
    }

    fn advance(
        &mut self,
        token: Option<String>,
        operation: &str,
        parent: Option<&str>,
        max_pages: u32,
        max_items: usize,
    ) {
        let Some(token) = token else {
            self.done = true;
            return;
        };
        let error = if token.is_empty() || token.len() > MAX_TOKEN_BYTES {
            Some("invalid or oversized discovery continuation token")
        } else if !self
            .seen_tokens
            .insert(Sha256::digest(token.as_bytes()).into())
        {
            Some("discovery continuation-token cycle")
        } else if self.listing.coverage.pages >= u64::from(max_pages) {
            Some("discovery page limit reached before terminal response")
        } else if self.listing.coverage.items_seen >= u64::try_from(max_items).unwrap_or(0) {
            Some("discovery item limit reached before terminal response")
        } else {
            None
        };
        if let Some(message) = error {
            self.fail(operation, parent, &data_error(message));
        } else {
            self.token = Some(token);
        }
    }
}

#[tracing::instrument(skip_all, fields(operation, parent))]
pub(super) async fn collect<T, F, Fut>(
    client: &DiscoveryClient<'_>,
    operation: &str,
    parent: Option<&str>,
    max_items: usize,
    fetch: F,
) -> Listing<T>
where
    T: Identified + Send,
    F: Fn(Option<String>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Page<T>, AwsScanError>> + Send,
{
    let pages = stream::unfold(Some(Cursor::new()), |state| {
        let fetch = &fetch;
        async move {
            let mut state = state?;
            match ensure_running(client.executor) {
                Err(error) => state.fail(operation, parent, &error),
                Ok(()) => match fetch(state.token.clone()).await {
                    Ok(page) => state.accept(page, operation, parent, client.max_pages, max_items),
                    Err(error) => state.fail(operation, parent, &error),
                },
            }
            if state.done {
                Some((Some(state.listing), None))
            } else {
                Some((None, Some(state)))
            }
        }
    });
    pages
        .filter_map(future::ready)
        .fold(None, |_, listing| future::ready(Some(listing)))
        .await
        .unwrap_or_else(|| {
            let mut coverage = DiscoveryCoverage::success();
            coverage.fail(
                operation,
                parent,
                &data_error("discovery driver produced no terminal outcome"),
            );
            Listing {
                items: Vec::new(),
                coverage,
            }
        })
}

#[cfg(test)]
pub(super) fn page<T>(items: Vec<Result<T, AwsScanError>>, next_token: Option<String>) -> Page<T> {
    Page { items, next_token }
}

#[cfg(test)]
mod property_tests;
