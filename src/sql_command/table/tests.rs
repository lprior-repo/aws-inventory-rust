use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_typed_rows_when_rendering_then_columns_align_and_values_are_explicit() -> TestResult {
    let result = SqlResult {
        columns: vec!["n".into(), "value".into()],
        rows: vec![
            vec![SqlValue::Int(12), SqlValue::Null],
            vec![SqlValue::Int(-1), SqlValue::Bool(true)],
            vec![SqlValue::Real(1.5), SqlValue::Text("a\nb".into())],
            vec![SqlValue::Bytes(vec![0, 255]), SqlValue::Json("{}".into())],
        ],
    };
    assert_eq!(
        render(&result, 1000)?,
        "n      | value\n------ | -----\n12     | NULL \n-1     | true \n1.5    | a\\nb \n\\x00ff | {}   \n"
    );
    Ok(())
}

#[test]
fn given_timestamp_edges_when_rendering_then_rfc3339_is_utc() -> TestResult {
    assert_eq!(timestamp(0, 0)?, "1970-01-01T00:00:00.000000Z");
    assert_eq!(timestamp(-1, 999_999)?, "1969-12-31T23:59:59.999999Z");
    assert_eq!(
        timestamp(951_782_400, 123_456)?,
        "2000-02-29T00:00:00.123456Z"
    );
    assert_eq!(timestamp(946_684_800, 1)?, "2000-01-01T00:00:00.000001Z");
    assert_eq!(
        timestamp(253_402_300_799, 999_999)?,
        "9999-12-31T23:59:59.999999Z"
    );
    assert_eq!(
        timestamp(253_402_300_800, 0)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "invalid PostgreSQL value: timestamp outside RFC 3339 range; cast to text"
    );
    assert_eq!(
        timestamp(i64::MAX, 0)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "invalid PostgreSQL value: timestamp outside RFC 3339 range; cast to text"
    );
    assert_eq!(
        timestamp(-62_167_219_201, 0)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "invalid PostgreSQL value: timestamp outside RFC 3339 range; cast to text"
    );
    Ok(())
}

#[test]
fn given_long_bytes_when_rendering_then_hex_preview_is_capped() -> TestResult {
    let mut remaining = usize::MAX;
    assert_eq!(
        cell(&SqlValue::Bytes(vec![171; 32]), &mut remaining)?,
        format!("\\x{}", "ab".repeat(32))
    );
    let value = SqlValue::Bytes(vec![171; 33]);
    assert_eq!(
        cell(&value, &mut remaining)?,
        format!("\\x{}... (33 bytes)", "ab".repeat(32))
    );
    let result = SqlResult {
        columns: vec!["name".into()],
        rows: Vec::new(),
    };
    assert_eq!(render(&result, 10)?, "name\n----\n");
    assert_eq!(
        render(&result, 9)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "SQL result exceeds bytes bound"
    );
    Ok(())
}

#[test]
fn given_multibyte_and_quoted_text_when_rendering_then_widths_and_escapes_are_exact() -> TestResult
{
    let result = SqlResult {
        columns: vec!["café".into(), "quote".into()],
        rows: vec![
            vec![SqlValue::Text("é".into()), SqlValue::Text("a'b".into())],
            vec![SqlValue::Text("café".into()), SqlValue::Text("a\"b".into())],
        ],
    };
    let expected = "café | quote\n---- | -----\né    | a\\'b \ncafé | a\\\"b \n";
    assert_eq!(render(&result, expected.len())?, expected);
    assert_eq!(
        render(&result, expected.len().saturating_sub(1))
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "SQL result exceeds bytes bound"
    );
    Ok(())
}

#[test]
fn given_expanding_cells_when_rendering_then_byte_bound_stops_before_later_cells() -> TestResult {
    for value in ["\u{200b}".repeat(3), "\u{1}".repeat(5)] {
        let result = SqlResult {
            columns: vec!["a".into(), "b".into()],
            rows: vec![vec![
                SqlValue::Text(value),
                SqlValue::Timestamp {
                    seconds: 253_402_300_800,
                    micros: 0,
                },
            ]],
        };
        let error = render(&result, 16).err().ok_or("expected an error")?;
        assert_eq!(error.to_string(), "SQL result exceeds bytes bound");
    }
    Ok(())
}

#[test]
fn given_bytea_cells_at_cell_bound_when_byte_budget_exhausts_then_no_preview_allocates()
-> TestResult {
    let result = SqlResult {
        columns: vec!["bytes".into()],
        rows: (0..inventory_engine::sql::Bounds::default().cells)
            .map(|_| vec![SqlValue::Bytes(Vec::new())])
            .collect(),
    };
    BYTE_PREVIEW_ALLOCATIONS.with(|count| count.set(0));
    let error = render(&result, 5).err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "SQL result exceeds bytes bound");
    assert_eq!(BYTE_PREVIEW_ALLOCATIONS.with(std::cell::Cell::get), 0);
    Ok(())
}
