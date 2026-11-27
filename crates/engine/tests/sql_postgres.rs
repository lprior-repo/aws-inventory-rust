mod postgres_support;
mod sql_support;

use inventory_engine::sql::{Bounds, ReaderUrl, SqlError, SqlValue, run};
use postgres_support::TestResult;
use sql_support::Fixture;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_when_selecting_binary_types_then_exact_typed_values_return() -> TestResult {
    sql_support::run(typed_values).await
}

async fn typed_values(fixture: &Fixture) -> TestResult {
    let result = run(
        &fixture.reader,
        r#"SELECT NULL::text AS absent, true AS flag,
        -2::int2 AS small, 3::int4 AS medium, 9223372036854775807::int8 AS large,
        1.5::float4 AS single, -2.25::float8 AS double, 'hello'::text AS txt,
        'wide'::varchar AS varying, 'named'::name AS named, '\x00ff'::bytea AS bytes,
        '2000-01-01 00:00:00.123456+00'::timestamptz AS at,
        '{"n": 1}'::jsonb AS document, '{ "n":1, "n":2 }'::json AS original"#,
        Bounds::default(),
    )
    .await?;
    assert_eq!(
        result.columns,
        [
            "absent", "flag", "small", "medium", "large", "single", "double", "txt", "varying",
            "named", "bytes", "at", "document", "original"
        ]
    );
    assert_eq!(
        result.rows,
        vec![vec![
            SqlValue::Null,
            SqlValue::Bool(true),
            SqlValue::Int(-2),
            SqlValue::Int(3),
            SqlValue::Int(i64::MAX),
            SqlValue::Real(1.5),
            SqlValue::Real(-2.25),
            SqlValue::Text("hello".into()),
            SqlValue::Text("wide".into()),
            SqlValue::Text("named".into()),
            SqlValue::Bytes(vec![0, 255]),
            SqlValue::Timestamp {
                seconds: 946_684_800,
                micros: 123_456
            },
            SqlValue::Json("{\"n\": 1}".into()),
            SqlValue::Json("{ \"n\":1, \"n\":2 }".into())
        ]]
    );
    timestamp_and_empty(fixture).await
}

