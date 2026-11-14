use super::{Counters, connection::Socket, error::Source, resolver::Resolver};
use http::Uri;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioIo};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Semaphore;
use tower_service::Service;

type Connecting = Pin<Box<dyn Future<Output = Result<TokioIo<Socket>, Source>> + Send>>;

#[derive(Clone)]
pub(super) struct Connector {
    inner: HttpConnector<Resolver>,
    slots: Arc<Semaphore>,
    counters: Arc<Counters>,
}

impl Connector {
    pub(super) fn new(cap: usize, counters: Arc<Counters>) -> Result<Self, std::io::Error> {
        let mut inner = HttpConnector::new_with_resolver(Resolver::new()?);
        inner.enforce_http(false);

        inner.set_happy_eyeballs_timeout(None);
        inner.set_nodelay(true);
        Ok(Self {
            inner,
            slots: Arc::new(Semaphore::new(cap)),
            counters,
        })
    }
}

impl Service<Uri> for Connector {
    type Response = TokioIo<Socket>;
    type Error = Source;
    type Future = Connecting;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let mut inner = self.inner.clone();
        let slots = Arc::clone(&self.slots);
        let counters = Arc::clone(&self.counters);
        Box::pin(async move {
            let permit = slots.acquire_owned().await?;
            let socket = inner.call(uri).await?.into_inner();
            Ok(TokioIo::new(Socket::new(socket, counters, permit)))
        })
    }
}
