use super::{TestDatabase, admin_url, database, names, prune};
use crate::HarnessError;
use sqlx::postgres::PgSslMode;
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
    assert!(
        name.as_str()
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    );
    assert!(names::created_at(name.as_str()).is_some());
    assert!(names::created_at("inv_t_00000000_1_deadbeef;DROP").is_none());
    assert!(names::created_at("inventory").is_none());
    Ok(())
}

const FIXTURE: &str = "inv_t_00000000_1_deadbeef";

#[test]
fn postgres_url_retarget_keeps_credentials_tls_session() -> Result<(), HarnessError> {
    let admin = "postgres://inv:p%40ss@localhost:6543/postgres?host=%2Ftmp%2Fsocket&sslmode=verify-full&sslrootcert=%2Fetc%2Fca.pem&application_name=inv-test&options=-c%20search_path%3Dinv";
    let url = database::database_url(admin, FIXTURE)?;
    assert_eq!(
        url,
        "postgres://inv:p%40ss@localhost:6543/inv_t_00000000_1_deadbeef?host=%2Ftmp%2Fsocket&sslmode=verify-full&sslrootcert=%2Fetc%2Fca.pem&application_name=inv-test&options=-c%20search_path%3Dinv"
    );
    let options = database::options(&url)?;
    assert_eq!(options.get_database(), Some(FIXTURE));
    assert_eq!(options.get_username(), "inv");
    assert_eq!(options.get_port(), 6543);
    assert_eq!(
        options.get_socket().map(std::path::PathBuf::as_path),
        Some(std::path::Path::new("/tmp/socket"))
    );
    assert!(matches!(options.get_ssl_mode(), PgSslMode::VerifyFull));
    assert_eq!(options.get_application_name(), Some("inv-test"));
    assert_eq!(options.get_options(), Some("-c search_path=inv"));
    Ok(())
}

#[test]
fn postgres_url_retarget_drops_dbname_overrides() -> Result<(), HarnessError> {
    let with_path = "postgres://inv:p%40ss@localhost/postgres?dbname=postgres&db%6Eame=postgres&sslmode=disable";
    let url = database::database_url(with_path, FIXTURE)?;
    assert_eq!(
        url,
        "postgres://inv:p%40ss@localhost/inv_t_00000000_1_deadbeef?sslmode=disable"
    );
    assert_eq!(database::options(&url)?.get_database(), Some(FIXTURE));

    let without_path = "postgres://inv:p%40ss@localhost?dbname=postgres";
    let url = database::database_url(without_path, FIXTURE)?;
    assert_eq!(url, "postgres://inv:p%40ss@localhost/inv_t_00000000_1_deadbeef");
    assert_eq!(database::options(&url)?.get_database(), Some(FIXTURE));
    Ok(())
}

#[test]
fn postgres_url_retarget_refuses_names_that_are_not_fixtures() -> Result<(), HarnessError> {
    let admin = "postgres://inv:p%40ss@localhost/postgres?sslmode=disable";
    assert!(database::database_url(admin, "postgres").is_err());
    assert!(database::database_url(admin, "inv_t_00000000_1_deadbeef?dbname=postgres").is_err());
    Ok(())
}

#[test]
fn postgres_url_errors_do_not_echo_credentials() -> Result<(), HarnessError> {
    let admin = "postgres://inv:s3cret-pass@localhost/postgres?sslmode=bogus";
    let Err(error) = database::database_url(admin, FIXTURE) else {
        return Err(HarnessError::invalid("invalid sslmode unexpectedly accepted"));
    };
    assert!(!error.to_string().contains("s3cret-pass"));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_create_connect_finish_removes_database() -> Result<(), HarnessError> {
    let admin = admin_url()?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    let mut connection = database::connect(database.url()).await?;
    let value: i32 = sqlx::query_scalar("SELECT 42")
        .fetch_one(&mut connection)
        .await?;
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
    let (first, second) =
        tokio::try_join!(TestDatabase::create(&admin), TestDatabase::create(&admin))?;
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
