use super::InventoryError;

pub const MAX_SQL_NAME_BYTES: usize = 63;
pub const MAX_DERIVED_SQL_NAME_BYTES: usize = 128;

pub fn shorten_sql_name(mut name: String) -> Result<String, InventoryError> {
    if name.len() > MAX_DERIVED_SQL_NAME_BYTES || !name.is_ascii() {
        return Err(InventoryError::SqlName);
    }
    if name.len() > MAX_SQL_NAME_BYTES {
        let hash = blake3::hash(name.as_bytes()).to_hex();
        let suffix = hash.get(..8).ok_or(InventoryError::SqlName)?;
        name.truncate(54);
        name.push('_');
        name.push_str(suffix);
    }
    Ok(name)
}
