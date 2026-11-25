use std::{io, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, UnixStream},
    sync::oneshot,
    task::JoinHandle,
};

type ProxyResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
const MAX_FRAME: usize = 2_097_152;

pub(super) struct Proxy {
    pub(super) endpoint: String,
    arrived: oneshot::Receiver<()>,
    release: Option<oneshot::Sender<()>>,
    executed: oneshot::Receiver<Vec<u8>>,
    task: Option<JoinHandle<Result<(), String>>>,
}

enum Barrier {
    Transaction(&'static str),
    Claim,
}

impl Proxy {
    pub(super) async fn start(native: &str, needle: &'static str) -> ProxyResult<Self> {
        Self::create(native, Barrier::Transaction(needle)).await
    }

    pub(super) async fn start_claim(native: &str) -> ProxyResult<Self> {
        Self::create(native, Barrier::Claim).await
    }

    async fn create(native: &str, barrier: Barrier) -> ProxyResult<Self> {
        let socket = native
            .strip_prefix("redis+unix://")
            .ok_or("owned native endpoint is not Unix")?
            .to_owned();
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("redis://{}", listener.local_addr()?);
        let (arrived_tx, arrived) = oneshot::channel();
        let (release, release_rx) = oneshot::channel();
        let (executed_tx, executed) = oneshot::channel();
        let task = tokio::spawn(async move {
            relay(
                listener,
                &socket,
                barrier,
                arrived_tx,
                release_rx,
                executed_tx,
            )
            .await
            .map_err(|error| error.to_string())
        });
        Ok(Self {
            endpoint,
            arrived,
            release: Some(release),
            executed,
            task: Some(task),
        })
    }

    pub(super) async fn paused(&mut self) -> ProxyResult {
        tokio::time::timeout(Duration::from_secs(20), &mut self.arrived).await??;
        Ok(())
    }

    pub(super) async fn resume(mut self) -> ProxyResult<(Vec<u8>, RunningProxy)> {
        self.release
            .take()
            .ok_or("proxy was already released")?
            .send(())
            .map_err(|()| "proxy release receiver disappeared")?;
        let result = tokio::time::timeout(Duration::from_secs(20), &mut self.executed).await;
        let task = self.task.take().ok_or("proxy worker is missing")?;
        match result {
            Ok(Ok(response)) => Ok((response, RunningProxy(Some(task)))),
            error => {
                task.abort();
                let joined = task.await;
                Err(
                    format!("native proxy did not execute released command: {error:?}; {joined:?}")
                        .into(),
                )
            }
        }
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

pub(super) struct RunningProxy(Option<JoinHandle<Result<(), String>>>);
impl RunningProxy {
    pub(super) async fn close(mut self) -> ProxyResult {
        let task = self.0.take().ok_or("proxy worker is missing")?;
        task.abort();
        match task.await {
            Ok(Ok(())) => Ok(()),
            Err(error) if error.is_cancelled() => Ok(()),
            other => Err(format!("native proxy failed: {other:?}").into()),
        }
    }
}

impl Drop for RunningProxy {
    fn drop(&mut self) {
        if let Some(task) = &self.0 {
            task.abort();
        }
    }
}

async fn relay(
    listener: TcpListener,
    socket: &str,
    kind: Barrier,
    arrived: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
    executed: oneshot::Sender<Vec<u8>>,
) -> ProxyResult {
    let (client, _) = listener.accept().await?;
    let (read, mut write) = client.into_split();
    let mut client = BufReader::new(read);
    let mut native = BufReader::new(UnixStream::connect(socket).await?);
    let mut barrier = Some((arrived, release, executed));
    let mut armed = false;
    for _ in 0..100_000 {
        let frame = match read_frame(&mut client).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        let paused = match &kind {
            Barrier::Transaction(needle) => {
                if frame
                    .windows(needle.len())
                    .any(|window| window == needle.as_bytes())
                {
                    armed = true;
                }
                armed && frame == b"*1\r\n$4\r\nEXEC\r\n"
            }
            Barrier::Claim => frame.starts_with(b"*5\r\n$4\r\nEVAL\r\n"),
        };
        let acknowledgement = if paused {
            match barrier.take() {
                Some((arrived, release, executed)) => {
                    arrived
                        .send(())
                        .map_err(|()| "proxy arrival receiver disappeared")?;
                    release.await?;
                    Some(executed)
                }
                None => None,
            }
        } else {
            None
        };
        native.get_mut().write_all(&frame).await?;
        let response = read_frame(&mut native).await?;
        if let Some(executed) = acknowledgement {
            executed
                .send(response.clone())
                .map_err(|_| "proxy execution receiver disappeared")?;
        }
        if let Err(error) = write.write_all(&response).await {
            if matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
            ) {
                return Ok(());
            }
            return Err(error.into());
        }
    }
    Err("native proxy command budget exhausted".into())
}

async fn read_frame<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut pending = 1_usize;
    for _ in 0..100_000 {
        let mut line = Vec::new();
        if reader.read_until(b'\n', &mut line).await? == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        if output.len().saturating_add(line.len()) > MAX_FRAME || !line.ends_with(b"\r\n") {
            return Err(io::Error::other("native RESP line exceeds bounds"));
        }
        pending = pending
            .checked_sub(1)
            .ok_or_else(|| io::Error::other("invalid RESP shape"))?;
        match line.first() {
            Some(b'*' | b'$') => {
                let text = std::str::from_utf8(
                    line.get(1..line.len().saturating_sub(2))
                        .ok_or_else(|| io::Error::other("missing RESP size"))?,
                )
                .map_err(io::Error::other)?;
                let size: i64 = text.parse().map_err(io::Error::other)?;
                if size < -1 {
                    return Err(io::Error::other("invalid RESP size"));
                }
                if size >= 0 {
                    let size = usize::try_from(size).map_err(io::Error::other)?;
                    if line.first() == Some(&b'*') {
                        pending = pending
                            .checked_add(size)
                            .filter(|count| *count <= 100_000)
                            .ok_or_else(|| io::Error::other("RESP array exceeds bound"))?;
                    } else {
                        append_bulk(reader, &mut output, &line, size).await?;
                        if pending == 0 {
                            return Ok(output);
                        }
                        continue;
                    }
                }
            }
            Some(b'+' | b'-' | b':') => (),
            _ => return Err(io::Error::other("unsupported owned RESP frame")),
        }
        output.extend_from_slice(&line);
        if pending == 0 {
            return Ok(output);
        }
    }
    Err(io::Error::other("native RESP item budget exhausted"))
}

async fn append_bulk<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    output: &mut Vec<u8>,
    line: &[u8],
    size: usize,
) -> io::Result<()> {
    let start = output
        .len()
        .checked_add(line.len())
        .ok_or_else(|| io::Error::other("RESP line length overflow"))?;
    let end = start
        .checked_add(size)
        .and_then(|end| end.checked_add(2))
        .filter(|end| *end <= MAX_FRAME)
        .ok_or_else(|| io::Error::other("RESP bulk exceeds bound"))?;
    output.extend_from_slice(line);
    output.resize(end, 0);
    let body = output
        .get_mut(start..end)
        .ok_or_else(|| io::Error::other("RESP bulk range missing"))?;
    reader.read_exact(body).await?;
    Ok(())
}
