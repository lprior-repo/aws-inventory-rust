use inventory_engine::sql::{Bounds, Budget, MAX_SQL_BYTES, SqlError, validate};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_postgres_quotes_when_validating_then_semicolons_remain_data() -> TestResult {
    [
        "SELECT ';'",
        "SELECT 'a'';b'",
        "SELECT 1 AS \"a\"\";b\"",
        "SELECT $$a';b$$",
        "SELECT $tag$a;$$b$tag$",
        r"SELECT E'a\';b'",
        r"SELECT e'a\\'; -- tail",
        "/* outer /* inner ; */ still comment */ SELECT 1; -- tail",
        "SELECT ARRAY[1,2];",
        "SELECT 1 /* ; */; /* tail */",
    ]
    .into_iter()
    .try_for_each(validate)?;
    Ok(())
}

#[test]
fn given_identifier_dollar_suffix_when_validating_then_semicolon_ends_the_statement() -> TestResult
{
    validate("SELECT a$$")?;
    let error = validate("SELECT a$$; SELECT 1")
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "SQL denied: multiple SQL statements are forbidden"
    );
    assert!(matches!(
        error,
        SqlError::QueryDenied("multiple SQL statements are forbidden")
    ));
    Ok(())
}

#[test]
fn given_first_keywords_when_validating_then_only_read_prefixes_pass() -> TestResult {
    [
        "SELECT 1",
        "with x as (select 1) select * from x",
        "VALUES (1)",
        "EXPLAIN SELECT 1",
        "TABLE resources",
        "SHOW search_path",
    ]
    .into_iter()
    .try_for_each(validate)?;
    [
        "INSERT INTO x VALUES (1)",
        "UPDATE x SET n=1",
        "DELETE FROM x",
        "CREATE TABLE x(n int)",
        "COPY x TO PROGRAM 'true'",
        "SET ROLE writer",
        "DO $$BEGIN END$$",
        "CALL procedure()",
        "ALTER TABLE x ADD n int",
        "DROP TABLE x",
        "LOCK TABLE x",
        "VACUUM x",
        "PRAGMA table_info(x)",
        "ATTACH DATABASE 'other' AS other",
        "GRANT SELECT ON x TO reader",
        "REINDEX TABLE x",
        "MERGE INTO x USING y ON false WHEN MATCHED THEN DELETE",
        "SELECTED 1",
    ]
    .into_iter()
    .try_for_each(|sql| -> TestResult {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: only SELECT, WITH, VALUES, EXPLAIN, TABLE or SHOW queries are permitted"
        );
        Ok(())
    })?;
    Ok(())
}

#[test]
fn given_hostile_statements_when_validating_then_exact_errors_are_returned() -> TestResult {
    [
        "SELECT 1; SELECT 2",
        r"SELECT 'a\'; SELECT 1",
        "SELECT 1; /* hide */ DELETE FROM x",
        "SELECT 1; -- hide\nSELECT 2",
        "SELECT $$;$$; SELECT 2",
        "SELECT 1;;",
    ]
    .into_iter()
    .try_for_each(|sql| -> TestResult {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: multiple SQL statements are forbidden"
        );
        Ok(())
    })?;
    [
        "SELECT $tag$open",
        "SELECT E'open\\'",
        "SELECT /* open",
        "SELECT \"open",
    ]
    .into_iter()
    .try_for_each(|sql| -> TestResult {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: unterminated SQL literal or comment"
        );
        Ok(())
    })?;
    [
        String::new(),
        "SELECT \0".into(),
        "x".repeat(MAX_SQL_BYTES.saturating_add(1)),
    ]
    .into_iter()
    .try_for_each(|sql| -> TestResult {
        assert_eq!(
            validate(&sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: empty, oversized, or NUL-containing SQL"
        );
        Ok(())
    })?;
    Ok(())
}

#[test]
fn given_result_budgets_when_charging_then_each_bound_is_distinct() -> TestResult {
    let bounds = Bounds {
        rows: 1,
        bytes: 4,
        cells: 2,
        columns: 2,
        millis: 100,
    };
    let mut budget = Budget::new(bounds)?;
    budget.charge_columns(["a", "b"])?;
    budget.charge_row(2)?;
    budget.charge_bytes(2)?;
    assert_eq!(
        budget
            .charge_bytes(1)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "SQL result exceeds bytes bound"
    );
    assert_eq!(
        budget
            .charge_row(1)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "SQL result exceeds rows bound"
    );
    let mut cells = Budget::new(Bounds { rows: 2, ..bounds })?;
    cells.charge_row(2)?;
    assert!(matches!(
        cells.charge_row(1),
        Err(SqlError::ResultTooLarge { bound: "cells" })
    ));
    let mut columns = Budget::new(bounds)?;
    assert!(matches!(
        columns.charge_columns(["a", "b", "c"]),
        Err(SqlError::ResultTooLarge { bound: "columns" })
    ));
    Ok(())
}

