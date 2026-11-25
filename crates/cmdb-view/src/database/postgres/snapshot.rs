use super::queries::{
    count_filtered, count_node_edges, fetch_node_edges, fetch_page, record_lengths, unsigned,
};
use super::*;

impl Snapshot {
    pub(in crate::database) async fn finish(self) -> Result<(), ViewError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub(in crate::database) async fn counts(&mut self) -> Result<DatabaseCounts, ViewError> {
        let row = sqlx::query("SELECT count(*) AS resources,count(DISTINCT account) AS accounts,count(DISTINCT region) AS regions,count(DISTINCT split_part(kind,'::',2)) AS services FROM inventory.resources")
            .fetch_one(&mut *self.tx).await?;
        Ok(DatabaseCounts {
            accounts: unsigned(&row, "accounts")?,
            regions: unsigned(&row, "regions")?,
            services: unsigned(&row, "services")?,
            resources: unsigned(&row, "resources")?,
        })
    }

    pub(in crate::database) async fn page(
        &mut self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage, ViewError> {
        validate_page_limit(request.limit)?;
        let total_matching = count_filtered(&mut self.tx, filter).await?;
        let mut rows = fetch_page(&mut self.tx, filter, &request).await?;
        let has_more = rows.len() > request.limit;
        rows.truncate(request.limit);
        let resources = decode_resources(self, rows).await?;
        Ok(ResourcePage {
            next: next_resource_cursor(&resources, has_more),
            resources,
            total_matching,
        })
    }

    pub(in crate::database) async fn page_rows(
        &mut self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourceRowPage, ViewError> {
        validate_page_limit(request.limit)?;
        let total_matching = count_filtered(&mut self.tx, filter).await?;
        let mut rows = fetch_page(&mut self.tx, filter, &request).await?;
        let has_more = rows.len() > request.limit;
        rows.truncate(request.limit);
        let lengths = record_lengths(self, &rows).await?;
        let (rows, truncated) = decode_rows(rows, &lengths)?;
        let tags_truncated = rows.iter().any(|row| row.tags_truncated);
        Ok(ResourceRowPage {
            next: next_row_cursor(&rows, has_more || truncated),
            rows,
            total_matching,
            truncated: truncated || tags_truncated,
        })
    }

    pub(in crate::database) async fn edges_page(
        &mut self,
        node: &GraphNode,
        request: EdgePageRequest,
    ) -> Result<EdgePage, ViewError> {
        validate_page_limit(request.limit)?;
        let total_matching = count_node_edges(&mut self.tx, node).await?;
        let mut edges = fetch_node_edges(&mut self.tx, node, &request).await?;
        let has_more = edges.len() > request.limit;
        edges.truncate(request.limit);
        Ok(EdgePage {
            next: next_edge_cursor(&edges, has_more),
            edges,
            total_matching,
        })
    }

    pub(in crate::database) async fn detail(
        &mut self,
        node: &GraphNode,
    ) -> Result<Option<StoredResource>, ViewError> {
        resource_key([&node.account, &node.region, &node.kind, &node.id])?;
        match queries::detail(&mut self.tx, node).await? {
            Some(row) => {
                let key = row_key(&row)?;
                let detail = queries::read_detail(self, &key).await?;
                Ok(Some(decode_resource(key, row, detail)?))
            }
            None => Ok(None),
        }
    }

    pub(in crate::database) async fn lod_summary(&mut self) -> Result<LodSummary, ViewError> {
        let row = sqlx::query("SELECT (SELECT count(*) FROM inventory.resources) AS resources,(SELECT count(*) FROM inventory.edges) AS edges,(SELECT count(*) FROM (SELECT from_account,from_region,from_kind,from_id FROM inventory.edges UNION SELECT to_account,to_region,to_kind,to_id FROM inventory.edges) connected) AS connected")
            .fetch_one(&mut *self.tx).await?;
        let resources = unsigned(&row, "resources")?;
        let connected_resources = unsigned(&row, "connected")?;
        Ok(LodSummary {
            resources,
            edges: unsigned(&row, "edges")?,
            connected_resources,
            isolated_resources: resources
                .checked_sub(connected_resources)
                .ok_or(ViewError::StoredRow)?,
        })
    }
}
