use super::Counters;
use hyper_util::client::legacy::connect::{Connected, Connection};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
    sync::OwnedSemaphorePermit,
};

pub(super) struct Socket {
    // Field order closes the socket before releasing its permit.
    inner: TcpStream,
    counters: Arc<Counters>,
    _permit: OwnedSemaphorePermit,
}

impl Socket {
    pub(super) fn new(
        inner: TcpStream,
        counters: Arc<Counters>,
        permit: OwnedSemaphorePermit,
    ) -> Self {
        counters.opened();
        Self {
            inner,
            counters,
            _permit: permit,
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.counters.closed();
    }
}

impl Connection for Socket {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl AsyncRead for Socket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let result = Pin::new(&mut this.inner).poll_read(cx, buf);
        this.counters
            .read(buf.filled().len().saturating_sub(before));
        result
    }
}

impl AsyncWrite for Socket {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
