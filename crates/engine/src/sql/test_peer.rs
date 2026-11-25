use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
    sync::oneshot,
    task::JoinHandle,
};

#[derive(Clone, Copy)]
pub(super) enum Stall {
    Rollback,
    Close,
}

pub(super) struct Peer {
    pub options: PgConnectOptions,
    pub stalled: oneshot::Receiver<()>,
    task: JoinHandle<io::Result<()>>,
}

impl Peer {
    pub async fn start(stall: Stall) -> io::Result<Self> {
        let socket = TcpSocket::new_v4()?;
        socket.set_recv_buffer_size(1024)?;
        socket.bind(SocketAddr::from(([127, 0, 0, 1], 0)))?;
        let listener = socket.listen(1)?;
        let options = PgConnectOptions::new()
            .host("127.0.0.1")
            .port(listener.local_addr()?.port())
            .username("reader")
            .database("inventory")
            .ssl_mode(PgSslMode::Disable);
        let (notify, stalled) = oneshot::channel();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            serve(stream, stall, notify).await
        });
        Ok(Self {
            options,
            stalled,
            task,
        })
    }

    pub async fn finish(mut self) -> io::Result<()> {
        self.task.abort();
        match (&mut self.task).await {
            Ok(result) => result,
            Err(error) if error.is_cancelled() => Ok(()),
            Err(error) => Err(io::Error::other(error)),
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(mut stream: TcpStream, stall: Stall, notify: oneshot::Sender<()>) -> io::Result<()> {
    let _startup = body(&mut stream).await?;
    message(&mut stream, b'R', &0u32.to_be_bytes()).await?;
    message(&mut stream, b'Z', b"I").await?;
    let tag = stream.read_u8().await?;
    let begin = body(&mut stream).await?;
    if tag != b'Q' || begin != b"BEGIN TRANSACTION READ ONLY\0" {
        return Err(invalid("expected read-only BEGIN"));
    }
    message(&mut stream, b'C', b"BEGIN\0").await?;
    message(&mut stream, b'Z', b"T").await?;
    if matches!(stall, Stall::Close) {
        if stream.read_u8().await? != b'Q' {
            return Err(invalid("expected simple query payload before close"));
        }
    } else {
        let mut command = Command::Result;
        loop {
            let tag = stream.read_u8().await?;
            let bytes = body(&mut stream).await?;
            if tag == b'Q' && bytes == b"ROLLBACK\0" {
                break;
            }
            respond(&mut stream, tag, &bytes, &mut command).await?;
        }
    }
    notify
        .send(())
        .map_err(|()| invalid("stall observer dropped"))?;
    std::future::pending::<()>().await;
    stream.shutdown().await
}

async fn body(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let length = stream
        .read_u32()
        .await?
        .checked_sub(4)
        .filter(|length| *length <= 65_536)
        .ok_or_else(|| invalid("invalid message length"))?;
    let mut bytes = vec![0; usize::try_from(length).map_err(io::Error::other)?];
    stream.read_exact(&mut bytes).await?;
    Ok(bytes)
}

async fn message(stream: &mut TcpStream, tag: u8, bytes: &[u8]) -> io::Result<()> {
    let length = u32::try_from(bytes.len())
        .map_err(io::Error::other)?
        .checked_add(4)
        .ok_or_else(|| invalid("message length overflow"))?;
    stream.write_u8(tag).await?;
    stream.write_u32(length).await?;
    stream.write_all(bytes).await
}

#[derive(Clone, Copy)]
enum Command {
    Timeout,
    SearchPath,
    Result,
}

impl Command {
    fn parse(bytes: &[u8]) -> io::Result<Self> {
        match bytes.split(|byte| *byte == 0).nth(1) {
            Some(b"SELECT set_config('statement_timeout', $1, true)") => Ok(Self::Timeout),
            Some(b"SET LOCAL search_path = inventory") => Ok(Self::SearchPath),
            Some(b"SELECT 1 AS n") => Ok(Self::Result),
            _ => Err(invalid("unexpected prepared statement")),
        }
    }
}

async fn respond(
    stream: &mut TcpStream,
    tag: u8,
    bytes: &[u8],
    command: &mut Command,
) -> io::Result<()> {
    match tag {
        b'P' => {
            *command = Command::parse(bytes)?;
            message(stream, b'1', b"").await
        }
        b'D' => describe(stream, *command).await,
        b'B' => message(stream, b'2', b"").await,
        b'E' => execute(stream, *command).await,
        b'C' => message(stream, b'3', b"").await,
        b'S' => message(stream, b'Z', b"T").await,
        _ => Err(invalid("unexpected frontend message")),
    }
}

async fn describe(stream: &mut TcpStream, command: Command) -> io::Result<()> {
    let parameters = match command {
        Command::Timeout => vec![0, 1, 0, 0, 0, 25],
        Command::SearchPath | Command::Result => vec![0, 0],
    };
    message(stream, b't', &parameters).await?;
    let (name, oid, size) = match command {
        Command::Timeout => ("set_config", 25u32, -1i16),
        Command::Result => ("n", 23u32, 4i16),
        Command::SearchPath => return message(stream, b'n', b"").await,
    };
    let mut column = Vec::from(1u16.to_be_bytes());
    column.extend_from_slice(name.as_bytes());
    column.push(0);
    column.extend_from_slice(&0u32.to_be_bytes());
    column.extend_from_slice(&0u16.to_be_bytes());
    column.extend_from_slice(&oid.to_be_bytes());
    column.extend_from_slice(&size.to_be_bytes());
    column.extend_from_slice(&(-1i32).to_be_bytes());
    column.extend_from_slice(&0u16.to_be_bytes());
    message(stream, b'T', &column).await
}

async fn execute(stream: &mut TcpStream, command: Command) -> io::Result<()> {
    let value = match command {
        Command::Timeout => b"5000ms".to_vec(),
        Command::Result => 1i32.to_be_bytes().to_vec(),
        Command::SearchPath => return message(stream, b'C', b"SET\0").await,
    };
    let mut row = Vec::from(1u16.to_be_bytes());
    row.extend_from_slice(
        &i32::try_from(value.len())
            .map_err(io::Error::other)?
            .to_be_bytes(),
    );
    row.extend_from_slice(&value);
    message(stream, b'D', &row).await?;
    message(stream, b'C', b"SELECT 1\0").await
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
