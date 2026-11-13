//! Bounded RESP command parsing and fixture response encoding.
use super::*;

pub(super) async fn line(reader: &mut BufReader<TcpStream>) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    (&mut *reader)
        .take(128)
        .read_until(b'\n', &mut bytes)
        .await?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() >= 128 || !bytes.ends_with(b"\r\n") {
        return Err(io::Error::other("invalid bounded RESP header"));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

pub(super) async fn command(reader: &mut BufReader<TcpStream>) -> io::Result<Option<Vec<Vec<u8>>>> {
    let Some(header) = line(reader).await? else {
        return Ok(None);
    };
    let count: usize = header
        .strip_prefix('*')
        .ok_or_else(|| io::Error::other("expected RESP array"))?
        .trim()
        .parse()
        .map_err(io::Error::other)?;
    if !(1..=16).contains(&count) {
        return Err(io::Error::other("command argument bound"));
    }
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let header = line(reader)
            .await?
            .ok_or_else(|| io::Error::other("missing bulk header"))?;
        let size: usize = header
            .strip_prefix('$')
            .ok_or_else(|| io::Error::other("expected bulk"))?
            .trim()
            .parse()
            .map_err(io::Error::other)?;
        if size > 2_105_344 {
            return Err(io::Error::other("command byte bound"));
        }
        let mut body = vec![0; size];
        reader.read_exact(&mut body).await?;
        let mut ending = [0; 2];
        reader.read_exact(&mut ending).await?;
        if ending != *b"\r\n" {
            return Err(io::Error::other("bad bulk ending"));
        }
        result.push(body);
    }
    Ok(Some(result))
}

pub(super) fn encode(value: &redis::Value, output: &mut Vec<u8>) -> io::Result<()> {
    match value {
        redis::Value::Nil => output.extend_from_slice(b"$-1\r\n"),
        redis::Value::Int(value) => output.extend_from_slice(format!(":{value}\r\n").as_bytes()),
        redis::Value::Okay => output.extend_from_slice(b"+OK\r\n"),
        redis::Value::SimpleString(value) => {
            output.extend_from_slice(format!("+{value}\r\n").as_bytes())
        }
        redis::Value::BulkString(bytes) => {
            output.extend_from_slice(format!("${}\r\n", bytes.len()).as_bytes());
            output.extend_from_slice(bytes);
            output.extend_from_slice(b"\r\n");
        }
        redis::Value::Array(values) => {
            output.extend_from_slice(format!("*{}\r\n", values.len()).as_bytes());
            for value in values {
                encode(value, output)?;
            }
        }
        _ => return Err(io::Error::other("unsupported test-proxy RESP response")),
    }
    if output.len() > 4_194_304 {
        return Err(io::Error::other("test proxy response bound"));
    }
    Ok(())
}
