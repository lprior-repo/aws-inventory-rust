use super::*;
use std::sync::Mutex;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_establishment_failure_when_creating_then_database_and_role_are_removed()
-> TestResult {
    let admin = PgPool::connect(&admin_url()?).await?;
    let result = establishment_cleanup(&admin).await;
    admin.close().await;
    result
}

async fn establishment_cleanup(admin: &PgPool) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let name = database.name().to_owned();
    let role = format!("{name}_reader");
    let error = match create_in(database, nonstandard_reader).await {
        Err(error) => error,
        Ok(fixture) => {
            finish(fixture).await?;
            return Err("expected reader establishment to fail".into());
        }
    };
    let remaining = presence(admin, &name, &role).await;
    let cleanup = drop_role(&role).await;
    let remaining = after_cleanup(remaining, cleanup)?;
    assert_eq!(
        error.to_string(),
        "SQL requires PostgreSQL setting standard_conforming_strings = on"
    );
    assert_eq!(remaining, (false, false));
    Ok(())
}

async fn nonstandard_reader(pool: &PgPool, role: &str) -> TestResult {
    grant(pool, role).await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ALTER ROLE \"{role}\" SET standard_conforming_strings = off",
    )))
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_check_and_cleanup_failures_when_running_then_the_check_error_is_preserved()
-> TestResult {
    let admin = PgPool::connect(&admin_url()?).await?;
    let result = cleanup_priority(&admin).await;
    admin.close().await;
    result
}

async fn cleanup_priority(admin: &PgPool) -> TestResult {
    let observed = Mutex::new(None);
    let actual = run(async |fixture| {
        {
            let mut recorded = observed.lock().map_err(|_| "fixture name lock poisoned")?;
            *recorded = Some((fixture.database.name().to_owned(), fixture.role.clone()));
        }
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "CREATE SCHEMA \"{}\" AUTHORIZATION \"{}\"",
            fixture.role, fixture.role,
        )))
        .execute(admin)
        .await?;
        Err("fixture check failed".into())
    })
    .await;
    let (name, role) = observed
        .into_inner()
        .map_err(|_| "fixture name lock poisoned")?
        .ok_or("fixture check did not run")?;
    let remaining = presence(admin, &name, &role).await;
    let schema = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP SCHEMA IF EXISTS \"{role}\" CASCADE"
    )))
    .execute(admin)
    .await
    .map(|_| ())
    .map_err(Into::into);
    let role_cleanup = drop_role(&role).await;
    let remaining = after_cleanup(remaining, after_cleanup(schema, role_cleanup))?;
    assert_eq!(
        actual.err().ok_or("expected an error")?.to_string(),
        "fixture check failed"
    );
    assert_eq!(remaining, (false, true));
    Ok(())
}

async fn presence(admin: &PgPool, name: &str, role: &str) -> TestResult<(bool, bool)> {
    Ok(sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname=$1), EXISTS(SELECT 1 FROM pg_roles WHERE rolname=$2)",
    ).bind(name).bind(role).fetch_one(admin).await?)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_database_cleanup_failure_when_finishing_then_role_cleanup_still_runs() -> TestResult
{
    let admin = PgPool::connect(&admin_url()?).await?;
    let result = database_cleanup_failure(&admin).await;
    admin.close().await;
    result
}

async fn database_cleanup_failure(admin: &PgPool) -> TestResult {
    let parent = TestDatabase::create(&admin_url()?).await?;
    let parent_name = parent.name().to_owned();
    let database = TestDatabase::create(parent.url()).await?;
    let name = database.name().to_owned();
    let fixture = create_in(database, grant).await?;
    let role = fixture.role.clone();
    sqlx::query(sqlx::AssertSqlSafe(format!("DROP OWNED BY \"{role}\"")))
        .execute(&fixture.pool)
        .await?;
    parent.finish().await?;
    let finished = finish(fixture).await;
    let remaining = presence(admin, &name, &role).await;
    let database_cleanup = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)",
    )))
    .execute(admin)
    .await
    .map(|_| ())
    .map_err(Into::into);
    let role_cleanup = drop_role(&role).await;
    let remaining = after_cleanup(remaining, after_cleanup(database_cleanup, role_cleanup))?;
    let error = finished.err().ok_or("expected an error")?;
    let error = error
        .downcast_ref::<inventory_harness::HarnessError>()
        .ok_or("expected a harness error")?;
    let inventory_harness::HarnessError::Sqlx(sqlx::Error::Database(error)) = error else {
        return Err("expected a PostgreSQL database error".into());
    };
    assert_eq!(error.code().as_deref(), Some("3D000"));
    assert_eq!(
        error.message(),
        format!("database \"{parent_name}\" does not exist")
    );
    assert_eq!(remaining, (true, false));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_panicking_check_when_running_then_database_and_reader_role_are_removed() -> TestResult
{
    let admin = PgPool::connect(&admin_url()?).await?;
    let result = panic_cleanup(&admin).await;
    admin.close().await;
    result
}

async fn panic_cleanup(admin: &PgPool) -> TestResult {
    let observed = Mutex::new(None);
    let actual = AssertUnwindSafe(run(async |fixture| {
        {
            let mut recorded = observed.lock().map_err(|_| "fixture name lock poisoned")?;
            *recorded = Some((fixture.database.name().to_owned(), fixture.role.clone()));
        }
        let actual: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&fixture.pool)
            .await?;
        assert_eq!(actual, 2, "injected fixture assertion failure");
        Ok(())
    }))
    .catch_unwind()
    .await;
    let (name, role) = observed
        .into_inner()
        .map_err(|_| "fixture name lock poisoned")?
        .ok_or("fixture check did not run")?;
    let remaining = presence(admin, &name, &role).await;
    let cleanup = drop_role(&role).await;
    let remaining = after_cleanup(remaining, cleanup)?;
    let actual = actual.map_err(|_| "fixture panic escaped cleanup")?;
    assert_eq!(
        actual.err().ok_or("expected an error")?.to_string(),
        "SQL fixture check panicked: assertion `left == right` failed: injected fixture assertion failure\n  left: 1\n right: 2"
    );
    assert_eq!(remaining, (false, false));
    Ok(())
}
