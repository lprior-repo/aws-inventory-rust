use super::{
    clock::within,
    support::{OK, TestResult, receive_request},
    wire,
};
use inventory_harness::proxy::{Config, Proxy};
use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::JoinHandle,
};

type Captured = (u64, Vec<u8>);
const UPSTREAM_REQUESTS: usize = 2;
struct Upstream {
    address: SocketAddr,
    accepted: Arc<AtomicU64>,
    captured: mpsc::Receiver<Captured>,
    owner: JoinHandle<io::Result<()>>,
}
impl Upstream {
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let accepted = Arc::new(AtomicU64::new(0));
        let (sender, captured) = mpsc::channel(UPSTREAM_REQUESTS);
        let owner = tokio::spawn(serve(listener, sender, accepted.clone()));
        Ok(Self {
            address,
            accepted,
            captured,
            owner,
        })
    }
}
impl Drop for Upstream {
    fn drop(&mut self) {
        self.owner.abort();
    }
}

async fn serve(
    listener: TcpListener,
    captured: mpsc::Sender<Captured>,
    accepted: Arc<AtomicU64>,
) -> io::Result<()> {
    let (socket, _) = listener.accept().await?;
    accepted.store(1, Ordering::SeqCst);
    let mut reader = BufReader::new(socket);
    for _ in 0..UPSTREAM_REQUESTS {
        let request = receive_request(&mut reader)
            .await?
            .ok_or_else(|| io::Error::other("early EOF"))?;
        captured
            .send((1, request))
            .await
            .map_err(io::Error::other)?;
        reader.get_mut().write_all(OK).await?;
    }
    Ok(())
}

pub async fn verify() -> TestResult {
    within("reuse one tagged upstream connection", scenario()).await
}

async fn scenario() -> TestResult {
    let mut upstream = Upstream::start().await?;
    let proxy = Proxy::start(Config::new(format!("http://{}", upstream.address), 7)).await?;
    let mut client = BufReader::new(TcpStream::connect(proxy.address()).await?);
    let requests = [
        b"GET /first HTTP/1.1\r\nHost: original.example\r\n\r\n".as_slice(),
        b"GET /second HTTP/1.1\r\nHost: original.example\r\nConnection: close\r\n\r\n",
    ];
    for request in requests {
        client.get_mut().write_all(request).await?;
        assert_eq!(wire::message(&mut client).await?, OK);
        let (connection, bytes) = upstream.captured.recv().await.ok_or("capture missing")?;
        assert_eq!(connection, 1);
        assert_eq!(bytes, request);
    }
    assert_eq!(upstream.accepted.load(Ordering::SeqCst), 1);
    assert_eq!(proxy.snapshot()?.counters.forwarded, 2);
    (&mut upstream.owner).await??;
    proxy.shutdown().await?;
    Ok(())
}
