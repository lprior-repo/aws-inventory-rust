use super::super::{MAX_SQL_NAME_BYTES, SqlName, SqlNameReason};
use super::{Exceptions, NameError, TestResult, render_names, sql_name, sql_name_used};

#[test]
fn exceptions_cannot_create_a_digit_initial_sql_identifier() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("AWS", "1")])?;
    assert_eq!(
        sql_name("AWS::EC2::Instance", &exceptions),
        Err(NameError::InvalidSqlName {
            name: "1_ec2_instance".to_owned(),
            reason: SqlNameReason::InvalidSyntax,
        })
    );
    Ok(())
}

#[test]
fn plain_names_reject_sqlites_reserved_namespace() -> TestResult {
    assert_eq!(
        sql_name("Sqlite::Thing", &Exceptions::none()),
        Err(NameError::InvalidSqlName {
            name: "sqlite_thing".to_owned(),
            reason: SqlNameReason::ReservedPrefix,
        })
    );
    Ok(())
}

#[test]
fn plain_names_reject_all_three_multiword_sqlite_keywords() -> TestResult {
    [
        ("Current::Date", "current_date"),
        ("Current::Time", "current_time"),
        ("Current::Timestamp", "current_timestamp"),
    ]
    .iter()
    .for_each(|(kind, name)| {
        assert_eq!(
            sql_name(kind, &Exceptions::none()),
            Err(NameError::InvalidSqlName {
                name: (*name).to_owned(),
                reason: SqlNameReason::ReservedKeyword,
            })
        );
    });
    Ok(())
}

#[test]
fn names_similar_to_reserved_sql_identifiers_remain_usable() -> TestResult {
    [
        ("Sqlite2::Thing", "sqlite2_thing"),
        ("Current::DateTime", "current_date_time"),
        ("Current::Timestamped", "current_timestamped"),
    ]
    .iter()
    .try_for_each(|(kind, expected)| {
        let name = sql_name(kind, &Exceptions::none())?;
        assert_eq!(name.as_str(), *expected);
        Ok(())
    })
}

#[test]
fn constructor_rejects_invalid_identifier_structure() -> TestResult {
    ["", "1_x", "_a", "a_", "a__b", "a_B", "a-b", "a_é", "a b"]
        .iter()
        .for_each(|name| {
            assert_eq!(
                SqlName::new((*name).to_owned(), "Test::Name"),
                Err(NameError::InvalidSqlName {
                    name: (*name).to_owned(),
                    reason: SqlNameReason::InvalidSyntax,
                })
            );
        });
    Ok(())
}

#[test]
fn constructor_requires_two_words_so_single_word_keywords_cannot_occur() -> TestResult {
    ["a", "select", "current"].iter().for_each(|name| {
        assert_eq!(
            SqlName::new((*name).to_owned(), "Test::Name"),
            Err(NameError::InvalidSqlName {
                name: (*name).to_owned(),
                reason: SqlNameReason::TooFewWords,
            })
        );
    });
    Ok(())
}

#[test]
fn constructor_itself_enforces_the_inclusive_sql_byte_limit() -> TestResult {
    let bounded = format!("a_{}", "b".repeat(126));
    let name = SqlName::new(bounded.clone(), "Test::Name")?;
    assert_eq!(name.as_str(), bounded);
    assert_eq!(
        SqlName::new(format!("{bounded}b"), "Test::Name"),
        Err(NameError::TooLong {
            type_name: "Test::Name".to_owned(),
            limit: MAX_SQL_NAME_BYTES,
        })
    );
    Ok(())
}

#[test]
fn used_and_catalog_naming_share_the_sql_safety_constructor() -> TestResult {
    let expected = NameError::InvalidSqlName {
        name: "sqlite_thing".to_owned(),
        reason: SqlNameReason::ReservedPrefix,
    };
    let mut used = vec![true];
    assert_eq!(
        sql_name_used("Sqlite::Thing", &Exceptions::none(), &mut used),
        Err(expected)
    );
    assert_eq!(used, Vec::<bool>::new());
    assert_eq!(
        render_names(&["Current::Time"], &Exceptions::none()),
        Err(NameError::InvalidSqlName {
            name: "current_time".to_owned(),
            reason: SqlNameReason::ReservedKeyword,
        })
    );
    Ok(())
}
