use super::{
    SEEDED, TOPIC,
    fixture::{Fixture, execute, topic_report},
    server::TestResult,
    store,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{Connection, PgConnection};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_select_only_postgres_role_when_command_runs_then_report_matches_without_inventory_writes()
-> TestResult {
    let run = with_role(async |url, role| {
        let mut connection = PgConnection::connect(url).await?;
        grant(&mut connection, role).await?;
        connection.close().await?;
        check(&role_url(url, role)).await
    })
    .await?;
    run.result
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_early_postgres_setup_error_when_returning_then_database_and_role_are_removed()
-> TestResult {
    let run = with_role(async |url, role| {
        let mut connection = PgConnection::connect(url).await?;
        let result = sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "GRANT SELECT ON inventory.missing TO {role}"
        )))
        .execute(&mut connection)
        .await;
        connection.close().await?;
        result?;
        Ok(())
    })
    .await?;
    let error = run.result.err().ok_or("expected a setup error")?;
    let Some(sqlx::Error::Database(database_error)) = error.downcast_ref::<sqlx::Error>() else {
        return Err("expected a SQLx database error".into());
    };
    assert_eq!(database_error.code().as_deref(), Some("42P01"));
    assert_eq!(
        database_error.message(),
        "relation \"inventory.missing\" does not exist"
    );
    let mut connection = PgConnection::connect(&admin_url()?).await?;
    let database_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&run.database)
            .fetch_one(&mut connection)
            .await?;
    let role_count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_roles WHERE rolname = $1")
        .bind(&run.role)
        .fetch_one(&mut connection)
        .await?;
    connection.close().await?;
    assert_eq!((database_count, role_count), (0, 0));
    Ok(())
}

struct RoleRun {
    database: String,
    role: String,
    result: TestResult,
}

async fn with_role(check: impl AsyncFn(&str, &str) -> TestResult) -> TestResult<RoleRun> {
    let admin = admin_url()?;
    let mut cluster = PgConnection::connect(&admin).await?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    let role = format!("cc_verify_{}", uuid::Uuid::new_v4().simple());
    let result = async {
        let row = store::resource("123456789012", "us-east-1", "AWS::SNS::Topic", "topic")?;
        store::postgres(database.url(), &[row]).await?;
        let mut connection = PgConnection::connect(database.url()).await?;
        let created = sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE ROLE {role} NOLOGIN")))
            .execute(&mut connection)
            .await;
        let closed = connection.close().await;
        created?;
        closed?;
        check(database.url(), &role).await
    }
    .await;
    let database_cleanup = database.finish().await;
    let role_cleanup = sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DROP ROLE IF EXISTS {role}")))
        .execute(&mut cluster)
        .await;
    let connection_cleanup = cluster.close().await;
    database_cleanup?;
    role_cleanup?;
    connection_cleanup?;
    Ok(RoleRun {
        database: name,
        role,
        result,
    })
}

fn role_url(url: &str, role: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}options%5Brole%5D={role}")
}

async fn grant(connection: &mut PgConnection, role: &str) -> TestResult {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA inventory TO {role}; GRANT SELECT ON inventory.resources TO {role}"
    )))
    .execute(connection)
    .await?;
    Ok(())
}

async fn check(url: &str) -> TestResult {
    let mut connection = PgConnection::connect(url).await?;
    let privileges: (bool, bool, bool, bool) = sqlx::query_as("SELECT has_table_privilege(current_user,'inventory.resources','SELECT'),has_table_privilege(current_user,'inventory.resources','INSERT'),has_table_privilege(current_user,'inventory.resources','UPDATE'),has_table_privilege(current_user,'inventory.resources','DELETE')")
        .fetch_one(&mut connection).await?;
    assert_eq!(privileges, (true, false, false, false));
    connection.close().await?;
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], Some(url)).await?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}
