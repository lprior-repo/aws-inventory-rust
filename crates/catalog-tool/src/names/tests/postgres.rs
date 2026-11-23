use super::{Exceptions, MAX_SQL_NAME_BYTES, NameError, TestResult, render_names, sql_name};
use super::super::{SqlName, SqlNameReason, render::render_report};

#[test]
fn given_63_byte_name_when_derived_then_it_is_unchanged() -> TestResult {
    let kind = format!("A::{}", "B".repeat(61));
    let name = sql_name(&kind, &Exceptions::none())?;
    assert_eq!(name.as_str(), format!("a_{}", "b".repeat(61)));
    assert_eq!(name.as_str().len(), MAX_SQL_NAME_BYTES);
    Ok(())
}

#[test]
fn given_64_and_128_byte_names_when_derived_then_core_accepts_shortened_names() -> TestResult {
    for count in [62, 126] {
        let kind = format!("A::{}", "B".repeat(count));
        let name = sql_name(&kind, &Exceptions::none())?;
        assert_eq!(name.as_str().len(), MAX_SQL_NAME_BYTES);
        inventory_core::inventory::SqlName::parse(name.as_str())?;
    }
    Ok(())
}

#[test]
fn given_same_long_prefix_when_rendered_then_names_and_report_are_distinct() -> TestResult {
    let first = format!("AWS::{}CCCCCCCCCC", "B".repeat(50));
    let second = format!("AWS::{}DDDDDDDDDD", "B".repeat(50));
    let types = [first.as_str(), second.as_str()];
    let rows = render_names(&types, &Exceptions::none())?;
    let mut names = rows.iter().map(|(_, name)| name.as_str());
    let first = names.next().ok_or("first name absent")?;
    let second = names.next().ok_or("second name absent")?;
    assert_eq!(first.get(..54), second.get(..54));
    assert_ne!(first, second);
    let report = render_report(&types, &Exceptions::none())?;
    assert_eq!(report.lines().filter(|line| line.starts_with("long ")).count(), 2);
    assert!(report.ends_with("names total=2 long=2 distinct=2\n"));
    assert_eq!(report, render_report(&types, &Exceptions::none())?);
    Ok(())
}

#[test]
fn given_postgres_reserved_names_when_constructed_then_keywords_are_rejected() {
    for name in ["select", "authorization", "current_catalog", "current_role", "current_schema", "current_user", "session_user", "system_user"] {
        assert_eq!(SqlName::new(name.to_owned(), "Test::Name"), Err(NameError::InvalidSqlName {
            name: name.to_owned(), reason: SqlNameReason::ReservedKeyword,
        }));
    }
}

#[test]
fn given_prefix_ending_separator_when_shortened_then_core_accepts_hash_separator() -> TestResult {
    let kind = format!("A::{}::{}", "B".repeat(51), "C".repeat(10));
    let name = sql_name(&kind, &Exceptions::none())?;
    assert!(name.as_str().contains("__"));
    inventory_core::inventory::SqlName::parse(name.as_str())?;
    Ok(())
}

#[test]
fn given_colliding_names_when_reporting_then_collision_is_not_hidden() -> TestResult {
    let exceptions = Exceptions::from_entries(&[("X", "same"), ("Y", "same")])?;
    assert!(matches!(render_report(&["AWS::X", "AWS::Y"], &exceptions), Err(NameError::DuplicateSqlName { .. })));
    Ok(())
}
