use super::*;

pub(super) fn validate_query(sql: &str, row_limit: usize) -> Result<(), StoreError> {
    if row_limit == 0 || row_limit > MAX_QUERY_ROWS {
        return Err(StoreError::Limit("query row limit"));
    }
    crate::sql::validate_with(sql, crate::sql::Dialect::Legacy).map_err(|error| match error {
        crate::sql::SqlError::QueryDenied(reason) => StoreError::QueryDenied(reason),
        _ => StoreError::QueryDenied("invalid SQL policy result"),
    })
}
