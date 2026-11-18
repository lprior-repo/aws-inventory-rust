use super::{GraphClient, GraphError};

// Compare fixed-width decimal generations as strings: Lua numbers lose integer
// precision above 2^53. Legacy bare CLIENT IDs migrate on a quiescent upgrade.
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

pub(super) async fn claim(
    connection: &mut redis::aio::MultiplexedConnection,
    key: &str,
    generation: i64,
) -> Result<String, GraphError> {
    let owner: i64 = redis::cmd("CLIENT")
        .arg("ID")
        .query_async(connection)
        .await?;
    let token = format!("{generation:020}:{owner}");
    let claimed: bool = redis::cmd("EVAL")
        .arg(CLAIM)
        .arg(1)
        .arg(key)
        .arg(&token)
        .query_async(connection)
        .await?;
    if !claimed {
        return Err(GraphError::OwnershipLost);
    }
    Ok(token)
}

impl GraphClient {
    /// MULTI executes GRAPH.QUERY synchronously on FalkorDB's Redis thread.
    /// A superseding claim either precedes EXEC (WATCH aborts), or follows the
    /// completed native effect. Dropping/timing out this future cannot let an
    /// already-submitted old write execute after a newer owner's native writes.
    /// Missing ownership keys fail closed; durable generations prevent ABA.
    pub(super) async fn fenced_request(
        &mut self,
        query: String,
    ) -> Result<redis::Value, GraphError> {
        let ((), current): ((), Option<String>) = redis::pipe()
            .cmd("WATCH")
            .arg(&self.fence_key)
            .cmd("GET")
            .arg(&self.fence_key)
            .query_async(&mut self.connection)
            .await?;
        if current.as_deref() != Some(self.fence.as_str()) {
            return Err(GraphError::OwnershipLost);
        }
        let response: Option<(redis::Value,)> = redis::pipe()
            .atomic()
            .cmd("GRAPH.QUERY")
            .arg(&self.graph)
            .arg(query)
            .arg("TIMEOUT")
            .arg(10_000_u32)
            .query_async(&mut self.connection)
            .await?;
        response
            .map(|(value,)| value)
            .ok_or(GraphError::OwnershipLost)
    }
}