#[test]
fn given_invalid_bounds_when_constructing_budget_then_limits_fail_closed() -> TestResult {
    let defaults = Bounds::default();
    assert_eq!(
        defaults,
        Bounds {
            rows: 10_000,
            bytes: 8_388_608,
            cells: 100_000,
            columns: 256,
            millis: 5_000
        }
    );
    for bounds in invalid_bounds(defaults) {
        assert_eq!(
            Budget::new(bounds)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            "SQL bounds must be positive and no greater than the defaults"
        );
    }
    let mut budget = Budget::new(defaults)?;
    budget.charge_bytes(1)?;
    assert_eq!(
        budget
            .charge_bytes(usize::MAX)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "SQL result exceeds bytes bound"
    );
    Ok(())
}

fn invalid_bounds(defaults: Bounds) -> impl Iterator<Item = Bounds> {
    [0, usize::MAX]
        .into_iter()
        .flat_map(move |limit| {
            [
                Bounds {
                    rows: limit,
                    ..defaults
                },
                Bounds {
                    bytes: limit,
                    ..defaults
                },
                Bounds {
                    cells: limit,
                    ..defaults
                },
                Bounds {
                    columns: limit,
                    ..defaults
                },
            ]
        })
        .chain([
            Bounds {
                millis: 0,
                ..defaults
            },
            Bounds {
                millis: u64::MAX,
                ..defaults
            },
        ])
}

#[test]
fn given_backend_control_identifiers_when_validating_then_effectful_calls_are_denied() -> TestResult
{
    for sql in [
        "SELECT pg_advisory_xact_lock(14)",
        "SELECT pg_catalog.PG_ADVISORY_LOCK_SHARED(14)",
        "SELECT pg_try_advisory_lock(14)",
        "SELECT pg_advisory_unlock_all()",
        "SELECT pg_advisory_unlock(14)",
        "SELECT pg_terminate_backend(pg_backend_pid())",
        "SELECT Pg_Cancel_Backend(42)",
        "WITH x AS (SELECT pg_cancel_backend(42)) SELECT * FROM x",
    ] {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: advisory locks and backend control functions are forbidden"
        );
    }
    for sql in [
        "SELECT 'pg_advisory_lock(14)'",
        "SELECT E'pg_cancel_backend(42)'",
        "SELECT $$pg_terminate_backend(42)$$",
        "SELECT $tag$pg_advisory_xact_lock(14)$tag$",
        "SELECT 1 AS \"harmless_alias\"",
        "SELECT 1 -- pg_advisory_lock(14)",
        "SELECT /* pg_terminate_backend(42) */ 1",
        "SELECT my_pg_cancel_backend, pg_cancel_backend_suffix, cafépg_advisory_lock FROM resources",
    ] {
        validate(sql)?;
    }
    Ok(())
}

#[test]
fn given_quoted_control_names_when_validating_then_names_and_aliases_are_denied() -> TestResult {
    for sql in [
        "SELECT \"pg_advisory_xact_lock\"(14)",
        "SELECT pg_catalog.\"pg_advisory_xact_lock\"(14)",
        "SELECT \"PG_ADVISORY_LOCK_SHARED\"(14)",
        "SELECT \"pg_try_advisory_lock\"(14)",
        "SELECT pg_catalog.\"pg_try_advisory_lock\"(14)",
        "SELECT \"pg_terminate_backend\"(pg_backend_pid())",
        "SELECT pg_catalog.\"pg_terminate_backend\"(pg_backend_pid())",
        "SELECT \"Pg_Cancel_Backend\"(42)",
        "SELECT pg_catalog.\"Pg_Cancel_Backend\"(42)",
        "SELECT 1 AS \"pg_cancel_backend\"",
    ] {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: advisory locks and backend control functions are forbidden",
            "{sql}"
        );
    }
    for sql in [
        "SELECT 1 AS \"pg_advisory_alias\"",
        "SELECT 1 AS \"pg_advisory\"\"suffix\"",
        "SELECT 1 AS \"harmless\"\"pg_cancel_backend\"",
        "SELECT 1 AS \"pg_advis\"\"ory_lock\"",
        "SELECT 1 AS \"pg_cancel_backend\"\"suffix\"",
    ] {
        validate(sql)?;
    }
    Ok(())
}

