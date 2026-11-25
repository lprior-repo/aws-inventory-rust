use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;
use std::time::Duration;

type PeerResult<T> = Result<T, String>;

pub(super) struct Peer {
    pub(super) endpoint: String,
    server: JoinHandle<PeerResult<Vec<String>>>,
}

impl Peer {
    pub(super) fn start(
        handler: impl FnMut(&str) -> PeerResult<String> + Send + 'static,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = format!("redis://{}", listener.local_addr()?);
        let server = std::thread::spawn(move || {
            let (socket, _) = listener.accept().map_err(|error| error.to_string())?;
            serve(socket, handler)
        });
        Ok(Self { endpoint, server })
    }

    pub(super) async fn finish(self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        tokio::task::spawn_blocking(move || {
            self.server
                .join()
                .map_err(|_| "fake graph peer panicked".to_owned())?
        })
        .await?
        .map_err(Into::into)
    }
}

fn serve(
    socket: TcpStream,
    mut handler: impl FnMut(&str) -> PeerResult<String>,
) -> PeerResult<Vec<String>> {
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    socket
        .set_nodelay(true)
        .map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(socket);
    let mut token = String::new();
    let mut pending = String::new();
    let mut queries = Vec::new();
    for _ in 0..200_000 {
        let Some(command) = command(&mut reader)? else {
            return Ok(queries);
        };
        let response = match command.first().map(String::as_str) {
            Some("CLIENT") if command.get(1).is_some_and(|argument| argument == "ID") => {
                ":7\r\n".into()
            }
            Some("CLIENT" | "SELECT" | "WATCH" | "MULTI") => "+OK\r\n".into(),
            Some("EVAL") => {
                token = command.last().ok_or("missing claim token")?.clone();
                ":1\r\n".into()
            }
            Some("GET") => bulk(&token),
            Some("GRAPH.QUERY" | "GRAPH.RO_QUERY") => {
                let query = command.get(2).ok_or("missing graph query")?;
                queries.push(query.clone());
                let response = handler(query)?;
                if command.first().is_some_and(|name| name == "GRAPH.QUERY") {
                    pending = response;
                    "+QUEUED\r\n".into()
                } else {
                    response
                }
            }
            Some("EXEC") => format!("*1\r\n{pending}"),
            other => return Err(format!("unexpected command {other:?}")),
        };
        reader
            .get_mut()
            .write_all(response.as_bytes())
            .map_err(|error| error.to_string())?;
    }
    Err("fake graph command budget exceeded".into())
}

fn command(reader: &mut BufReader<TcpStream>) -> PeerResult<Option<Vec<String>>> {
    let mut line = String::new();
    if reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?
        == 0
    {
        return Ok(None);
    }
    let count = line
        .trim()
        .strip_prefix('*')
        .ok_or("expected RESP array")?
        .parse::<usize>()
        .map_err(|error| error.to_string())?;
    if count > 16 {
        return Err("too many command arguments".into());
    }
    (0..count)
        .map(|_| {
            line.clear();
            reader
                .read_line(&mut line)
                .map_err(|error| error.to_string())?;
            let size = line
                .trim()
                .strip_prefix('$')
                .ok_or("expected RESP bulk string")?
                .parse::<usize>()
                .map_err(|error| error.to_string())?;
            if size > crate::MAX_QUERY_BYTES {
                return Err("oversized command".into());
            }
            let mut bytes = vec![0; size.checked_add(2).ok_or("command length overflow")?];
            reader
                .read_exact(&mut bytes)
                .map_err(|error| error.to_string())?;
            bytes.truncate(size);
            String::from_utf8(bytes).map_err(|error| error.to_string())
        })
        .collect::<PeerResult<Vec<_>>>()
        .map(Some)
}

pub(super) fn bulk(value: &str) -> String {
    format!("${}\r\n{value}\r\n", value.len())
}

pub(super) fn rows(rows: &[Vec<String>]) -> String {
    let mut reply = format!("*2\r\n*0\r\n*{}\r\n", rows.len());
    for row in rows {
        reply.push_str(&format!("*{}\r\n", row.len()));
        for value in row {
            reply.push_str(&bulk(value));
        }
    }
    reply
}

pub(super) fn empty() -> String {
    rows(&[])
}
