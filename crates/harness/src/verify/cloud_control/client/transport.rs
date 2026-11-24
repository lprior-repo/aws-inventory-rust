use aws_smithy_runtime_api::{
    box_error::BoxError,
    client::{
        http::{
            HttpClient, HttpConnector, HttpConnectorFuture, SharedHttpClient, SharedHttpConnector,
            http_client_fn,
        },
        orchestrator::HttpRequest,
        result::ConnectorError,
    },
};
use aws_smithy_types::{body::SdkBody, byte_stream::ByteStream};
use futures_util::{TryStreamExt, stream};

pub(super) const MAX_RESPONSE_BYTES: usize = 1_048_576;

#[derive(Debug, thiserror::Error)]
#[error("response exceeds {limit} bytes")]
pub(super) struct ResponseBound {
    pub(super) limit: usize,
}

#[derive(Debug)]
struct BoundedConnector(SharedHttpConnector);

impl HttpConnector for BoundedConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let pending = self.0.call(request);
        HttpConnectorFuture::new(async move {
            let mut response = pending.await?;
            let body = bounded(response.take_body())
                .await
                .map_err(|error| ConnectorError::other(error, None))?;
            *response.body_mut() = body;
            Ok(response)
        })
    }
}

pub(super) fn client(inner: SharedHttpClient) -> SharedHttpClient {
    http_client_fn(move |settings, components| {
        SharedHttpConnector::new(BoundedConnector(inner.http_connector(settings, components)))
    })
}

async fn bounded(body: SdkBody) -> Result<SdkBody, BoxError> {
    let chunks = stream::try_unfold(ByteStream::new(body), |mut body| async move {
        body.try_next()
            .await
            .map(|next| next.map(|chunk| (chunk, body)))
    });
    let bytes = chunks
        .map_err(|error| -> BoxError { Box::new(error) })
        .try_fold(Vec::new(), |mut bytes, chunk| async move {
            if bytes
                .len()
                .checked_add(chunk.len())
                .is_none_or(|size| size > MAX_RESPONSE_BYTES)
            {
                let error: BoxError = Box::new(ResponseBound {
                    limit: MAX_RESPONSE_BYTES,
                });
                return Err(error);
            }
            bytes.extend_from_slice(&chunk);
            Ok(bytes)
        })
        .await?;
    Ok(SdkBody::from(bytes))
}

pub(super) fn exceeded(error: &(dyn std::error::Error + 'static)) -> bool {
    std::iter::successors(Some(error), |error| error.source())
        .any(|error| error.downcast_ref::<ResponseBound>().is_some())
}
