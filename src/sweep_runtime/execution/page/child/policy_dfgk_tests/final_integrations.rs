use super::*;

mod channel_assertions;
mod channels;
mod compiled;
mod healthlake;
mod roots;

fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    compiled::catalog()
}

async fn reopen<'a, 'catalog>(
    scenario: Scenario<'a, 'catalog>,
) -> TestResult<Scenario<'a, 'catalog>> {
    let Scenario {
        directory,
        store,
        sweep,
        account,
        home,
        consumers,
    } = scenario;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.db")).await?;
    Ok(Scenario {
        directory,
        store,
        sweep,
        account,
        home,
        consumers,
    })
}

fn page(items: &str, rows: Vec<Json>, token: Option<&str>, token_name: &str) -> Json {
    let mut body = serde_json::Map::new();
    body.insert(items.to_owned(), Json::Array(rows));
    if let Some(token) = token {
        body.insert(token_name.to_owned(), Json::String(token.to_owned()));
    }
    Json::Object(body)
}

fn assert_wire_token(
    wire: &http::Request<bytes::Bytes>,
    expected: Option<&str>,
    name: &str,
) -> TestResult {
    if wire.method() == http::Method::GET {
        let query = wire.uri().query().map_or("", |query| query);
        match expected {
            Some(value) => assert!(
                query
                    .split('&')
                    .any(|pair| pair == format!("{name}={value}"))
            ),
            None => assert!(
                !query
                    .split('&')
                    .any(|pair| pair.starts_with(&format!("{name}=")))
            ),
        }
    } else {
        let body: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(body.get(name).and_then(Json::as_str), expected);
    }
    Ok(())
}

async fn saved(scenario: &Scenario<'_, '_>, kind: &str) -> TestResult<Vec<(String, Vec<u8>)>> {
    let reader = scenario.reader().await?;
    let rows = sqlx::query_as("SELECT id,record FROM resource_facets WHERE kind=? ORDER BY id")
        .bind(kind)
        .fetch_all(&reader)
        .await?;
    reader.close().await;
    Ok(rows)
}
