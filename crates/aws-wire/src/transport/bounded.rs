use super::{TransportError, error::Source};
use http::Uri;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tower_service::Service;

type Connecting<T> = Pin<Box<dyn Future<Output = Result<T, Source>> + Send>>;

#[derive(Clone)]
pub(super) struct Bounded<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Bounded<S> {
    pub(super) fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

impl<S> Service<Uri> for Bounded<S>
where
    S: Service<Uri>,
    S::Future: Send + 'static,
    S::Response: 'static,
    S::Error: Into<Source>,
{
    type Response = S::Response;
    type Error = Source;
    type Future = Connecting<S::Response>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Source>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let connecting = tokio::time::timeout(self.timeout, self.inner.call(uri));
        Box::pin(async move {
            connecting
                .await
                .map_err(|_| Source::from(TransportError::Timeout))?
                .map_err(Into::into)
        })
    }
}
