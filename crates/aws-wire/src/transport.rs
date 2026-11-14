//! One shared HTTP/1.1/ALPN HTTP/2 pool, without retries or redirects.
//!
//! A socket holds one global permit through TLS, active use and idle pooling.
//! Idle eviction checks every `max(Config::idle_timeout, 90 ms)`; an idle socket
//! can retain its permit for idle timeout plus one check interval (scheduler permitting).
//! Connect waiters remain deadline-bound and may time out before eviction.
//! A sweep of M distinct endpoints with cap N needs approximately ceil(M/N) capacity waves.
//! Idle permits can delay fresh-host waiters past their deadlines. Choose the cap near the
//! in-flight limit and a short valid expiry; defaults await sweep benchmark measurements.
//! Hyper-util 0.1.21 ages shared HTTP/2 senders by checkout, not active streams
//! (`pool.rs:227-241,318-325,488-503`). Expiry preserves a live stream and its socket permit,
//! but another request needs a second connection/permit rather than multiplexing that stream.
//! Construct outside latency-sensitive async code: system/extra roots load synchronously.
//! Bodies are limited before collection; HTTP/1 read buffers are capped at 8 KiB.

mod bounded;
mod config;
mod connection;
mod connector;
mod counters;
mod error;
mod resolver;
mod trust;

use bounded::Bounded;
use bytes::Bytes;
use http::{HeaderMap, Request, StatusCode, Version};
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Body, Incoming};
use hyper_util::{
    client::legacy::{
        Client as Pool,
        connect::{CaptureConnection, capture_connection},
    },
    rt::{TokioExecutor, TokioTimer},
};
use std::sync::Arc;

pub use config::Config;
use connector::Connector;
use counters::Counters;
pub use counters::Statistics;
pub use error::TransportError;

type HttpPool = Pool<Bounded<hyper_rustls::HttpsConnector<Connector>>, Full<Bytes>>;

/// Complete bounded response; redirects and error HTTP statuses are returned unchanged.
#[derive(Debug)]
pub struct Response {
    /// HTTP status, including non-success statuses.
    pub status: StatusCode,
    /// Response headers without interpretation.
    pub headers: HeaderMap,
    /// Whole response body, no larger than the configured limit.
    pub body: Bytes,
    /// Negotiated protocol, observable for ALPN tests and diagnostics.
    pub version: Version,
}

/// Cheaply cloned handle; all clones share a single connection pool and socket cap.
#[derive(Clone)]
pub struct Client {
    pool: HttpPool,
    config: Arc<Config>,
    counters: Arc<Counters>,
}

impl Client {
    /// Build a pool and load trust roots synchronously.
    ///
    /// # Errors
    /// Returns `InvalidRequest` for invalid bounds, `Io` for bundle access/allocation,
    /// or `Tls` for root-loading/verification-configuration failures.
    pub fn new(config: Config) -> Result<Self, TransportError> {
        config.validate()?;
        let counters = Arc::new(Counters::default());
        let pool = build_pool(&config, Arc::clone(&counters))?;
        Ok(Self {
            pool,
            config: Arc::new(config),
            counters,
        })
    }

    /// Send once, with a deadline covering pool wait, connect, TLS, headers and body.
    /// Cancellation discards incomplete HTTP/1 connections, but resets only the HTTP/2 stream.
    ///
    /// # Errors
    /// Returns a typed transport error; never follows redirects or retries.
    pub async fn send(&self, request: Request<Bytes>) -> Result<Response, TransportError> {
        self.validate(&request)?;
        self.counters.sent();
        let mut request = request.map(Full::new);
        let guard = InFlight(Some(capture_connection(&mut request)));
        let result = tokio::time::timeout(self.config.request_deadline, self.exchange(request))
            .await
            .map_err(|_| TransportError::Timeout)
            .and_then(|result| result);
        if matches!(result, Err(TransportError::Timeout)) {
            self.counters.timed_out();
        }
        let response = result?;
        guard.complete();
        Ok(response)
    }

    /// Read approximate atomic counters without taking a lock.
    #[must_use]
    pub fn statistics(&self) -> Statistics {
        self.counters.snapshot()
    }

    fn validate(&self, request: &Request<Bytes>) -> Result<(), TransportError> {
        let uri = request.uri();
        let allowed = uri.scheme_str() == Some("https")
            || (self.config.allow_plain_http && uri.scheme_str() == Some("http"));
        if !allowed || uri.host().is_none() || request.method() == http::Method::CONNECT {
            return Err(TransportError::InvalidRequest(
                "absolute HTTP(S) URI required; CONNECT forbidden",
            ));
        }
        Ok(())
    }

    async fn exchange(&self, request: Request<Full<Bytes>>) -> Result<Response, TransportError> {
        let response = self
            .pool
            .request(request)
            .await
            .map_err(|e| error::classify(Box::new(e)))?;
        let (parts, body) = response.into_parts();
        let body = self
            .bounded_body(body)?
            .collect()
            .await
            .map_err(|e| self.body_error(e))?
            .to_bytes();
        Ok(Response {
            status: parts.status,
            headers: parts.headers,
            body,
            version: parts.version,
        })
    }

    fn bounded_body(&self, body: Incoming) -> Result<Limited<Incoming>, TransportError> {
        let within = usize::try_from(body.size_hint().lower())
            .is_ok_and(|length| length <= self.config.response_body_limit);
        if !within {
            return Err(TransportError::BodyTooLarge {
                limit: self.config.response_body_limit,
            });
        }
        Ok(Limited::new(body, self.config.response_body_limit))
    }

    fn body_error(&self, error: error::Source) -> TransportError {
        if error.is::<http_body_util::LengthLimitError>() {
            return TransportError::BodyTooLarge {
                limit: self.config.response_body_limit,
            };
        }
        error::classify(error)
    }
}

fn build_pool(config: &Config, counters: Arc<Counters>) -> Result<HttpPool, TransportError> {
    let connector = Connector::new(config.max_connections, counters);
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(trust::load(config)?)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .wrap_connector(connector);
    Ok(Pool::builder(TokioExecutor::new())
        .pool_timer(TokioTimer::new())
        .pool_idle_timeout(config.idle_timeout)
        .pool_max_idle_per_host(config.pool_idle_per_host)
        .http1_max_buf_size(8192)
        .retry_canceled_requests(false)
        .build(Bounded::new(https, config.connect_timeout)))
}

struct InFlight(Option<CaptureConnection>);

impl InFlight {
    fn complete(mut self) {
        self.0 = None;
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        if let Some(captured) = &self.0
            && let Some(connection) = captured.connection_metadata().as_ref()
            && !connection.is_negotiated_h2()
        {
            connection.poison();
        }
    }
}
