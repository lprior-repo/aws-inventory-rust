use inventory_harness::proxy::{
    Config, Fault, MAX_BODY_BYTES, MAX_HEAD_BYTES, MAX_HEADERS, Schedule, Scope,
};
use std::{error::Error, io, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::{JoinHandle, JoinSet},
};

pub type TestResult = Result<(), Box<dyn Error>>;

pub type Message<'a> = (&'a [u8], &'a [u8]);
pub const TEST_REQUESTS: usize = 1_024;
pub const TEST_RESPONSE_CAP: u64 = 16_777_216;
pub const OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"ok\":\"value!\"}";

pub struct Server {
    pub address: SocketAddr,
    pub captured: mpsc::Receiver<Vec<u8>>,
    owner: JoinHandle<io::Result<()>>,
}
impl Server {
    pub async fn start(response: &[u8]) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let response = response.to_vec();
        let (sender, captured) = mpsc::channel(TEST_REQUESTS);
        let owner = tokio::spawn(accept(listener, response, sender));
        Ok(Self {
            address,
            captured,
            owner,
        })
    }
    pub fn config(&self) -> Config {
        Config::new(format!("http://{}", self.address), 7)
    }
    pub fn fault(&self, fault: Fault) -> Config {
        let mut config = self.config();
        config.schedule.push(Schedule {
            scope: Scope::Unsigned,
            ordinal: 0,
            fault,
        });
        config
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.owner.abort();
    }
}

async fn serve(
    socket: TcpStream,
    response: Vec<u8>,
    captured: mpsc::Sender<Vec<u8>>,
) -> io::Result<()> {
    let mut reader = BufReader::new(socket);
    for _ in 0..TEST_REQUESTS {
        let Some(request) = receive_request(&mut reader).await? else {
            return Ok(());
        };
        let close = closes(&request);
        captured.send(request).await.map_err(io::Error::other)?;
        reader.get_mut().write_all(&response).await?;
        if close || closes(&response) || close_delimited(&response) {
            return Ok(());
        }
    }
    Err(io::Error::other("test request budget exhausted"))
}

pub async fn receive_request(reader: &mut BufReader<TcpStream>) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    for _ in 0..MAX_HEAD_BYTES {
        match reader.read_u8().await {
            Ok(byte) => bytes.push(byte),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof && bytes.is_empty() => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        }
        if bytes.ends_with(b"\r\n\r\n") {
            let size = request_length(&bytes)?;
            let mut body = vec![0; size];
            reader.read_exact(&mut body).await?;
            bytes.extend_from_slice(&body);
            return Ok(Some(bytes));
        }
    }
    Err(io::Error::other("test head limit"))
}
fn request_length(bytes: &[u8]) -> io::Result<usize> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut parsed = httparse::Request::new(&mut headers);
    parsed.parse(bytes).map_err(io::Error::other)?;
    let Some(header) = parsed
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("content-length"))
    else {
        return Ok(0);
    };
    let length = std::str::from_utf8(header.value)
        .map_err(io::Error::other)?
        .parse::<usize>()
        .map_err(io::Error::other)?;
    if length > MAX_BODY_BYTES {
        return Err(io::Error::other("test body limit"));
    }
    Ok(length)
}

pub fn request(key: &str, region: &str, service: &str) -> Vec<u8> {
    format!("GET / HTTP/1.1\r\nHost: signed.example:1234\r\nAuthorization: AWS4-HMAC-SHA256 Credential={key}/20261113/{region}/{service}/aws4_request, SignedHeaders=host, Signature=abc\r\nConnection: close\r\n\r\n").into_bytes()
}
pub const UNSIGNED: &[u8] = b"GET / HTTP/1.1\r\nHost: example\r\nConnection: close\r\n\r\n";

async fn exchange_inner(address: SocketAddr, request: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut socket = TcpStream::connect(address).await?;
    socket.write_all(request).await?;
    let mut bytes = Vec::new();
    socket
        .take(TEST_RESPONSE_CAP)
        .read_to_end(&mut bytes)
        .await?;
    Ok(bytes)
}

pub fn split(bytes: &[u8]) -> Result<Message<'_>, Box<dyn Error>> {
    let at = bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("missing HTTP head")?
        .checked_add(4)
        .ok_or("head overflow")?;
    Ok((
        bytes.get(..at).ok_or("head")?,
        bytes.get(at..).ok_or("body")?,
    ))
}
pub fn field<'a>(bytes: &'a [u8], name: &str) -> Result<&'a str, Box<dyn Error>> {
    std::str::from_utf8(split(bytes)?.0)?
        .split("\r\n")
        .filter_map(|s| s.split_once(':'))
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
        .ok_or_else(|| format!("missing {name}").into())
}
pub async fn reached(proxy: &inventory_harness::proxy::Proxy, requests: u64) -> TestResult {
    super::clock::until("proxy consumes request", || {
        Ok(proxy.snapshot()?.counters.requests >= requests)
    })
    .await
}

pub async fn exchange(address: SocketAddr, request: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    super::clock::within("TCP HTTP exchange", exchange_inner(address, request)).await
}

pub async fn ready(proxy: &inventory_harness::proxy::Proxy, responses: u64) -> TestResult {
    super::clock::until("response prepared before latency", || {
        Ok(proxy.snapshot()?.counters.responses_ready >= responses)
    })
    .await
}

pub async fn finished(proxy: &inventory_harness::proxy::Proxy, sessions: u64) -> TestResult {
    super::clock::until("session terminates before shutdown", || {
        Ok(proxy.snapshot()?.counters.finished >= sessions)
    })
    .await
}

pub async fn shutdown(
    proxy: inventory_harness::proxy::Proxy,
) -> Result<inventory_harness::proxy::DrainReport, Box<dyn Error>> {
    super::clock::within("proxy shutdown and task drain", proxy.shutdown()).await
}

pub async fn captured(server: &mut Server) -> Result<Vec<u8>, Box<dyn Error>> {
    super::clock::within("upstream captures exact request", async {
        server
            .captured
            .recv()
            .await
            .ok_or("missing upstream request")
    })
    .await
}

type ControlResult = Result<(), inventory_harness::proxy::ProxyError>;
pub async fn resume_outcome(
    proxy: &inventory_harness::proxy::Proxy,
) -> Result<ControlResult, Box<dyn Error>> {
    super::clock::within("proxy resume acknowledgement", async {
        Ok::<ControlResult, Box<dyn Error>>(proxy.resume().await)
    })
    .await
}

async fn accept(
    listener: TcpListener,
    response: Vec<u8>,
    sender: mpsc::Sender<Vec<u8>>,
) -> io::Result<()> {
    let mut tasks = JoinSet::new();
    for _ in 0..TEST_REQUESTS {
        tokio::select! {
            joined = tasks.join_next(), if !tasks.is_empty() => {
                joined.ok_or_else(|| io::Error::other("missing test task"))?
                    .map_err(io::Error::other)??;
            }
            accepted = listener.accept() => {
                let (socket, _) = accepted?;
                drop(tasks.spawn(serve(socket, response.clone(), sender.clone())));
            }
        }
    }
    Ok(())
}

fn closes(bytes: &[u8]) -> bool {
    bytes
        .windows(b"Connection: close".len())
        .any(|window| window == b"Connection: close")
}

fn close_delimited(response: &[u8]) -> bool {
    !response
        .windows(b"Content-Length".len())
        .any(|window| window == b"Content-Length")
        && !response
            .windows(b"Transfer-Encoding".len())
            .any(|window| window == b"Transfer-Encoding")
}