#[test]
fn given_unicode_identifiers_when_validating_then_all_names_are_refused() -> TestResult {
    for sql in [
        r#"SELECT U&"pg_\0061dvisory_xact_lock"(14)"#,
        r#"SELECT u&"pg_\0061dvisory_xact_lock"(14)"#,
        r#"SELECT U&"pg_!0061dvisory_xact_lock"(14) UESCAPE '!'"#,
        r#"SELECT U&"d\0061ta" FROM resources"#,
    ] {
        let error = validate(sql).err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "SQL denied: Unicode-escaped SQL identifiers are forbidden",
            "{sql}"
        );
        assert!(matches!(
            error,
            SqlError::QueryDenied("Unicode-escaped SQL identifiers are forbidden")
        ));
    }
    Ok(())
}

#[test]
fn given_unicode_strings_when_validating_then_control_names_remain_data() -> TestResult {
    validate(r"SELECT U&'pg_advisory_xact_lock; \0061''pg_cancel_backend'")?;
    validate(r"SELECT u&'pg_terminate_backend; \0061' UESCAPE '!'")?;
    Ok(())
}

#[test]
fn given_spaced_unicode_prefix_when_validating_then_operator_form_passes() -> TestResult {
    validate(r#"SELECT U & "x" FROM resources"#)?;
    validate(r#"SELECT u & "x" FROM resources"#)?;
    Ok(())
}

#[test]
fn given_empty_prefixes_when_validating_then_missing_query_is_explicit() -> TestResult {
    [" ", "-- only comment", "/* comment */", "/* unfinished"]
        .into_iter()
        .try_for_each(|sql| -> TestResult {
            assert_eq!(
                validate(sql).err().ok_or("expected an error")?.to_string(),
                "SQL denied: missing SQL query"
            );
            Ok(())
        })?;
    let sql = format!("SELECT 1{}", " ".repeat(MAX_SQL_BYTES.saturating_sub(8)));
    assert_eq!(sql.len(), MAX_SQL_BYTES);
    validate(&sql)?;
    Ok(())
}

#[test]
fn given_advisory_named_columns_when_validating_then_harmless_identifiers_pass() -> TestResult {
    validate("SELECT pg_advisory_note FROM resources")?;
    validate("SELECT \"pg_advisory_note\" FROM resources")?;
    validate("SELECT 1 AS \"pg_advisory_alias\"")?;
    validate("SELECT 1 AS \"pg_advisory\"\"suffix\"")?;
    Ok(())
}

#[test]
fn given_sql_text_entry_points_when_validating_then_exact_names_are_refused() -> TestResult {
    for sql in [
        "SELECT query_to_xml('SELECT pg_advisory_lock(14)', false, false, '')",
        "SELECT pg_catalog.QUERY_TO_XMLSCHEMA('SELECT 1', false, false, '')",
        "SELECT \"query_to_xml_and_xmlschema\"('SELECT 1', false, false, '')",
        "SELECT \"ts_stat\"('SELECT 1')",
        "SELECT pg_catalog.ts_rewrite('a', 'SELECT 1')",
        "SELECT public.\"crosstab\"('SELECT 1')",
        "SELECT dblink_send_query('connection', 'SELECT 1')",
    ] {
        assert_eq!(
            validate(sql).err().ok_or("expected an error")?.to_string(),
            "SQL denied: SQL-executing functions are forbidden",
            "{sql}"
        );
    }
    for sql in [
        "SELECT 'query_to_xml', E'ts_stat', $$dblink_exec$$",
        "SELECT 1 /* query_to_xmlschema('SELECT 1') */",
        "SELECT harmless_query_to_xml, query_to_xml_suffix FROM resources",
        "SELECT table_to_xml('resources', false, false, '')",
        "SELECT cursor_to_xml('named_cursor', 1, false, false, '')",
        "SELECT schema_to_xml('inventory', false, false, '')",
        "SELECT database_to_xml(false, false, '')",
    ] {
        validate(sql)?;
    }
    Ok(())
}

#[test]
fn given_config_and_notification_functions_when_validating_then_effectful_names_are_refused()
-> TestResult {
    for (name, reason) in [
        (
            "set_config",
            "session configuration functions are forbidden",
        ),
        (
            "pg_notify",
            "notification and logical message functions are forbidden",
        ),
        (
            "pg_logical_emit_message",
            "notification and logical message functions are forbidden",
        ),
    ] {
        for sql in [
            format!("SELECT {name}()"),
            format!("SELECT pg_catalog.\"{name}\"()"),
        ] {
            assert_eq!(
                validate(&sql).err().ok_or("expected an error")?.to_string(),
                format!("SQL denied: {reason}")
            );
        }
    }
    for sql in [
        "SELECT 'set_config', $$pg_notify$$",
        "SELECT set_config_suffix, harmless_pg_notify FROM resources",
    ] {
        validate(sql)?;
    }
    Ok(())
}
