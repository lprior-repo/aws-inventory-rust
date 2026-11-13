//! Pure bounded preparation of trusted collection pages.
use super::*;

pub(super) struct PreparedResource {
    pub(super) id: String,
    pub(super) account_id: String,
    pub(super) region: String,
    pub(super) kind: String,
    pub(super) name: Option<String>,
    pub(super) source: Source,
    pub(super) observed_at: i64,
    pub(super) json: String,
}
pub(super) struct PreparedEdge {
    pub(super) from_id: String,
    pub(super) to_id: String,
    pub(super) relation: String,
    pub(super) source: Source,
    pub(super) observed_at: i64,
    pub(super) json: String,
}
pub(super) struct PreparedPage {
    pub(super) scope: CollectionScope,
    pub(super) scope_key: String,
    pub(super) page_number: u32,
    pub(super) next_token: Option<String>,
    pub(super) fingerprint: Vec<u8>,
    pub(super) resources: Vec<PreparedResource>,
    pub(super) edges: Vec<PreparedEdge>,
}

pub(super) fn prepare_page(page: CollectionPage) -> Result<PreparedPage, StoreError> {
    validate_page(&page)?;
    let fingerprint = page_fingerprint(&page)?;
    let scope_key = page.scope.key()?;
    let resources = page
        .resources
        .into_iter()
        .map(|resource| {
            let json = bounded_json(&resource)?;
            Ok(PreparedResource {
                id: resource.id,
                account_id: resource.account_id,
                region: resource.region,
                kind: resource.kind,
                name: resource.name,
                source: resource.source,
                observed_at: resource.observed_at,
                json,
            })
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    let edges = page
        .edges
        .into_iter()
        .map(|edge| {
            let json = bounded_json(&edge)?;
            Ok(PreparedEdge {
                from_id: edge.from_id,
                to_id: edge.to_id,
                relation: edge.relation,
                source: edge.source,
                observed_at: edge.observed_at,
                json,
            })
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(PreparedPage {
        scope: page.scope,
        scope_key,
        page_number: page.page_number,
        next_token: page.next_token,
        fingerprint,
        resources,
        edges,
    })
}

pub(super) fn validate_scope(scope: &CollectionScope) -> Result<(), StoreError> {
    validate_identifier(&scope.account_id, "scope account")?;
    validate_identifier(&scope.region, "scope region")?;
    validate_identifier(&scope.collector, "scope collector")?;
    if let Some(parent) = &scope.parent_id {
        validate_identifier(parent, "scope parent")?;
    }
    Ok(())
}

pub(super) fn validate_page(page: &CollectionPage) -> Result<(), StoreError> {
    page.validate()?;
    for resource in &page.resources {
        if resource.source != page.scope.source {
            return Err(StoreError::InvalidInput(
                "resource provenance differs from scope",
            ));
        }
    }
    for edge in &page.edges {
        if edge.source != page.scope.source {
            return Err(StoreError::InvalidInput(
                "edge provenance differs from scope",
            ));
        }
    }
    Ok(())
}

pub(super) struct ByteBudget {
    pub(super) used: usize,
    pub(super) limit: usize,
}

impl ByteBudget {
    pub(super) fn new(limit: usize) -> Self {
        Self { used: 0, limit }
    }

    pub(super) fn charge<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), StoreError> {
        serde_json::to_writer(self, value).map_err(|_| StoreError::Limit("serialized output bytes"))
    }

    pub(super) fn add(&mut self, size: usize) -> io::Result<()> {
        let next = self
            .used
            .checked_add(size)
            .ok_or_else(|| io::Error::other("byte count overflow"))?;
        if next > self.limit {
            return Err(io::Error::other("serialized byte limit exceeded"));
        }
        self.used = next;
        Ok(())
    }
}

impl Write for ByteBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.add(bytes.len())?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct DigestWriter {
    pub(super) budget: ByteBudget,
    pub(super) digest: Sha256,
}

impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.budget.add(bytes.len())?;
        self.digest.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn page_fingerprint(page: &CollectionPage) -> Result<Vec<u8>, StoreError> {
    let mut writer = DigestWriter {
        budget: ByteBudget::new(MAX_PAGE_BYTES),
        digest: Sha256::new(),
    };
    serde_json::to_writer(&mut writer, page).map_err(|_| StoreError::Limit("page bytes"))?;
    Ok(writer.digest.finalize().to_vec())
}

pub(super) struct RecordWriter {
    pub(super) budget: ByteBudget,
    pub(super) bytes: Vec<u8>,
}

impl Write for RecordWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.budget.add(bytes.len())?;
        self.bytes
            .try_reserve(bytes.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn bounded_json<T: Serialize>(value: &T) -> Result<String, StoreError> {
    let mut writer = RecordWriter {
        budget: ByteBudget::new(MAX_RECORD_BYTES),
        bytes: Vec::new(),
    };
    serde_json::to_writer(&mut writer, value)?;
    String::from_utf8(writer.bytes)
        .map_err(|_| StoreError::InvalidInput("serialized JSON is not UTF-8"))
}
