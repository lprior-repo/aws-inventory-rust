use super::{TestDatabase, admin_url, database, names, prune};
use crate::HarnessError;
use sqlx::Connection;

async fn exists(admin: &str, name: &str) -> Result<bool, HarnessError> {
    let mut connection = database::connect(admin).await?;
    let present = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(name)
        .fetch_one(&mut connection)
        .await?;
    connection.close().await?;
    Ok(present)
}

#[test]
fn postgres_names_are_bounded_identifiers() -> Result<(), HarnessError> {
    let name = names::generate()?;
    assert!(name.as_str().len() <= 40);
    assert!(name.as_str().bytes().all(|byte| byte.is_ascii_lowercase()
        || byte.is_ascii_digit() || byte == b'_'));
    assert!(names::created_at(name.as_str()).is_some());
    assert!(names::created_at("inv_t_00000000_1_deadbeef;DROP").is_none());
    assert!(names::created_at("inventory").is_none());
    Ok(())
}

#[test]
fn postgres_url_preserves_socket_and_credentials() -> Result<(), HarnessError> {
    let url = database::database_url(
        "postgres://inv:p%40ss@localhost/postgres?host=%2Ftmp%2Fsocket&sslmode=disable",
        "inv_t_00000000_1_deadbeef",
    )?;
    let options: sqlx::postgres::PgConnectOptions = url.parse()?;
    assert_eq!(options.get_database(), Some("inv_t_00000000_1_deadbeef"));
    assert_eq!(options.get_socket().map(std::path::PathBuf::as_path), Some(std::path::Path::new("/tmp/socket")));
    assert_eq!(options.get_username(), "inv");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_create_connect_finish_removes_database() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    let mut connection = database::connect(database.url()).await?;
    let value: i32 = sqlx::query_scalar("SELECT 42").fetch_one(&mut connection).await?;
    assert_eq!(value, 42);
    database.finish().await?;
    assert!(!exists(&admin, &name).await?);
    drop(connection);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_simultaneous_databases_are_isolated() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    let (first, second) = tokio::try_join!(TestDatabase::create(&admin), TestDatabase::create(&admin))?;
    assert_ne!(first.name(), second.name());
    assert_ne!(first.url(), second.url());
    first.finish().await?;
    assert!(exists(&admin, second.name()).await?);
    second.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_drop_cleans_without_finish() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    drop(database);
    assert!(!exists(&admin, &name).await?);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_prune_removes_old_leaves_fresh() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    let old = TestDatabase::create_at(&admin, 0).await?;
    let fresh = TestDatabase::create(&admin).await?;
    assert!(prune(&admin, 3_600).await? >= 1);
    assert!(!exists(&admin, old.name()).await?);
    assert!(exists(&admin, fresh.name()).await?);
    old.finish().await?;
    fresh.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_unrelated_databases_survive_prune() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    prune(&admin, 3_600).await?;
    assert!(exists(&admin, "postgres").await?);
    Ok(())
}
