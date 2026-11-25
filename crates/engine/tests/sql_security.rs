mod postgres_support;
mod sql_support;

use inventory_engine::sql::{Bounds, SqlError, run};
use postgres_support::TestResult;
use sql_support::Fixture;
use sqlx::{Connection, PgConnection};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_sql_text_functions_when_querying_then_the_validator_refuses_nested_actions()
-> TestResult {
    sql_support::run(nested_actions).await
}

async fn nested_actions(fixture: &Fixture) -> TestResult {
    for outer in [
        "query_to_xml",
        "query_to_xmlschema",
        "query_to_xml_and_xmlschema",
    ] {
        for inner in [
            "SELECT 1 FROM pg_advisory_lock(14)",
            "SELECT pg_terminate_backend(0)",
            "SELECT pg_cancel_backend(0)",
        ] {
            let sql = format!("SELECT {outer}('{inner}', false, false, '')::text");
            let error = run(&fixture.reader, &sql, Bounds::default())
                .await
                .err()
                .ok_or("expected an error")?;
            assert_eq!(
                error.to_string(),
                "SQL denied: SQL-executing functions are forbidden"
            );
            assert!(matches!(
                error,
                SqlError::QueryDenied("SQL-executing functions are forbidden")
            ));
        }
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_config_overrides_when_querying_then_the_validator_refuses_them() -> TestResult {
    sql_support::run(config_overrides).await
}

async fn config_overrides(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT set_config('statement_timeout', '0', true)",
        "SELECT pg_catalog.set_config('statement_timeout', '0', true)",
        "SELECT \"set_config\"('statement_timeout', '0', true)",
    ] {
        let error = run(&fixture.reader, sql, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: session configuration functions are forbidden"
        );
        assert!(matches!(
            error,
            SqlError::QueryDenied("session configuration functions are forbidden")
        ));
    }
    for sql in [
        "SET statement_timeout = 0",
        "RESET statement_timeout",
        "RESET ALL",
    ] {
        let error = run(&fixture.reader, sql, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: only SELECT, WITH, VALUES, EXPLAIN, TABLE or SHOW queries are permitted"
        );
    }
    let error = run(
        &fixture.reader,
        "SELECT pg_sleep(60)::text",
        Bounds {
            millis: 50,
            ..Bounds::default()
        },
    )
    .await
    .err()
    .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "SQL query exceeded time bound");
    assert!(matches!(error, SqlError::StatementTimeout));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_default_function_grants_when_xml_executes_sql_then_session_lock_survives_rollback()
-> TestResult {
    sql_support::run(xml_session_lock).await
}

async fn xml_session_lock(fixture: &Fixture) -> TestResult {
    let mut reader = PgConnection::connect(&fixture.reader_url).await?;
    let mut owner = PgConnection::connect(&fixture.owner_url).await?;
    let result = inspect_xml_lock(&mut reader, &mut owner).await;
    let reader_closed = reader.close().await;
    let owner_closed = owner.close().await;
    reader_closed?;
    owner_closed?;
    let (executable, xml_executed, retained) = result?;
    assert!(executable, "reader has the default PUBLIC EXECUTE grant");
    assert!(xml_executed, "query_to_xml executed the supplied SQL text");
    assert!(
        retained,
        "session advisory lock survives transaction rollback"
    );
    Ok(())
}

async fn inspect_xml_lock(
    reader: &mut PgConnection,
    owner: &mut PgConnection,
) -> TestResult<(bool, bool, bool)> {
    let executable: bool = sqlx::query_scalar(
        "SELECT has_function_privilege(current_user, 'pg_catalog.query_to_xml(text,boolean,boolean,text)', 'EXECUTE')"
    ).fetch_one(&mut *reader).await?;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *reader)
        .await?;
    sqlx::query("BEGIN READ ONLY").execute(&mut *reader).await?;
    let executed = sqlx::query_scalar::<_, bool>(
        "SELECT strpos(query_to_xml('SELECT 1 AS locked FROM pg_advisory_lock(14)', false, false, '')::text, '<locked>1</locked>') > 0"
    ).fetch_one(&mut *reader).await;
    let rolled_back = sqlx::query("ROLLBACK").execute(&mut *reader).await;
    let xml_executed = executed?;
    rolled_back?;
    let retained = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND locktype='advisory' AND objid=14 AND granted)"
    ).bind(pid).fetch_one(&mut *owner).await?;
    sqlx::query("SELECT pg_advisory_unlock(14)")
        .execute(&mut *reader)
        .await?;
    Ok((executable, xml_executed, retained))
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_privileges_when_creating_extensions_then_the_server_refuses_them()
-> TestResult {
    sql_support::run(extension_privileges).await
}

