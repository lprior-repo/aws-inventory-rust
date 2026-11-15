use super::*;

impl DatabaseSnapshot {
    pub async fn finish(self) -> Result<(), ViewError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn counts(&mut self) -> Result<DatabaseCounts, ViewError> {
        let resources = scalar_count(&mut self.tx, "SELECT count(*) FROM resources").await?;
        let accounts = scalar_count(&mut self.tx, "SELECT count(DISTINCT account) FROM resources").await?;
        let regions = scalar_count(&mut self.tx, "SELECT count(DISTINCT region) FROM resources").await?;
        let services = scalar_count(&mut self.tx, "SELECT count(DISTINCT substr(kind,instr(kind,':')+2,instr(substr(kind,instr(kind,':')+2),':')-1)) FROM resources").await?;
        Ok(DatabaseCounts { accounts, regions, services, resources })
    }

    pub async fn page(
        &mut self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage, ViewError> {
        validate_page_limit(request.limit)?;
        let total = count_filtered(&mut self.tx, filter).await?;
        let (resources, has_more) = decode_resources(&mut self.tx, filter, &request).await?;
        Ok(ResourcePage {
            next: next_resource_cursor(&resources, has_more),
            resources,
            total_matching: total,
        })
    }

    pub async fn edges_page(
        &mut self,
        node: &GraphNode,
        request: EdgePageRequest,
    ) -> Result<EdgePage, ViewError> {
        validate_page_limit(request.limit)?;
        let total = count_node_edges(&mut self.tx, node).await?;
        let (edges, has_more) = decode_edges(&mut self.tx, node, &request).await?;
        Ok(EdgePage {
            next: next_edge_cursor(&edges, has_more),
            edges,
            total_matching: total,
        })
    }

    pub async fn detail(&mut self, node: &GraphNode) -> Result<Option<StoredResource>, ViewError> {
        resource_key([&node.account, &node.region, &node.kind, &node.id])?;
        let row = sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources WHERE account=? AND region=? AND kind=? AND id=?")
            .bind(&node.account).bind(&node.region).bind(&node.kind).bind(&node.id)
            .fetch_optional(&mut *self.tx).await?;
        match row {
            Some(row) => Ok(Some(decode_row(&mut self.tx, row).await?)),
            None => Ok(None),
        }
    }

    pub async fn lod_summary(&mut self) -> Result<LodSummary, ViewError> {
        let resources = scalar_count(&mut self.tx, "SELECT count(*) FROM resources").await?;
        let edges = scalar_count(&mut self.tx, "SELECT count(*) FROM edges").await?;
        let connected = scalar_count(&mut self.tx, "SELECT count(*) FROM (SELECT from_account AS account,from_region AS region,from_kind AS kind,from_id AS id FROM edges UNION SELECT to_account,to_region,to_kind,to_id FROM edges)").await?;
        let isolated = resources.checked_sub(connected).ok_or(ViewError::StoredRow)?;
        Ok(LodSummary { resources, edges, connected_resources: connected, isolated_resources: isolated })
    }
}
