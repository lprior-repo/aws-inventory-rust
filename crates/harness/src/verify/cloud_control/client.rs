mod transport;

use crate::HarnessError;
use aws_sdk_cloudcontrol::{Client, error::ProvideErrorMetadata};
use aws_smithy_http_client::{
    Builder,
    tls::{Provider, TlsContext, TrustStore, rustls_provider::CryptoMode},
};
use futures_util::{TryStreamExt, stream};
use std::{collections::BTreeSet, time::Duration};

const MAX_PAGES: usize = 256;
const PAGE_SIZE: i32 = 100;
const MAX_IDENTIFIER_BYTES: usize = 4096;

pub(super) enum Listed {
    Identities(BTreeSet<String>),
    NotListable,
}

#[derive(Debug, thiserror::Error)]
pub enum CloudControlError {
    #[error("Cloud Control type is not listable")]
    NotListable,
    #[error("Cloud Control page bound exceeded")]
    PageBound,
    #[error("Cloud Control repeated pagination token")]
    RepeatedToken,
    #[error("Cloud Control page identity bound exceeded")]
    PageIdentityBound,
    #[error("Cloud Control resource lacks primary identifier")]
    MissingIdentifier,
    #[error("Cloud Control {type_name} page {page} response exceeds {limit} bytes")]
    ResponseBytes {
        type_name: String,
        page: usize,
        limit: usize,
    },
    #[error("Cloud Control {type_name} page {page} identifier exceeds {limit} bytes")]
    IdentifierBytes {
        type_name: String,
        page: usize,
        limit: usize,
    },
    #[error("Cloud Control {type_name}: {source}")]
    AmbiguousIdentity {
        type_name: String,
        #[source]
        source: crate::assertions::cloud_control::identities::AmbiguousIdentity,
    },
    #[error("Cloud Control seeded kinds have no mapped target: {kinds:?}")]
    UnmappedKinds { kinds: Vec<String> },
    #[error("Cloud Control kind not in catalog: {kinds:?}")]
    UnknownKinds { kinds: Vec<String> },
    #[error("Cloud Control ListResources {type_name}: {source}")]
    Service {
        type_name: String,
        code: Option<String>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub(super) fn http_client(
    pem: Vec<u8>,
) -> Result<aws_sdk_cloudcontrol::config::SharedHttpClient, HarnessError> {
    let context = TlsContext::builder()
        .with_trust_store(TrustStore::empty().with_pem_certificate(pem))
        .build()
        .map_err(|error| HarnessError::invalid(error.to_string()))?;
    Ok(transport::client(
        Builder::new()
            .tls_provider(Provider::Rustls(CryptoMode::AwsLc))
            .tls_context(context)
            .build_https(),
    ))
}

pub(super) async fn connect(
    endpoint: &str,
    region: &str,
    pem: Vec<u8>,
) -> Result<Client, HarnessError> {
    let http = http_client(pem)?;
    let shared = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_cloudcontrol::config::Region::new(region.to_owned()))
        .http_client(http)
        .load()
        .await;
    let config = aws_sdk_cloudcontrol::config::Builder::from(&shared)
        .endpoint_url(endpoint)
        .retry_config(
            aws_sdk_cloudcontrol::config::retry::RetryConfig::standard().with_max_attempts(1),
        )
        .timeout_config(
            aws_sdk_cloudcontrol::config::timeout::TimeoutConfig::builder()
                .operation_timeout(Duration::from_secs(30))
                .build(),
        )
        .build();
    Ok(Client::from_conf(config))
}

pub(super) async fn list(client: &Client, type_name: &str) -> Result<Listed, HarnessError> {
    let state = (Some(String::new()), BTreeSet::new(), 0_usize);
    let pages = stream::try_unfold(state, |(token, mut seen, page)| async move {
        let Some(token) = token else { return Ok(None) };
        if page >= MAX_PAGES {
            return Err(CloudControlError::PageBound);
        }
        let token = (!token.is_empty()).then_some(token);
        let output = client
            .list_resources()
            .type_name(type_name)
            .max_results(PAGE_SIZE)
            .set_next_token(token)
            .send()
            .await
            .map_err(|error| list_error(type_name, page, error))?;
        let next = output
            .next_token()
            .filter(|token| !token.is_empty())
            .map(str::to_owned);
        if next
            .as_ref()
            .is_some_and(|token| !seen.insert(token.clone()))
        {
            return Err(CloudControlError::RepeatedToken);
        }
        let page = page.saturating_add(1);
        Ok(Some(((output, page), (next, seen, page))))
    });
    let result = pages
        .try_fold(
            BTreeSet::new(),
            |mut identities, (output, page)| async move {
                if output.resource_descriptions().len() > 100 {
                    return Err(CloudControlError::PageIdentityBound);
                }
                output
                    .resource_descriptions()
                    .iter()
                    .try_for_each(|description| {
                        let id = identifier(description.identifier(), type_name, page)?;
                        identities.insert(id.to_owned());
                        Ok::<(), CloudControlError>(())
                    })?;
                Ok(identities)
            },
        )
        .await;
    match result {
        Ok(identities) => Ok(Listed::Identities(identities)),
        Err(CloudControlError::NotListable) => Ok(Listed::NotListable),
        Err(error) => Err(error.into()),
    }
}

fn identifier<'a>(
    value: Option<&'a str>,
    type_name: &str,
    page: usize,
) -> Result<&'a str, CloudControlError> {
    let id = value
        .filter(|id| !id.is_empty())
        .ok_or(CloudControlError::MissingIdentifier)?;
    if id.len() > MAX_IDENTIFIER_BYTES {
        return Err(CloudControlError::IdentifierBytes {
            type_name: type_name.to_owned(),
            page,
            limit: MAX_IDENTIFIER_BYTES,
        });
    }
    Ok(id)
}

fn list_error(
    type_name: &str,
    page: usize,
    error: aws_sdk_cloudcontrol::error::SdkError<
        aws_sdk_cloudcontrol::operation::list_resources::ListResourcesError,
    >,
) -> CloudControlError {
    if transport::exceeded(&error) {
        return CloudControlError::ResponseBytes {
            type_name: type_name.to_owned(),
            page: page.saturating_add(1),
            limit: transport::MAX_RESPONSE_BYTES,
        };
    }
    let code = error.as_service_error().and_then(|error| error.code());
    if page == 0 && unsupported(code) {
        return CloudControlError::NotListable;
    }
    CloudControlError::Service {
        type_name: type_name.to_owned(),
        code: code.map(str::to_owned),
        source: Box::new(error),
    }
}

pub(super) fn unsupported(code: Option<&str>) -> bool {
    matches!(
        code,
        Some("TypeNotFoundException" | "UnsupportedAction" | "UnsupportedActionException")
    )
}

#[cfg(test)]
mod tests;
