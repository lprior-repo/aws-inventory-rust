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

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,

    pub headers: HeaderMap,

    pub body: Bytes,

    pub version: Version,
}

#[derive(Clone)]
pub struct Client {
    pool: HttpPool,
    config: Arc<Config>,
    counters: Arc<Counters>,
}

impl Client {
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