async fn extension_privileges(fixture: &Fixture) -> TestResult {
    let mut reader = PgConnection::connect(&fixture.reader_url).await?;
    let result = async {
        for extension in ["tablefunc", "dblink"] {
            let error = sqlx::query(sqlx::AssertSqlSafe(format!("CREATE EXTENSION {extension}")))
                .execute(&mut reader).await.err().ok_or("expected an error")?;
            let database = error.as_database_error().ok_or("expected a PostgreSQL error")?;
            assert_eq!(database.code().as_deref(), Some("42501"), "{extension}");
        }
        let query_to_json: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='pg_catalog' AND p.proname='query_to_json')"
        ).fetch_one(&mut reader).await?;
        assert!(!query_to_json, "supported PostgreSQL has no core query_to_json");
        Ok(())
    }.await;
    let closed = reader.close().await;
    closed?;
    result
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_nontransactional_side_effects_when_querying_then_the_validator_refuses_them()
-> TestResult {
    sql_support::run(nontransactional_effects).await
}

async fn nontransactional_effects(fixture: &Fixture) -> TestResult {
    assert_effect_refused(
        fixture,
        "SELECT pg_notify('inventory_sql_policy_probe', 'must_not_send')::text",
    )
    .await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_nontransactional_logical_messages_when_querying_then_the_validator_refuses_them()
-> TestResult {
    sql_support::run(logical_effect).await
}

async fn logical_effect(fixture: &Fixture) -> TestResult {
    assert_effect_refused(fixture, "SELECT pg_logical_emit_message(false, 'inventory_sql_policy_probe', 'must_not_emit')::text").await
}

async fn assert_effect_refused(fixture: &Fixture, sql: &str) -> TestResult {
    let error = run(&fixture.reader, sql, Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "SQL denied: notification and logical message functions are forbidden"
    );
    assert!(matches!(
        error,
        SqlError::QueryDenied("notification and logical message functions are forbidden")
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_role_when_calling_privileged_functions_then_postgres_denies_each()
-> TestResult {
    sql_support::run(privileged_functions).await
}

async fn privileged_functions(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT lo_import('/inventory-sql-no-such-file')::bigint",
        "SELECT lo_export(0, '/inventory-sql-no-such-file')",
        "SELECT pg_read_file('postgresql.conf')",
        "SELECT pg_read_binary_file('postgresql.conf')",
        "SELECT pg_ls_dir('.')",
        "SELECT pg_stat_file('.')::text",
        "SELECT pg_reload_conf()",
        "SELECT pg_switch_wal()::text",
    ] {
        assert_sqlstate(fixture, sql, "42501").await?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_read_only_transaction_when_mutating_large_objects_then_postgres_denies_each()
-> TestResult {
    sql_support::run(large_object_writes).await
}

async fn large_object_writes(fixture: &Fixture) -> TestResult {
    for sql in ["SELECT lo_create(0)::bigint", "SELECT lo_unlink(0)"] {
        assert_sqlstate(fixture, sql, "25006").await?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_sequence_grants_when_mutating_sequences_then_read_only_transaction_still_denies_each()
-> TestResult {
    sql_support::run(sequence_writes).await
}

async fn sequence_writes(fixture: &Fixture) -> TestResult {
    sqlx::query("CREATE SEQUENCE public.reader_side_effect_seq")
        .execute(&fixture.pool)
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE, UPDATE ON SEQUENCE public.reader_side_effect_seq TO \"{}\"",
        fixture.role
    )))
    .execute(&fixture.pool)
    .await?;
    for sql in [
        "SELECT nextval('public.reader_side_effect_seq')",
        "SELECT setval('public.reader_side_effect_seq', 7)",
    ] {
        assert_sqlstate(fixture, sql, "25006").await?;
    }
    Ok(())
}

async fn assert_sqlstate(fixture: &Fixture, sql: &str, expected: &str) -> TestResult {
    let error = run(&fixture.reader, sql, Bounds::default())
        .await
        .err()
        .ok_or("expected an error")?;
    let SqlError::Database(database) = error else {
        return Err(
            format!("expected PostgreSQL SQLSTATE {expected} for {sql}, got {error}").into(),
        );
    };
    let database = database
        .as_database_error()
        .ok_or("expected a PostgreSQL error")?;
    assert_eq!(database.code().as_deref(), Some(expected), "{sql}");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_sleep_variants_when_waiting_then_the_server_budget_cancels_each() -> TestResult {
    sql_support::run(sleep_variants).await
}

async fn sleep_variants(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT pg_sleep_for(interval '60 seconds')::text",
        "SELECT pg_sleep_until(current_timestamp + interval '60 seconds')::text",
    ] {
        let error = run(
            &fixture.reader,
            sql,
            Bounds {
                millis: 50,
                ..Bounds::default()
            },
        )
        .await
        .err()
        .ok_or("expected an error")?;
        assert_eq!(error.to_string(), "SQL query exceeded time bound", "{sql}");
        assert!(matches!(error, SqlError::StatementTimeout));
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_text_search_sql_executors_when_querying_then_the_validator_refuses_each()
-> TestResult {
    sql_support::run(text_search_executors).await
}

async fn text_search_executors(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT * FROM ts_stat('SELECT ''a''::tsvector FROM pg_advisory_lock(14)')",
        "SELECT ts_rewrite('a'::tsquery, 'SELECT ''a''::tsquery, ''b''::tsquery FROM pg_advisory_lock(14)')::text",
    ] {
        let error = run(&fixture.reader, sql, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: SQL-executing functions are forbidden"
        );
        assert!(matches!(
            error,
            SqlError::QueryDenied("SQL-executing functions are forbidden")
        ));
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_installed_sql_extensions_when_querying_then_each_entry_point_is_refused()
-> TestResult {
    sql_support::run(extension_executors).await
}

async fn extension_executors(fixture: &Fixture) -> TestResult {
    sqlx::query("CREATE EXTENSION tablefunc")
        .execute(&fixture.pool)
        .await?;
    sqlx::query("CREATE EXTENSION dblink")
        .execute(&fixture.pool)
        .await?;
    let installed: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT p.proname::text FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND (p.proname LIKE 'dblink%' OR p.proname IN ('crosstab','crosstab2','crosstab3','crosstab4','connectby')) ORDER BY 1"
    ).fetch_all(&fixture.pool).await?;
    let mut expected = [
        "connectby",
        "crosstab",
        "crosstab2",
        "crosstab3",
        "crosstab4",
        "dblink",
        "dblink_build_sql_delete",
        "dblink_build_sql_insert",
        "dblink_build_sql_update",
        "dblink_cancel_query",
        "dblink_close",
        "dblink_connect",
        "dblink_connect_u",
        "dblink_current_query",
        "dblink_disconnect",
        "dblink_error_message",
        "dblink_exec",
        "dblink_fdw_validator",
        "dblink_fetch",
        "dblink_get_connections",
        "dblink_get_notify",
        "dblink_get_pkey",
        "dblink_get_result",
        "dblink_is_busy",
        "dblink_open",
        "dblink_send_query",
    ];
    expected.sort_unstable();
    assert_eq!(installed, expected);
    for name in installed {
        let sql = format!("SELECT public.\"{name}\"()");
        let error = run(&fixture.reader, &sql, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: SQL-executing functions are forbidden",
            "{name}"
        );
        assert!(matches!(
            error,
            SqlError::QueryDenied("SQL-executing functions are forbidden")
        ));
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_backend_control_functions_when_querying_then_the_validator_refuses_each()
-> TestResult {
    sql_support::run(backend_controls).await
}

async fn backend_controls(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT pg_terminate_backend(0)",
        "SELECT pg_cancel_backend(0)",
    ] {
        let error = run(&fixture.reader, sql, Bounds::default())
            .await
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: advisory locks and backend control functions are forbidden"
        );
        assert!(matches!(
            error,
            SqlError::QueryDenied("advisory locks and backend control functions are forbidden")
        ));
    }
    Ok(())
}
