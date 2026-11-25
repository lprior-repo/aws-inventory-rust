use crate::postgres_support::{TestResult, registry};
use futures_util::FutureExt;
use inventory_engine::{
    inventory::{DatabaseUrl, PgStore, PgWriteConfig},
    sql::ReaderUrl,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{ConnectOptions, Connection, PgConnection, PgPool, postgres::PgConnectOptions};
use std::{panic::AssertUnwindSafe, str::FromStr};

pub struct Fixture {
    pub pool: PgPool,
    pub reader: ReaderUrl,
    pub reader_url: String,
    pub owner_url: String,
    database: TestDatabase,
    store: PgStore,
    pub role: String,
}

pub async fn run(check: impl AsyncFn(&Fixture) -> TestResult) -> TestResult {
    let fixture = create().await?;
    let result = match AssertUnwindSafe(check(&fixture)).catch_unwind().await {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .map_or("non-string panic payload", |message| message);
            Err(format!("SQL fixture check panicked: {message}").into())
        }
    };
    let cleanup = finish(fixture).await;
    after_cleanup(result, cleanup)
}

async fn create() -> TestResult<Fixture> {
    create_in(TestDatabase::create(&admin_url()?).await?, grant).await
}

async fn create_in(
    database: TestDatabase,
    setup: impl AsyncFn(&PgPool, &str) -> TestResult,
) -> TestResult<Fixture> {
    let pool = PgPool::connect(database.url()).await?;
    let (registry, _) = registry(1, Some("sql_fixture"))?;
    let store = PgStore::create(
        &DatabaseUrl::parse(database.url())?,
        registry,
        PgWriteConfig::default(),
    )
    .await?;
    let role = format!("{}_reader", database.name());
    if role.len() > 63
        || !role
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("invalid generated reader role".into());
    }
    let created = sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'inventory-sql-test' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS")))
        .execute(&pool).await;
    let role_created = created.is_ok();
    let prepared = match created {
        Ok(_) => prepare_reader(&pool, database.url(), &role, setup).await,
        Err(error) => Err(error.into()),
    };
    match prepared {
        Ok((reader, reader_url)) => Ok(Fixture {
            owner_url: database.url().to_owned(),
            database,
            store,
            pool,
            role,
            reader,
            reader_url,
        }),
        Err(error) => {
            let cleanup =
                finish_resources(store, pool, database, role_created.then_some(role.as_str()))
                    .await;
            after_cleanup(Err(error), cleanup)
        }
    }
}

async fn prepare_reader(
    pool: &PgPool,
    owner_url: &str,
    role: &str,
    setup: impl AsyncFn(&PgPool, &str) -> TestResult,
) -> TestResult<(ReaderUrl, String)> {
    setup(pool, role).await?;
    let reader_url = PgConnectOptions::from_str(owner_url)?
        .username(role)
        .password("inventory-sql-test")
        .to_url_lossy()
        .to_string();
    let reader = ReaderUrl::establish(&reader_url).await?;
    Ok((reader, reader_url))
}

async fn grant(pool: &PgPool, role: &str) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA inventory TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT SELECT ON ALL TABLES IN SCHEMA inventory TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    Ok(())
}

async fn finish(fixture: Fixture) -> TestResult {
    finish_resources(
        fixture.store,
        fixture.pool,
        fixture.database,
        Some(&fixture.role),
    )
    .await
}

async fn finish_resources(
    store: PgStore,
    pool: PgPool,
    database: TestDatabase,
    role: Option<&str>,
) -> TestResult {
    store.close().await;
    pool.close().await;
    let database_result = database.finish().await.map_err(Into::into);
    let role_result = match role {
        Some(role) => drop_role(role).await,
        None => Ok(()),
    };
    after_cleanup(database_result, role_result)
}

async fn drop_role(role: &str) -> TestResult {
    let mut admin = PgConnection::connect(&admin_url()?).await?;
    let result = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP ROLE IF EXISTS \"{role}\""
    )))
    .execute(&mut admin)
    .await
    .map(|_| ())
    .map_err(Into::into);
    let closed = admin.close().await.map_err(Into::into);
    after_cleanup(result, closed)
}

fn after_cleanup<T>(result: TestResult<T>, cleanup: TestResult) -> TestResult<T> {
    result.and_then(|value| cleanup.map(|()| value))
}

#[cfg(test)]
mod tests;