async fn timestamp_and_empty(fixture: &Fixture) -> TestResult {
    let before = run(
        &fixture.reader,
        "SELECT '1999-12-31 23:59:59.999999+00'::timestamptz",
        Bounds::default(),
    )
    .await?;
    assert_eq!(
        before.rows,
        vec![vec![SqlValue::Timestamp {
            seconds: 946_684_799,
            micros: 999_999
        }]]
    );
    let empty = run(
        &fixture.reader,
        "SELECT id FROM resources WHERE false",
        Bounds::default(),
    )
    .await?;
    assert_eq!(empty.columns, ["id"]);
    assert_eq!(empty.rows, Vec::<Vec<SqlValue>>::new());
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_when_attempting_writes_then_every_statement_is_refused() -> TestResult {
    sql_support::run(writes_refused).await
}

async fn writes_refused(fixture: &Fixture) -> TestResult {
    sqlx::query("INSERT INTO inventory.resources VALUES('a','r','k','seed',NULL,'\\x00',1,1)")
        .execute(&fixture.pool)
        .await?;
    sqlx::query("CREATE SEQUENCE inventory.sql_sequence")
        .execute(&fixture.pool)
        .await?;
    assert_resource_count(fixture, 1).await?;
    for statement in [
        "INSERT INTO resources SELECT * FROM resources",
        "UPDATE resources SET name='x'",
        "DELETE FROM resources",
        "CREATE TABLE inventory.bad(n int)",
        "COPY resources TO PROGRAM 'true'",
    ] {
        let error = run(&fixture.reader, statement, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: only SELECT, WITH, VALUES, EXPLAIN, TABLE or SHOW queries are permitted"
        );
    }
    for (statement, code, message) in [
        (
            "WITH gone AS (DELETE FROM resources RETURNING id) SELECT * FROM gone",
            "25006",
            "cannot execute SELECT in a read-only transaction",
        ),
        (
            "SELECT * INTO inventory.bad FROM resources",
            "25006",
            "cannot execute SELECT INTO in a read-only transaction",
        ),
        (
            "EXPLAIN ANALYZE DELETE FROM resources",
            "25006",
            "cannot execute DELETE in a read-only transaction",
        ),
        (
            "WITH i AS (INSERT INTO resources SELECT * FROM resources RETURNING id) SELECT * FROM i",
            "25006",
            "cannot execute SELECT in a read-only transaction",
        ),
        (
            "SELECT nextval('inventory.sql_sequence')",
            "42501",
            "permission denied for sequence sql_sequence",
        ),
    ] {
        assert_server_refusal(fixture, statement, code, message).await?;
    }
    assert_resource_count(fixture, 1).await
}

async fn assert_server_refusal(
    fixture: &Fixture,
    statement: &str,
    code: &str,
    message: &str,
) -> TestResult {
    let error = run(&fixture.reader, statement, Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    match error {
        SqlError::Database(sqlx::Error::Database(error)) => {
            assert_eq!(error.code().as_deref(), Some(code), "{statement}");
            assert_eq!(error.message(), message, "{statement}");
        }
        error => return Err(format!("unexpected refusal: {error}").into()),
    }
    Ok(())
}

async fn assert_resource_count(fixture: &Fixture, expected: i64) -> TestResult {
    let count = run(
        &fixture.reader,
        "SELECT count(*) FROM resources",
        Bounds::default(),
    )
    .await?;
    assert_eq!(count.rows, vec![vec![SqlValue::Int(expected)]]);
    Ok(())
}
use std::time::Duration;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_when_results_or_time_exceed_budget_then_typed_errors_return() -> TestResult {
    sql_support::run(bounds_and_types).await
}

async fn bounds_and_types(fixture: &Fixture) -> TestResult {
    for (statement, bounds, message) in [
        (
            "SELECT n FROM generate_series(1, 2) n",
            Bounds {
                rows: 1,
                ..Bounds::default()
            },
            "SQL result exceeds rows bound",
        ),
        (
            "SELECT repeat('x', 100) AS payload",
            Bounds {
                bytes: 50,
                ..Bounds::default()
            },
            "SQL result exceeds bytes bound",
        ),
        (
            "SELECT 1 AS a, 2 AS b, 3 AS c",
            Bounds {
                columns: 1,
                ..Bounds::default()
            },
            "SQL result exceeds columns bound",
        ),
        (
            "SELECT n, n FROM generate_series(1, 2) n",
            Bounds {
                rows: 2,
                cells: 3,
                ..Bounds::default()
            },
            "SQL result exceeds cells bound",
        ),
    ] {
        let error = run(&fixture.reader, statement, bounds)
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(error.to_string(), message);
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_server_time_budget_when_sleeping_then_server_cancels_before_transport_watchdog()
-> TestResult {
    sql_support::run(server_timeout).await
}

async fn server_timeout(fixture: &Fixture) -> TestResult {
    let bounds = Bounds {
        millis: 1_000,
        ..Bounds::default()
    };
    assert_eq!(
        run(
            &fixture.reader,
            "SELECT current_setting('statement_timeout')",
            bounds
        )
        .await?
        .rows,
        vec![vec![SqlValue::Text("1s".into())]]
    );
    let timeout = run(&fixture.reader, "SELECT pg_sleep(60)::text", bounds)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(timeout.to_string(), "SQL query exceeded time bound");
    assert!(matches!(timeout, SqlError::StatementTimeout));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_and_owner_when_querying_then_type_errors_and_privileges_are_exact()
-> TestResult {
    sql_support::run(types_and_privileges).await
}

async fn types_and_privileges(fixture: &Fixture) -> TestResult {
    let unsupported = run(
        &fixture.reader,
        "SELECT 1::numeric AS amount",
        Bounds::default(),
    )
    .await
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        unsupported.to_string(),
        "column \"amount\" has unsupported PostgreSQL type \"NUMERIC\"; cast to text"
    );
    let cast = run(
        &fixture.reader,
        "SELECT 1::numeric::text AS amount",
        Bounds::default(),
    )
    .await?;
    assert_eq!(cast.rows, vec![vec![SqlValue::Text("1".into())]]);
    let owner = ReaderUrl::establish(&fixture.owner_url)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        owner.to_string(),
        "SQL requires a reader role with no inventory write or administrative privileges"
    );
    assert!(matches!(owner, SqlError::ReaderRequired));
    let verified = ReaderUrl::establish(&fixture.reader_url).await?;
    assert_eq!(
        run(&verified, "SHOW search_path", Bounds::default())
            .await?
            .rows,
        vec![vec![SqlValue::Text("inventory".into())]]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_committed_page_and_open_writer_when_querying_then_reader_sees_only_committed_pages()
-> TestResult {
    sql_support::run(concurrent_pages).await
}

async fn concurrent_pages(fixture: &Fixture) -> TestResult {
    sqlx::query("INSERT INTO inventory.resources VALUES('a','r','k','page-1',NULL,'\\x00',1,1)")
        .execute(&fixture.pool)
        .await?;
    let mut writer = fixture.pool.begin().await?;
    let result: TestResult = async {
        sqlx::query(
            "INSERT INTO inventory.resources VALUES('a','r','k','page-2',NULL,'\\x00',1,1)",
        )
        .execute(&mut *writer)
        .await?;
        let during = run(
            &fixture.reader,
            "SELECT id FROM resources ORDER BY id",
            Bounds::default(),
        )
        .await?;
        if during.rows != vec![vec![SqlValue::Text("page-1".into())]] {
            return Err("reader observed an uncommitted page".into());
        }
        Ok(())
    }
    .await;
    if result.is_err() {
        let rolled_back = writer.rollback().await.map_err(Into::into);
        return result.and(rolled_back);
    }
    writer.commit().await?;
    let after = run(
        &fixture.reader,
        "SELECT id FROM resources ORDER BY id",
        Bounds::default(),
    )
    .await?;
    assert_eq!(
        after.rows,
        vec![
            vec![SqlValue::Text("page-1".into())],
            vec![SqlValue::Text("page-2".into())]
        ]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_catalog_columns_when_querying_then_existing_physical_types_are_supported()
-> TestResult {
    sql_support::run(physical_columns).await
}

async fn physical_columns(fixture: &Fixture) -> TestResult {
    sqlx::query("INSERT INTO inventory.sql_fixture(account,region,id,content_hash,first_seen,last_changed,record,nested,f_1,f_2,f_3,f_4,f_5,f_6) VALUES('a','r','id','\\x00',1,1,'\\x00','{\"x\": true}',false,42,1.5,'cidr','\\x00ff','2000-01-01 00:00:00+00')")
        .execute(&fixture.pool).await?;
    let result = run(
        &fixture.reader,
        "SELECT f_1,f_2,f_3,f_4,f_5,f_6,nested FROM sql_fixture",
        Bounds::default(),
    )
    .await?;
    assert_eq!(
        result.columns,
        ["f_1", "f_2", "f_3", "f_4", "f_5", "f_6", "nested"]
    );
    assert_eq!(
        result.rows,
        vec![vec![
            SqlValue::Bool(false),
            SqlValue::Int(42),
            SqlValue::Real(1.5),
            SqlValue::Text("cidr".into()),
            SqlValue::Bytes(vec![0, 255]),
            SqlValue::Timestamp {
                seconds: 946_684_800,
                micros: 0
            },
            SqlValue::Json("{\"x\": true}".into())
        ]]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_established_reader_when_write_privilege_is_added_then_execution_rechecks_it()
-> TestResult {
    sql_support::run(privilege_change).await
}

async fn privilege_change(fixture: &Fixture) -> TestResult {
    let result = run(
        &fixture.reader,
        "SELECT current_user::text",
        Bounds::default(),
    )
    .await?;
    let role = match result.rows.first().and_then(|row| row.first()) {
        Some(SqlValue::Text(role)) => role,
        _ => return Err("reader identity is missing".into()),
    };
    if !role
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("invalid reader role".into());
    }
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT UPDATE(name) ON inventory.resources TO \"{role}\""
    )))
    .execute(&fixture.pool)
    .await?;
    let error = run(&fixture.reader, "SELECT 1", Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "SQL requires a reader role with no inventory write or administrative privileges"
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_active_reader_when_writer_commits_then_reader_does_not_block_it() -> TestResult {
    sql_support::run(writer_during_read).await
}

async fn writer_during_read(fixture: &Fixture) -> TestResult {
    prepare_writer_barrier(fixture).await?;
    sqlx::query("INSERT INTO inventory.resources VALUES('a','r','k','page-1',NULL,'\\x00',1,1)")
        .execute(&fixture.pool)
        .await?;
    let reader = fixture.reader.clone();
    let reading =
        tokio::spawn(async move { run(&reader, CONCURRENT_READ, Bounds::default()).await });
    let committed = commit_during_read(fixture).await;
    let read = reading.await?;
    committed?;
    assert_eq!(
        read?.rows,
        vec![vec![SqlValue::Text("page-1".into()), SqlValue::Int(2)]]
    );
    assert_resource_count(fixture, 2).await
}

const CONCURRENT_READ: &str =
    "SELECT id, wait_for_writer() AS committed FROM resources ORDER BY id";

async fn prepare_writer_barrier(fixture: &Fixture) -> TestResult {
    sqlx::query(
        "CREATE FUNCTION inventory.wait_for_writer() RETURNS bigint LANGUAGE plpgsql AS $$
         DECLARE committed bigint;
         BEGIN
           FOR attempt IN 1..1000 LOOP
             SELECT count(*) INTO committed FROM inventory.resources;
             IF committed = 2 THEN RETURN committed; END IF;
             PERFORM pg_sleep(0.01);
           END LOOP;
           RAISE EXCEPTION 'writer did not commit';
         END $$",
    )
    .execute(&fixture.pool)
    .await?;
    Ok(())
}

async fn commit_during_read(fixture: &Fixture) -> TestResult {
    wait_for_sleeping_reader(fixture).await?;
    let mut writer = fixture.pool.begin().await?;
    sqlx::query("SET LOCAL lock_timeout = '100ms'")
        .execute(&mut *writer)
        .await?;
    sqlx::query("INSERT INTO inventory.resources VALUES('a','r','k','page-2',NULL,'\\x00',1,1)")
        .execute(&mut *writer)
        .await?;
    writer.commit().await?;
    Ok(())
}

async fn wait_for_sleeping_reader(fixture: &Fixture) -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let active: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE datname=current_database() AND state='active' AND wait_event='PgSleep' AND query=$1)")
                .bind(CONCURRENT_READ)
                .fetch_one(&fixture.pool).await?;
            if active {
                return Ok::<_, sqlx::Error>(());
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await??;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_broad_read_roles_when_verifying_then_establishment_and_execution_refuse_them()
-> TestResult {
    sql_support::run(broad_read_roles).await
}

async fn broad_read_roles(fixture: &Fixture) -> TestResult {
    for role in [
        "pg_read_all_data",
        "pg_read_server_files",
        "pg_monitor",
        "pg_read_all_stats",
    ] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "GRANT {role} TO \"{}\"",
            fixture.role
        )))
        .execute(&fixture.pool)
        .await?;
        let established = ReaderUrl::establish(&fixture.reader_url)
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            established.to_string(),
            "SQL requires a reader role with no inventory write or administrative privileges"
        );
        let executed = run(&fixture.reader, "SELECT 1", Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            executed.to_string(),
            "SQL requires a reader role with no inventory write or administrative privileges"
        );
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "REVOKE {role} FROM \"{}\"",
            fixture.role
        )))
        .execute(&fixture.pool)
        .await?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_nonstandard_strings_when_connecting_then_the_setting_is_named() -> TestResult {
    sql_support::run(nonstandard_strings).await
}

async fn nonstandard_strings(fixture: &Fixture) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ALTER ROLE \"{}\" SET standard_conforming_strings = off",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    let established = ReaderUrl::establish(&fixture.reader_url)
        .await
        .err()
        .ok_or("expected an error")?;
    let executed = run(&fixture.reader, "SELECT 1", Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    for error in [established, executed] {
        assert_eq!(
            error.to_string(),
            "SQL requires PostgreSQL setting standard_conforming_strings = on"
        );
        assert!(matches!(
            error,
            SqlError::SettingRequired {
                name: "standard_conforming_strings",
                required: "on"
            }
        ));
    }
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT UPDATE ON inventory.resources TO \"{}\"",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    assert_reader_refused(fixture).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_noninherited_monitor_membership_when_connecting_then_reader_is_refused() -> TestResult
{
    sql_support::run(noninherited_monitor).await
}

async fn noninherited_monitor(fixture: &Fixture) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT pg_monitor TO \"{}\" WITH INHERIT FALSE",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    assert_reader_refused(fixture).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_indirect_monitor_membership_when_connecting_then_reader_is_refused() -> TestResult {
    sql_support::run(indirect_monitor).await
}

async fn indirect_monitor(fixture: &Fixture) -> TestResult {
    let bridge = format!("{}_bridge", fixture.role);
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{bridge}\" NOLOGIN"
    )))
    .execute(&fixture.pool)
    .await?;
    let result = check_indirect_monitor(fixture, &bridge).await;
    let cleanup = sqlx::query(sqlx::AssertSqlSafe(format!("DROP ROLE \"{bridge}\"")))
        .execute(&fixture.pool)
        .await
        .map(|_| ())
        .map_err(Into::into);
    result.and(cleanup)
}

