use crate::{GraphError, cypher, response};
use redis::aio::MultiplexedConnection;

// Fixed-width strings preserve generation ordering above Lua's numeric precision.
const CLAIM: &str = r"
local current = redis.call('GET', KEYS[1])
if current and not string.match(current, '^%d+$') then
    local previous = string.match(current, '^(%d+):%d+$')
    if not previous or string.len(previous) ~= 20 then
        return redis.error_reply('invalid inventory projection ownership token')
    end
    if previous >= string.sub(ARGV[1], 1, 20) then return 0 end
end
redis.call('SET', KEYS[1], ARGV[1])
return 1
";

pub(crate) struct Session {
    pub(crate) connection: MultiplexedConnection,
    key: String,
    token: String,
}

impl Session {
    pub(crate) async fn claim(
        mut connection: MultiplexedConnection,
        graph: &str,
        generation: i64,
    ) -> Result<Self, GraphError> {
        if generation < 0 {
            return Err(GraphError::InvalidRow);
        }
        let owner: i64 = redis::cmd("CLIENT")
            .arg("ID")
            .query_async(&mut connection)
            .await?;
        let token = format!("{generation:020}:{owner}");
        let key = format!("aws-inventory:projection-owner:{graph}");
        let claimed: bool = redis::cmd("EVAL")
            .arg(CLAIM)
            .arg(1)
            .arg(&key)
            .arg(&token)
            .query_async(&mut connection)
            .await?;
        if !claimed {
            return Err(GraphError::OwnershipLost);
        }
        Ok(Self {
            connection,
            key,
            token,
        })
    }

    pub(crate) async fn request(
        &mut self,
        graph: &str,
        query: &str,
    ) -> Result<redis::Value, GraphError> {
        let ((), current): ((), Option<String>) = redis::pipe()
            .cmd("WATCH")
            .arg(&self.key)
            .cmd("GET")
            .arg(&self.key)
            .query_async(&mut self.connection)
            .await?;
        if current.as_deref() != Some(self.token.as_str()) {
            return Err(GraphError::OwnershipLost);
        }
        let response: Result<Option<(redis::Value,)>, redis::RedisError> = redis::pipe()
            .atomic()
            .cmd(cypher::COMMAND)
            .arg(graph)
            .arg(query)
            .arg("TIMEOUT")
            .arg(10_000_u32)
            .query_async(&mut self.connection)
            .await;
        let response = match response {
            Err(error)
                if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX)
                    && index_already_exists(&error) =>
            {
                return Ok(redis::Value::Nil);
            }
            result => result?,
        };
        response
            .map(|(value,)| value)
            .ok_or(GraphError::OwnershipLost)
    }
}

pub(crate) async fn clear_pages(
    graph: &super::falkor::FalkorDb,
    exists: &str,
    delete: &str,
) -> Result<(), GraphError> {
    let mut remaining = 10_000_usize;
    // Each successful step consumes one of the fixed cleanup-page budget.
    while remaining > 0 {
        let value = graph.request(exists).await?;
        if !has_row(&value)? {
            return Ok(());
        }
        graph.request(delete).await?;
        remaining = remaining.saturating_sub(1);
    }
    let value = graph.request(exists).await?;
    if !has_row(&value)? {
        Ok(())
    } else {
        Err(GraphError::Limits)
    }
}

fn has_row(value: &redis::Value) -> Result<bool, GraphError> {
    match response::rows(value, 1)? {
        [] => Ok(false),
        [redis::Value::Array(fields)] if fields.as_slice() == [redis::Value::Int(1)] => Ok(true),
        _ => Err(GraphError::Response),
    }
}

fn index_already_exists(error: &redis::RedisError) -> bool {
    error.clone().into_server_errors().is_some_and(|errors| {
        !errors.is_empty()
            && errors.iter().all(|(_, server)| {
                matches!(server.code(), "Attribute" | "ERR")
                    && matches!(
                        server.details(),
                        Some("'key' is already indexed")
                            | Some("\"Attribute\": 'key' is already indexed")
                    )
            })
    })
}

#[cfg(test)]
mod tests;
