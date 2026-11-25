use super::*;

impl Scenario {
    pub async fn assert_resources(
        &self,
        catalog: &RuntimeCatalog<'_>,
        case: &cases::Case,
    ) -> TestResult {
        let resources = self.store.projection_resources(None, 10).await?;
        assert_eq!(resources.len(), 2);
        let expected = [case.id("first"), case.id("later")];
        assert_eq!(
            resources
                .iter()
                .map(|row| row.key.id.as_str())
                .collect::<Vec<_>>(),
            expected.iter().map(String::as_str).collect::<Vec<_>>()
        );
        for resource in resources {
            assert_eq!(resource.key.kind.as_str(), case.kind);
            assert_eq!(&resource.key.account, &self.account);
            assert_eq!(resource.key.region, Region::Regional(self.home.clone()));
            assert_eq!(
                self.store.projection_resource(&resource.key).await?,
                Some(resource)
            );
        }
        let reader = self.reader().await?;
        let keys: Vec<(String, String, String, String)> =
            sqlx::query_as("SELECT account,region,kind,id FROM resources ORDER BY id")
                .fetch_all(&reader)
                .await?;
        assert_eq!(
            keys,
            expected
                .iter()
                .map(|id| (
                    "123456789012".to_owned(),
                    "us-east-1".to_owned(),
                    case.kind.to_owned(),
                    id.clone()
                ))
                .collect::<Vec<_>>()
        );
        let schema = catalog
            .schemas()
            .iter()
            .find(|schema| schema.kind().as_str() == case.kind)
            .ok_or("IVS kind schema absent")?;
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT id,record FROM \"");
        query
            .push(schema.table().as_str().replace('"', "\"\""))
            .push("\" ORDER BY id");
        let rows: Vec<(String, Vec<u8>)> = query.build_query_as().fetch_all(&reader).await?;
        assert_eq!(rows.len(), 2);
        let fields = fixture::fields()?;
        for ((id, bytes), ordinal) in rows.into_iter().zip(["first", "later"]) {
            assert_eq!(id, case.id(ordinal));
            let row = Record::from_bytes(bytes)?;
            assert_eq!(
                fixture::json_value(ValueRef::Map(row.root()).to_value()?, &fields)?,
                case.row(ordinal),
                "every metadata descendant must survive SQLite, without content/secret additions"
            );
        }
        let tags: Vec<(String, String, String)> =
            sqlx::query_as("SELECT id,tag_key,tag_value FROM tags ORDER BY id,tag_key")
                .fetch_all(&reader)
                .await?;
        assert_eq!(
            tags,
            ["first", "later"]
                .into_iter()
                .flat_map(|ordinal| [
                    (case.id(ordinal), "owner".to_owned(), "inventory".to_owned()),
                    (case.id(ordinal), "page".to_owned(), ordinal.to_owned()),
                ])
                .collect::<Vec<_>>()
        );
        let states: Vec<(i64,)> = sqlx::query_as("SELECT state FROM work_queue")
            .fetch_all(&reader)
            .await?;
        assert_eq!(
            states.len(),
            1,
            "root must not schedule any descendant operations"
        );
        let outcomes: Vec<(String,)> = sqlx::query_as("SELECT outcome FROM scope_outcomes")
            .fetch_all(&reader)
            .await?;
        assert_eq!(outcomes, [("complete".to_owned(),)]);
        assert_eq!(
            self.store.projection_edges(None, 10).await?.edges,
            Vec::new(),
            "standalone roots must not invent ownership edges"
        );
        reader.close().await;
        Ok(())
    }
}