async fn check_indirect_monitor(fixture: &Fixture, bridge: &str) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT pg_monitor TO \"{bridge}\""
    )))
    .execute(&fixture.pool)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT \"{bridge}\" TO \"{}\" WITH INHERIT FALSE",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    assert_reader_refused(fixture).await
}

async fn assert_reader_refused(fixture: &Fixture) -> TestResult {
    let established = ReaderUrl::establish(&fixture.reader_url)
        .await
        .err()
        .ok_or("expected an error")?;
    let executed = run(&fixture.reader, "SELECT 1", Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    for error in [established, executed] {
        if !matches!(error, SqlError::ReaderRequired) {
            return Err(format!("expected reader refusal, received: {error}").into());
        }
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_pending_result_rows_when_budget_fails_then_the_bound_error_is_preserved()
-> TestResult {
    sql_support::run(pending_rows_bound).await
}

async fn pending_rows_bound(fixture: &Fixture) -> TestResult {
    let error = run(
        &fixture.reader,
        "SELECT n, repeat('x', 65536), pg_sleep(CASE WHEN n < 3 THEN 0 ELSE 60 END)::text FROM generate_series(1, 3) n",
        Bounds { rows: 1, ..Bounds::default() },
    ).await.err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "SQL result exceeds rows bound");
    assert!(matches!(error, SqlError::ResultTooLarge { bound: "rows" }));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_float4_when_selecting_then_shortest_decimal_becomes_the_real_value() -> TestResult {
    sql_support::run(float4_decimal).await
}

async fn float4_decimal(fixture: &Fixture) -> TestResult {
    let result = run(
        &fixture.reader,
        "SELECT 1.1::float4 AS single",
        Bounds::default(),
    )
    .await?;
    assert_eq!(result.columns, ["single"]);
    assert_eq!(result.rows, vec![vec![SqlValue::Real(1.1)]]);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_uuid_when_selecting_then_unsupported_type_names_the_column_and_type() -> TestResult {
    sql_support::run(unsupported_uuid).await
}

async fn unsupported_uuid(fixture: &Fixture) -> TestResult {
    let error = run(
        &fixture.reader,
        "SELECT '00000000-0000-0000-0000-000000000001'::uuid AS identifier",
        Bounds::default(),
    )
    .await
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "column \"identifier\" has unsupported PostgreSQL type \"UUID\"; cast to text"
    );
    assert!(matches!(
        error,
        SqlError::UnsupportedType { column, type_name }
            if column == "identifier" && type_name == "UUID"
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_unrelated_backend_when_reading_activity_then_its_query_and_state_are_hidden()
-> TestResult {
    sql_support::run(activity_visibility).await
}

async fn activity_visibility(fixture: &Fixture) -> TestResult {
    let mut owner = fixture.pool.acquire().await?;
    sqlx::query("SET application_name = 'sql-reader-visibility'")
        .execute(&mut *owner)
        .await?;
    sqlx::query("SELECT 'private fixture query'")
        .execute(&mut *owner)
        .await?;
    let result = run(
        &fixture.reader,
        "SELECT query, state FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND application_name='sql-reader-visibility'",
        Bounds::default(),
    ).await?;
    assert_eq!(result.columns, ["query", "state"]);
    assert_eq!(
        result.rows,
        vec![vec![
            SqlValue::Text("<insufficient privilege>".into()),
            SqlValue::Null
        ]]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_default_inherited_transitive_monitor_when_connecting_then_reader_is_refused()
-> TestResult {
    sql_support::run(default_inherit_monitor).await
}

async fn default_inherit_monitor(fixture: &Fixture) -> TestResult {
    let bridge = format!("{}_bridge", fixture.role);
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{bridge}\" NOLOGIN"
    )))
    .execute(&fixture.pool)
    .await?;
    let result = async {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "GRANT pg_monitor TO \"{bridge}\""
        )))
        .execute(&fixture.pool)
        .await?;
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "GRANT \"{bridge}\" TO \"{}\"",
            fixture.role
        )))
        .execute(&fixture.pool)
        .await?;
        assert_reader_refused(fixture).await
    }
    .await;
    let cleanup = sqlx::query(sqlx::AssertSqlSafe(format!("DROP ROLE \"{bridge}\"")))
        .execute(&fixture.pool)
        .await
        .map(|_| ())
        .map_err(Into::into);
    result.and(cleanup)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_wrong_refusal_error_when_checking_then_an_error_returns_without_panicking()
