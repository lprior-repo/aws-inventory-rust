use super::*;
use inventory_core::record::FieldId;

pub(crate) fn lease(
    response: &Record,
    endpoints: &[FieldId],
    address: &[FieldId],
    minutes: &[FieldId],
    suffix: &str,
) -> Result<(String, Duration), SweepError> {
    let rows = endpoint_rows(response, endpoints)?;
    let mut selected = None;
    for row in &rows {
        let (address, ttl) = endpoint_row(row, address, minutes)?;
        if selected.is_none() {
            selected = Some((trusted_url(address, suffix)?, ttl));
        } else {
            trusted_host(address, suffix)?;
        }
    }
    selected.ok_or(SweepError::Operation("endpoint discovery returned no endpoints"))
}

fn endpoint_rows(response: &Record, endpoints: &[FieldId]) -> Result<Vec<Record>, SweepError> {
    let rows = record::item_records(
        response,
        &inventory_core::catalog::runtime::ItemPlan::Path(endpoints.to_vec().into_boxed_slice()),
    )?;
    if rows.is_empty() || rows.len() > MAX_ENDPOINTS {
        return Err(SweepError::Operation(
            "endpoint discovery returned invalid endpoint count",
        ));
    }
    Ok(rows)
}

fn endpoint_row<'a>(
    row: &'a Record,
    address: &[FieldId],
    minutes: &[FieldId],
) -> Result<(&'a str, Duration), SweepError> {
    let Some(ValueRef::Text(address)) = record::path_value(row.root(), address)? else {
        return Err(SweepError::Operation("discovered endpoint address is missing"));
    };
    let Some(ValueRef::Int(minutes)) = record::path_value(row.root(), minutes)? else {
        return Err(SweepError::Operation("discovered endpoint TTL is missing"));
    };
    Ok((address, lease_duration(minutes)?))
}

fn lease_duration(minutes: i64) -> Result<Duration, SweepError> {
    if !(1..=MAX_TTL_MINUTES).contains(&minutes) {
        return Err(SweepError::Operation("discovered endpoint TTL exceeds bounds"));
    }
    let seconds = u64::try_from(minutes)
        .map_err(|_| SweepError::Operation("discovered endpoint TTL is negative"))?
        .checked_mul(60)
        .ok_or(SweepError::Operation("discovered endpoint TTL overflow"))?;
    Ok(Duration::from_secs(seconds))
}

pub(super) fn trusted_url(address: &str, suffix: &str) -> Result<String, SweepError> {
    trusted_host(address, suffix)?;
    Ok(format!("https://{address}"))
}

fn trusted_host(address: &str, suffix: &str) -> Result<(), SweepError> {
    if address.len() > 253
        || !suffix.starts_with('.')
        || !address.ends_with(suffix)
        || address.len() <= suffix.len()
        || !address.split('.').all(trusted_label)
    {
        return Err(SweepError::Operation(
            "discovered endpoint violates trusted HTTPS host policy",
        ));
    }
    Ok(())
}

fn trusted_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
        && label.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
        })
}
