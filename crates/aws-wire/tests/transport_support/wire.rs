use super::{TestError, TestResult, tls, watchdog::watch};
use aws_wire::transport::{Client, Config, Response, TransportError};
use bytes::Bytes;
use std::{fmt::Debug, io, path::PathBuf, pin::Pin, task::Poll};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpSocket},
};
use tokio_rustls::TlsAcceptor;

pub trait TestIo: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> TestIo for T {}
pub type Io = Box<dyn TestIo>;
pub type SendResult = Result<Response, TransportError>;

pub struct Endpoint {
    pub listener: TcpListener,
    pub url: String,
    pub ca: Option<PathBuf>,
    pub acceptor: Option<TlsAcceptor>,
}

impl Endpoint {
    pub async fn bind(tls: bool, alpn: &[u8]) -> Result<Self, TestError> {
        let socket = TcpSocket::new_v4()?;
        socket.set_recv_buffer_size(1024)?;
        socket.bind("127.0.0.1:0".parse()?)?;
        let listener = socket.listen(16)?;
        let address = listener.local_addr()?;
        let (ca, acceptor, scheme) = if tls {
            let path = tls::ca_path(address.port());
            let acceptor = tls::credentials(&path, alpn)?;
            (Some(path), Some(acceptor), "https")
        } else {
            (None, None, "http")
        };
        Ok(Self {
            listener,
            url: format!("{scheme}://{address}/"),
            ca,
            acceptor,
        })
    }

    pub fn config(&self, cap: usize) -> Config {
        Config {
            extra_ca_bundle: self.ca.clone(),
            max_connections: cap,
            pool_idle_per_host: cap.min(2),
            ..super::config()
        }
    }

    pub async fn accept<F>(&self, send: &mut Pin<Box<F>>) -> Result<Io, TestError>
    where
        F: Future + ?Sized,
        F::Output: Debug,
    {
        let (socket, _) = until(send, self.listener.accept()).await??;
        socket.set_nodelay(true)?;
        if let Some(acceptor) = &self.acceptor {
            return Ok(Box::new(until(send, acceptor.accept(socket)).await??));
        }
        Ok(Box::new(socket))
    }

    pub fn cleanup(self) -> io::Result<()> {
        if let Some(path) = self.ca {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

pub async fn until<F, S>(send: &mut Pin<Box<F>>, signal: S) -> Result<S::Output, TestError>
where
    F: Future + ?Sized,
    F::Output: Debug,
    S: Future,
{
    watch(async {
        tokio::select! {
            result = send => Err(format!("request completed before phase: {result:?}").into()),
            result = signal => Ok::<_, TestError>(result),
        }
    })
    .await?
}

pub async fn head<R: AsyncRead + Unpin + ?Sized>(stream: &mut R) -> io::Result<Bytes> {
    let mut bytes = Vec::with_capacity(8192);
    for _ in 0..8192 {
        let mut byte = [0];
        if stream.read(&mut byte).await? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "request head incomplete",
            ));
        }
        bytes.extend_from_slice(&byte);
        if bytes.ends_with(b"\r\n\r\n") {
            return Ok(Bytes::from(bytes));
        }
    }
    Err(io::Error::other("request head exceeds fixture bound"))
}

pub async fn write(stream: &mut Io, bytes: &[u8]) -> io::Result<()> {
    stream.write_all(bytes).await?;
    stream.flush().await
}

pub async fn read_at_least<F>(client: &Client, send: &mut Pin<Box<F>>, bytes: usize) -> TestResult
where
    F: Future<Output = Result<Response, TransportError>> + ?Sized,
{
    watch(std::future::poll_fn(|cx| {
        if let Poll::Ready(result) = send.as_mut().poll(cx) {
            return Poll::Ready(Err(format!(
                "request completed before read phase: {result:?}"
            )
            .into()));
        }
        if client.statistics().bytes_read >= bytes {
            return Poll::Ready(Ok::<(), TestError>(()));
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    }))
    .await?
}

pub async fn no_accept(listener: &TcpListener) -> TestResult {
    std::future::poll_fn(|cx| match listener.poll_accept(cx) {
        Poll::Pending => Poll::Ready(Ok(())),
        Poll::Ready(result) => Poll::Ready(Err(
            format!("unexpected extra connection: {result:?}").into()
        )),
    })
    .await
}

pub async fn complete<F>(
    mut send: Pin<Box<F>>,
    stream: &mut Io,
    response: &[u8],
) -> Result<SendResult, TestError>
where
    F: Future<Output = SendResult> + ?Sized,
{
    until(&mut send, head(stream)).await??;
    write(stream, response).await?;
    watch(send).await
}