-> TestResult {
    sql_support::run(wrong_refusal_error).await
}

async fn wrong_refusal_error(fixture: &Fixture) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ALTER ROLE \"{}\" SET standard_conforming_strings = off",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    let error = assert_reader_refused(fixture)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "expected reader refusal, received: SQL requires PostgreSQL setting standard_conforming_strings = on"
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_statement_error_after_begin_when_reconnecting_then_reader_remains_usable()
-> TestResult {
    sql_support::run(statement_error_reconnect).await
}

async fn statement_error_reconnect(fixture: &Fixture) -> TestResult {
    let error = run(&fixture.reader, "SELECT 1 / 0", Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    let SqlError::Database(sqlx::Error::Database(error)) = error else {
        return Err(format!("expected a PostgreSQL statement error, received: {error}").into());
    };
    assert_eq!(error.code().as_deref(), Some("22012"));
    assert_eq!(error.message(), "division by zero");
    let result = run(&fixture.reader, "SELECT 42 AS n", Bounds::default()).await?;
    assert_eq!(result.columns, vec!["n"]);
    assert_eq!(result.rows, vec![vec![SqlValue::Int(42)]]);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_failure_with_open_writer_when_querying_then_writer_is_rolled_back()
-> TestResult {
    sql_support::run(writer_error_rollback).await
}

async fn writer_error_rollback(fixture: &Fixture) -> TestResult {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "REVOKE SELECT ON inventory.resources FROM \"{}\"",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    let error = concurrent_pages(fixture)
        .await
        .err()
        .ok_or("expected an error")?;
    let Some(SqlError::Database(database)) = error.downcast_ref::<SqlError>() else {
        return Err(format!("expected reader permission error, got {error}").into());
    };
    assert_eq!(
        database
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref(),
        Some("42501")
    );
    let (committed, uncommitted): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE id='page-1'), count(*) FILTER (WHERE id='page-2') FROM inventory.resources"
    ).fetch_one(&fixture.pool).await?;
    assert_eq!((committed, uncommitted), (1, 0));
    let open: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND state LIKE 'idle in transaction%'"
    ).fetch_one(&fixture.pool).await?;
    assert_eq!(open, 0);
    Ok(())
}
