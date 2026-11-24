use super::{has_row, index_already_exists};
use redis::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn reply(rows: Vec<Value>) -> Value {
    Value::Array(vec![Value::Array(Vec::new()), Value::Array(rows)])
}

#[test]
fn cleanup_checks_one_bounded_sentinel_and_rejects_malformed_replies() -> TestResult {
    assert!(!has_row(&reply(Vec::new()))?);
    assert!(has_row(&reply(vec![Value::Array(vec![Value::Int(1)])]))?);
    [
        Value::Nil,
        reply(vec![Value::Int(1)]),
        reply(vec![Value::Array(vec![Value::Int(0)])]),
        reply(vec![
            Value::Array(vec![Value::Int(1)]),
            Value::Array(vec![Value::Int(1)]),
        ]),
        reply(vec![Value::Array(vec![Value::Int(1), Value::Int(1)])]),
    ]
    .into_iter()
    .try_for_each(|value| {
        let error = has_row(&value).err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "graph response is malformed or exceeds its row bound"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

fn server_error(reply: &[u8]) -> Result<redis::RedisError, Box<dyn std::error::Error>> {
    match redis::parse_redis_value(reply)? {
        Value::ServerError(error) => Ok(redis::RedisError::from(error)),
        _ => Err("expected a server error".into()),
    }
}

#[test]
fn already_indexed_reply_is_benign_only_for_that_exact_server_error() -> TestResult {
    assert!(index_already_exists(&server_error(
        b"-Attribute 'key' is already indexed\r\n"
    )?));
    assert!(index_already_exists(&server_error(
        b"-ERR \"Attribute\": 'key' is already indexed\r\n"
    )?));
    [
        b"-Attribute 'name' is already indexed\r\n".as_slice(),
        b"-ERR syntax error\r\n",
        b"-Other 'key' is already indexed\r\n",
        b"-ERR 'key' is already indexed extra\r\n",
    ]
    .into_iter()
    .try_for_each(|reply| {
        assert!(!index_already_exists(&server_error(reply)?));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
