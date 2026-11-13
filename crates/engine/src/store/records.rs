//! Record decoding and bounded collection of export streams.
use super::*;

pub(super) fn decode_record<T: serde::de::DeserializeOwned>(
    row: SqliteRow,
) -> Result<T, StoreError> {
    let json: &str = row.try_get(0)?;
    if json.len() > MAX_RECORD_BYTES {
        return Err(StoreError::Limit("stored record bytes"));
    }
    Ok(serde_json::from_str(json)?)
}

pub(super) async fn collect_export<T: Serialize>(
    records: impl Stream<Item = Result<T, StoreError>>,
    limit: usize,
) -> Result<Vec<T>, StoreError> {
    let (result, _) = records
        .try_fold(
            (Vec::new(), ByteBudget::new(MAX_EXPORT_BYTES)),
            |(mut result, mut budget), value| {
                let operation = (|| {
                    charge_export(&mut budget, &value, result.len(), limit)?;
                    result.try_reserve(1)?;
                    result.push(value);
                    Ok((result, budget))
                })();
                future::ready(operation)
            },
        )
        .await?;
    Ok(result)
}

pub(super) fn charge_export<T: Serialize>(
    budget: &mut ByteBudget,
    value: &T,
    count: usize,
    limit: usize,
) -> Result<(), StoreError> {
    if count >= limit {
        return Err(StoreError::Limit("export row count; use streaming export"));
    }
    budget
        .charge(value)
        .map_err(|_| StoreError::Limit("export bytes; use streaming export"))
}
