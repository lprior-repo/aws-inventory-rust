use super::{GraphClient, GraphError, QueryError};
use futures_util::{TryStreamExt, stream};

const MAX_CLEANUP_PAGES: usize = 10_000;

impl GraphClient {
    /// Page old graph data as well as current bounded tag snapshots. This also
    /// handles databases written before the per-resource tag limit existed.
    pub(super) async fn clear_pages(
        &mut self,
        existence_query: &str,
        deletion_query: &str,
    ) -> Result<(), GraphError> {
        stream::try_unfold(
            (self, MAX_CLEANUP_PAGES),
            |(client, remaining)| async move {
                let response = client.request(existence_query.to_owned()).await?;
                if !has_row(response)? {
                    return Ok(None);
                }
                let remaining = remaining
                    .checked_sub(1)
                    .ok_or(GraphError::Query(QueryError::InvalidResponse))?;
                client.send(deletion_query.to_owned()).await?;
                Ok::<_, GraphError>(Some(((), (client, remaining))))
            },
        )
        .try_for_each(|()| async { Ok(()) })
        .await
    }
}

fn has_row(response: redis::Value) -> Result<bool, GraphError> {
    let redis::Value::Array(parts) = response else {
        return Err(GraphError::Query(QueryError::InvalidResponse));
    };
    let Some(redis::Value::Array(rows)) = parts.get(1) else {
        return Err(GraphError::Query(QueryError::InvalidResponse));
    };
    match rows.as_slice() {
        [] => Ok(false),
        [redis::Value::Array(fields)] if fields.as_slice() == [redis::Value::Int(1)] => Ok(true),
        _ => Err(GraphError::Query(QueryError::InvalidResponse)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(rows: Vec<redis::Value>) -> redis::Value {
        redis::Value::Array(vec![redis::Value::Array(vec![]), redis::Value::Array(rows)])
    }

    #[test]
    fn cleanup_checks_one_bounded_sentinel_and_rejects_malformed_replies() {
        assert!(matches!(has_row(reply(vec![])), Ok(false)));
        assert!(matches!(
            has_row(reply(vec![redis::Value::Array(vec![redis::Value::Int(1)])])),
            Ok(true)
        ));
        assert!(has_row(redis::Value::Nil).is_err());
        assert!(has_row(reply(vec![redis::Value::Int(1)])).is_err());
        assert!(has_row(reply(vec![redis::Value::Array(vec![redis::Value::Int(0)])])).is_err());
    }
}
