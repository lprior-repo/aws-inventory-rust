use inventory_engine::sql::{SqlError, SqlResult, SqlValue};
use std::fmt::Write;
use std::io::{Cursor, Write as IoWrite};

const BYTE_PREVIEW: usize = 32;

#[cfg(test)]
thread_local! {
    static BYTE_PREVIEW_ALLOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn render(result: &SqlResult, max_bytes: usize) -> anyhow::Result<String> {
    let mut remaining = max_bytes;
    let headers = result
        .columns
        .iter()
        .map(|name| escape(name, &mut remaining))
        .collect::<anyhow::Result<Vec<String>>>()?;
    let rows = result
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|value| cell(value, &mut remaining))
                .collect()
        })
        .collect::<anyhow::Result<Vec<Vec<String>>>>()?;
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(|cell| cell.chars().count())
                .fold(header.chars().count(), usize::max)
        })
        .collect();
    let line = widths
        .iter()
        .try_fold(0usize, |total, width| total.checked_add(*width))
        .and_then(|total| total.checked_add(widths.len().saturating_sub(1).saturating_mul(3)))
        .and_then(|total| total.checked_add(1))
        .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
    let unicode = headers
        .iter()
        .chain(rows.iter().flatten())
        .try_fold(0usize, |total, cell| {
            total.checked_add(cell.len().saturating_sub(cell.chars().count()))
        })
        .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
    let size = line
        .checked_mul(rows.len().saturating_add(2))
        .and_then(|size| size.checked_add(unicode))
        .filter(|size| *size <= max_bytes)
        .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
    let mut output = String::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| SqlError::Allocation)?;
    append_row(&mut output, &headers, &widths)?;
    let divider: Vec<String> = widths.iter().map(|width| "-".repeat(*width)).collect();
    append_row(&mut output, &divider, &widths)?;
    rows.iter()
        .try_for_each(|row| append_row(&mut output, row, &widths))?;
    Ok(output)
}

fn append_row(output: &mut String, row: &[String], widths: &[usize]) -> std::fmt::Result {
    row.iter()
        .zip(widths)
        .enumerate()
        .try_for_each(|(index, (cell, width))| {
            if index > 0 {
                output.push_str(" | ");
            }
            output.push_str(cell);
            output.extend(std::iter::repeat_n(
                ' ',
                width.saturating_sub(cell.chars().count()),
            ));
            Ok::<_, std::fmt::Error>(())
        })?;
    output.write_char('\n')
}

fn cell(value: &SqlValue, remaining: &mut usize) -> anyhow::Result<String> {
    let mut buffer = Cursor::new([0_u8; 384]);
    match value {
        SqlValue::Null => buffer.write_all(b"NULL")?,
        SqlValue::Bool(value) => write!(&mut buffer, "{value}")?,
        SqlValue::Int(value) => write!(&mut buffer, "{value}")?,
        SqlValue::Real(value) => write!(&mut buffer, "{value}")?,
        SqlValue::Text(value) | SqlValue::Json(value) => return escape(value, remaining),
        SqlValue::Bytes(value) => {
            buffer.write_all(b"\\x")?;
            value
                .iter()
                .take(BYTE_PREVIEW)
                .try_for_each(|byte| write!(&mut buffer, "{byte:02x}"))?;
            if value.len() > BYTE_PREVIEW {
                write!(&mut buffer, "... ({} bytes)", value.len())?;
            }
        }
        SqlValue::Timestamp { seconds, micros } => {
            *remaining = remaining
                .checked_sub(27)
                .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
            return timestamp(*seconds, *micros);
        }
    }
    let length = usize::try_from(buffer.position())?;
    *remaining = remaining
        .checked_sub(length)
        .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
    let bytes = buffer
        .get_ref()
        .get(..length)
        .ok_or(SqlError::InvalidValue("formatted scalar length"))?;
    let text = std::str::from_utf8(bytes)?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(length)
        .map_err(|_| SqlError::Allocation)?;
    #[cfg(test)]
    if matches!(value, SqlValue::Bytes(_)) {
        BYTE_PREVIEW_ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
    }
    owned.push_str(text);
    Ok(owned)
}

fn escape(value: &str, remaining: &mut usize) -> anyhow::Result<String> {
    let escaped = value.chars().flat_map(char::escape_debug);
    let bytes = escaped.clone().try_fold(0usize, |bytes, character| {
        let width = character.len_utf8();
        *remaining = remaining
            .checked_sub(width)
            .ok_or(SqlError::ResultTooLarge { bound: "bytes" })?;
        bytes
            .checked_add(width)
            .ok_or(SqlError::ResultTooLarge { bound: "bytes" })
    })?;
    let mut text = String::new();
    text.try_reserve_exact(bytes)
        .map_err(|_| SqlError::Allocation)?;
    text.extend(escaped);
    Ok(text)
}

fn timestamp(seconds: i64, micros: u32) -> anyhow::Result<String> {
    let days = seconds
        .div_euclid(86_400)
        .checked_add(719_468)
        .ok_or(SqlError::InvalidValue("timestamp range; cast to text"))?;
    let era = days.div_euclid(146_097);
    let doe = days.saturating_sub(era.saturating_mul(146_097));
    let yoe = doe
        .saturating_sub(doe / 1460)
        .saturating_add(doe / 36_524)
        .saturating_sub(doe / 146_096)
        / 365;
    let doy = doe.saturating_sub(
        yoe.saturating_mul(365)
            .saturating_add(yoe / 4)
            .saturating_sub(yoe / 100),
    );
    let mp = doy.saturating_mul(5).saturating_add(2) / 153;
    let day = doy
        .saturating_sub(mp.saturating_mul(153).saturating_add(2) / 5)
        .saturating_add(1);
    let month = mp.saturating_add(if mp < 10 { 3 } else { -9 });
    let year = yoe
        .saturating_add(era.saturating_mul(400))
        .saturating_add(i64::from(month <= 2));
    if !(0..=9999).contains(&year) || micros >= 1_000_000 {
        return Err(
            SqlError::InvalidValue("timestamp outside RFC 3339 range; cast to text").into(),
        );
    }
    let clock = seconds.rem_euclid(86_400);
    let hour = clock / 3600;
    let minute = clock.rem_euclid(3600) / 60;
    let second = clock.rem_euclid(60);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{micros:06}Z"
    ))
}

#[cfg(test)]
mod tests;
