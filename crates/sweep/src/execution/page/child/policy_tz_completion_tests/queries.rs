use super::*;

impl<'a, 'catalog> Scenario<'a, 'catalog> {
    pub(super) async fn drain(
        mut self,
        catalog: &RuntimeCatalog<'_>,
        service: &str,
        mut respond: impl FnMut(
            &CallablePlan<'_>,
            &Record,
            &http::Request<bytes::Bytes>,
            bool,
        ) -> TestResult<Json>,
    ) -> TestResult<(Self, BTreeMap<String, usize>)> {
        let mut seen = BTreeMap::new();
        while let Some(lease) = self.store.claim(&self.sweep, 1000, 10000).await? {
            let operation = lease.scope().operation().as_str();
            let lease_service = lease.scope().service().as_str();
            let plan = catalog
                .lookup(lease_service, operation)
                .ok_or_else(|| format!("seed plan absent: {lease_service}.{operation}"))?;
            assert!(
                plan.service_name() == service
                    || matches!(plan.service_name(), "cloudwatch-logs" | "firehose"),
                "unexpected cross-service fixture producer: {}",
                plan.service_name()
            );
            let (request, wire) = self.request(plan, &lease)?;
            let body = respond(plan, &request, &wire, lease.token().is_some())?;
            let count = seen.entry(operation.to_owned()).or_insert(0_usize);
            *count = count.saturating_add(1);
            assert!(
                seen.values().sum::<usize>() < 512,
                "fixture unexpectedly repeated work"
            );
            self.apply(catalog, plan, lease, &request, body).await?;
            self = self.reopen(catalog).await?;
        }
        Ok((self, seen))
    }

    pub(super) async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("inventory.db"))
                    .read_only(true),
            )
            .await?)
    }

    pub(super) async fn reopen(self, catalog: &RuntimeCatalog<'_>) -> TestResult<Self> {
        let Self {
            directory,
            store,
            sweep,
            account,
            home,
            consumers,
        } = self;
        drop(store);
        let store = BinaryStore::open_with_catalog(
            &directory.path().join("inventory.db"),
            catalog.raw_bytes(),
        )
        .await?;
        Ok(Self {
            directory,
            store,
            sweep,
            account,
            home,
            consumers,
        })
    }

    pub(super) async fn assert_ids(&self, kind: &str, expected: Vec<String>) -> TestResult {
        self.assert_ids_at(kind, "us-east-1", expected).await
    }

    pub(super) async fn assert_ids_at(
        &self,
        kind: &str,
        region: &str,
        expected: Vec<String>,
    ) -> TestResult {
        let reader = self.reader().await?;
        let actual: Vec<(String, String, String, String)> =
            sqlx::query_as("SELECT account,region,kind,id FROM resources WHERE kind=? ORDER BY id")
                .bind(kind)
                .fetch_all(&reader)
                .await?;
        reader.close().await;
        let mut expected: Vec<_> = expected
            .into_iter()
            .map(|id| {
                (
                    "123456789012".to_owned(),
                    region.to_owned(),
                    kind.to_owned(),
                    id,
                )
            })
            .collect();
        expected.sort();
        assert_eq!(actual, expected, "{kind} canonical ResourceKeys");
        Ok(())
    }

    pub(super) async fn assert_edges(
        &self,
        kind: &str,
        relation: &str,
        expected: Vec<(String, String, String)>,
    ) -> TestResult {
        self.assert_edges_at(kind, relation, "us-east-1", "us-east-1", expected)
            .await
    }

    pub(super) async fn assert_edges_at(
        &self,
        kind: &str,
        relation: &str,
        from_region: &str,
        to_region: &str,
        expected: Vec<(String, String, String)>,
    ) -> TestResult {
        self.assert_edge_regions(
            kind,
            relation,
            from_region,
            expected
                .into_iter()
                .map(|(from, kind, to)| (from, to_region.to_owned(), kind, to))
                .collect(),
        )
        .await
    }

    pub(super) async fn assert_edge_regions(
        &self,
        kind: &str,
        relation: &str,
        from_region: &str,
        expected: Vec<(String, String, String, String)>,
    ) -> TestResult {
        let reader = self.reader().await?;
        type EdgeRow = (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        );
        let mut actual: Vec<EdgeRow> =
            sqlx::query_as("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE from_kind=? AND relation=?")
                .bind(kind).bind(relation).fetch_all(&reader).await?;
        reader.close().await;
        let mut expected: Vec<_> = expected
            .into_iter()
            .map(|(from, to_region, to_kind, to)| {
                (
                    "123456789012".to_owned(),
                    from_region.to_owned(),
                    kind.to_owned(),
                    from,
                    "123456789012".to_owned(),
                    to_region.to_owned(),
                    to_kind,
                    to,
                    relation.to_owned(),
                    "reviewed-rule".to_owned(),
                )
            })
            .collect();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{kind} {relation} resolved DAG edges");
        Ok(())
    }
}
