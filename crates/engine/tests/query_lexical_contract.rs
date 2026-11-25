use inventory_engine::store::{Store, StoreError};
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_readonly_sql_when_quoted_or_commented_then_exact_values_survive() -> TestResult {
    let store = Store::open(":memory:").await?;
    let cases: &[(&str, Value)] = &[
        ("SELECT 1 AS n", json!([{"n": 1}])),
        (" \t\r\nSELECT 1 AS n;\n-- trailing", json!([{"n": 1}])),
        (
            "/* leading ; */ SELECT ';' AS value",
            json!([{"value": ";"}]),
        ),
        ("SELECT 'a'';b' AS value", json!([{"value": "a';b"}])),
        (
            "SELECT '/* ; -- */' AS value",
            json!([{"value": "/* ; -- */"}]),
        ),
        ("SELECT 1 AS [semi;column]", json!([{"semi;column": 1}])),
        ("SELECT 1 AS \"semi;column\"", json!([{"semi;column": 1}])),
        ("SELECT 1 AS `semi;column`", json!([{"semi;column": 1}])),
        ("SELECT 1 AS \"a\"\";b\"", json!([{"a\";b": 1}])),
        ("SELECT 1 AS `a``;b`", json!([{"a`;b": 1}])),
        (
            "WITH value(n) AS (SELECT 1) SELECT n FROM value;",
            json!([{"n": 1}]),
        ),
        ("SELECT 'é;☃' AS value", json!([{"value": "é;☃"}])),
        ("SELECT 'end'", json!([{"'end'": "end"}])),
        ("SELECT 1 AS \"n\"/* tail */", json!([{"n": 1}])),
        ("SELECT 1 AS `n`-- tail\n;", json!([{"n": 1}])),
        ("-- leading ; '\nSELECT 1 AS n", json!([{"n": 1}])),
        ("/* lead\n; */ SELECT 1 AS n; /* tail */", json!([{"n": 1}])),
    ];
    for (sql, expected) in cases {
        assert_eq!(&store.query(sql, 10).await?, expected, "SQL: {sql:?}");
    }
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn given_hostile_sql_when_querying_then_exact_policy_errors_prevent_execution() -> TestResult
{
    let store = Store::open(":memory:").await?;
    let cases = [
        (
            "SELECT 1; SELECT 2",
            "multiple SQL statements are forbidden",
        ),
        ("SELECT 1;;", "multiple SQL statements are forbidden"),
        (
            "SELECT 1; /* trailer */ SELECT 2",
            "multiple SQL statements are forbidden",
        ),
        (
            "SELECT 1; -- trailer\nSELECT 2",
            "multiple SQL statements are forbidden",
        ),
        (
            "SELECT ';'; SELECT 2",
            "multiple SQL statements are forbidden",
        ),
        (
            "SELECT 1; -/* tail */",
            "multiple SQL statements are forbidden",
        ),
        (
            "SELECT 1; /-- tail",
            "multiple SQL statements are forbidden",
        ),
        ("SELECT 1; -", "multiple SQL statements are forbidden"),
        ("SELECT 1; /", "multiple SQL statements are forbidden"),
        ("SELECT 1; 'more'", "multiple SQL statements are forbidden"),
        ("SELECT 'a", "unterminated SQL literal or comment"),
        ("SELECT \"a", "unterminated SQL literal or comment"),
        ("SELECT `a", "unterminated SQL literal or comment"),
        ("SELECT [a", "unterminated SQL literal or comment"),
        ("SELECT 1 /* open", "unterminated SQL literal or comment"),
        ("SELECT 1; /* open", "unterminated SQL literal or comment"),
        ("", "empty, oversized, or NUL-containing SQL"),
        ("SELECT '\0'", "empty, oversized, or NUL-containing SQL"),
        (" \t\r\n", "missing SELECT or WITH query"),
        ("-- only comment", "missing SELECT or WITH query"),
        ("/* closed */", "missing SELECT or WITH query"),
        ("/* unterminated", "missing SELECT or WITH query"),
        ("SELECTED 1", "only SELECT or WITH queries are permitted"),
        (
            "WITHDRAW value",
            "only SELECT or WITH queries are permitted",
        ),
        (
            "DELETE FROM scans",
            "only SELECT or WITH queries are permitted",
        ),
        (
            "/* leading */ PRAGMA user_version",
            "only SELECT or WITH queries are permitted",
        ),
    ];
    for (sql, expected) in cases {
        assert!(
            matches!(store.query(sql, 10).await, Err(StoreError::QueryDenied(actual)) if actual == expected),
            "SQL: {sql:?}; expected: {expected}"
        );
    }
    assert_eq!(
        store.query("SELECT 7 AS alive", 1).await?,
        json!([{"alive": 7}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn given_unfinished_operators_when_querying_then_native_syntax_errors_remain_typed()
-> TestResult {
    let store = Store::open(":memory:").await?;
    for sql in ["SELECT 1 -", "SELECT 1 /"] {
        assert!(
            matches!(store.query(sql, 1).await, Err(StoreError::Sqlx(_))),
            "SQL: {sql:?}"
        );
    }
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn given_legacy_dialect_when_reading_quotes_then_select_and_with_keep_their_values()
-> TestResult {
    let store = Store::open(":memory:").await?;
    for sql in [
        "SELECT 1 AS `semi;column`",
        "SELECT 1 AS [semi;column]",
        "WITH row AS (SELECT 1 AS n) SELECT n AS `semi;column` FROM row",
    ] {
        assert_eq!(store.query(sql, 10).await?, json!([{"semi;column": 1}]));
    }
    assert_eq!(
        store.query("SELECT 1 AS \"pg_cancel_backend\"", 10).await?,
        json!([{"pg_cancel_backend": 1}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn given_postgres_only_prefixes_when_querying_legacy_then_they_are_denied() -> TestResult {
    let store = Store::open(":memory:").await?;
    for sql in [
        "VALUES (1)",
        "EXPLAIN SELECT 1",
        "TABLE resources",
        "SHOW search_path",
    ] {
        let error = store
            .query(sql, 10)
            .await
            .err()
            .ok_or("expected an error")?;
        match error {
            StoreError::QueryDenied(reason) => {
                assert_eq!(reason, "only SELECT or WITH queries are permitted")
            }
            error => return Err(format!("unexpected error: {error}").into()),
        }
    }
    store.close().await;
    Ok(())
}
