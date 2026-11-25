mod postgres;
mod sqlite;

use crate::{GraphNode, ViewError};
pub use sqlite::{
    DatabaseCounts, EdgeCursor, EdgePage, EdgePageRequest, LodSummary, MAX_PAGE_LIMIT, PageCursor,
    PageRequest, ResourceFilter, ResourcePage, ResourceRow, ResourceRowPage, ResourceTag,
    ScopeFreshness, ScopeFreshnessReport, ScopeOutcomeCounts, StoredResource, StoredTagObservation,
    TagFilter,
};
use std::path::Path;

pub struct Database {
    backend: Backend,
}

enum Backend {
    Sqlite(sqlite::Database),
    Postgres(postgres::Database),
}

pub struct DatabaseSnapshot {
    backend: SnapshotBackend,
}

enum SnapshotBackend {
    Sqlite(sqlite::DatabaseSnapshot),
    Postgres(postgres::Snapshot),
}

pub fn is_postgres_url(target: &str) -> bool {
    target.split_once("://").is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("postgres") || scheme.eq_ignore_ascii_case("postgresql")
    })
}

impl Database {
    pub async fn open(target: &str) -> Result<Self, ViewError> {
        let backend = if is_postgres_url(target) {
            Backend::Postgres(postgres::Database::open(target).await?)
        } else {
            Backend::Sqlite(sqlite::Database::open(Path::new(target)).await?)
        };
        Ok(Self { backend })
    }

    pub async fn snapshot(&self) -> Result<DatabaseSnapshot, ViewError> {
        let backend = match &self.backend {
            Backend::Sqlite(database) => SnapshotBackend::Sqlite(database.snapshot().await?),
            Backend::Postgres(database) => SnapshotBackend::Postgres(database.snapshot().await?),
        };
        Ok(DatabaseSnapshot { backend })
    }

    pub async fn counts(&self) -> Result<DatabaseCounts, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.counts().await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn page(
        &self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.page(filter, request).await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn page_rows(
        &self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourceRowPage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.page_rows(filter, request).await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn detail(&self, node: &GraphNode) -> Result<Option<StoredResource>, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.detail(node).await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn edges_page(
        &self,
        node: &GraphNode,
        request: EdgePageRequest,
    ) -> Result<EdgePage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.edges_page(node, request).await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn lod_summary(&self) -> Result<LodSummary, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.lod_summary().await?;
        snapshot.finish().await?;
        Ok(result)
    }

    pub async fn scope_freshness(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<ScopeFreshnessReport, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let result = snapshot.scope_freshness(now_ms, limit).await?;
        snapshot.finish().await?;
        Ok(result)
    }
}

impl DatabaseSnapshot {
    pub async fn finish(self) -> Result<(), ViewError> {
        match self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.finish().await,
            SnapshotBackend::Postgres(snapshot) => snapshot.finish().await,
        }
    }

    pub async fn counts(&mut self) -> Result<DatabaseCounts, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.counts().await,
            SnapshotBackend::Postgres(snapshot) => snapshot.counts().await,
        }
    }

    pub async fn page(
        &mut self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.page(filter, request).await,
            SnapshotBackend::Postgres(snapshot) => snapshot.page(filter, request).await,
        }
    }

    pub async fn page_rows(
        &mut self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourceRowPage, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.page_rows(filter, request).await,
            SnapshotBackend::Postgres(snapshot) => snapshot.page_rows(filter, request).await,
        }
    }

    pub async fn detail(&mut self, node: &GraphNode) -> Result<Option<StoredResource>, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.detail(node).await,
            SnapshotBackend::Postgres(snapshot) => snapshot.detail(node).await,
        }
    }

    pub async fn edges_page(
        &mut self,
        node: &GraphNode,
        request: EdgePageRequest,
    ) -> Result<EdgePage, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.edges_page(node, request).await,
            SnapshotBackend::Postgres(snapshot) => snapshot.edges_page(node, request).await,
        }
    }

    pub async fn lod_summary(&mut self) -> Result<LodSummary, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.lod_summary().await,
            SnapshotBackend::Postgres(snapshot) => snapshot.lod_summary().await,
        }
    }

    pub async fn scope_freshness(
        &mut self,
        now_ms: i64,
        limit: usize,
    ) -> Result<ScopeFreshnessReport, ViewError> {
        match &mut self.backend {
            SnapshotBackend::Sqlite(snapshot) => snapshot.scope_freshness(now_ms, limit).await,
            SnapshotBackend::Postgres(snapshot) => snapshot.scope_freshness(now_ms, limit).await,
        }
    }
}
