pub mod body;
mod checksum;
pub(crate) mod common;
mod compression;
mod customization;
mod endpoint;
pub use compression::{CompressionPolicy, CompressionThreshold};
pub use endpoint::addressing::{
    AddressingStyle, EndpointAddressing, ResolvedAddressing, S3EndpointConfig, s3_endpoint,
};
pub mod query;
pub mod rest;

use inventory_core::{
    catalog::{CatalogError, Operation, Service},
    record::{FieldId, MapRef, RecordError},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    AwsJson10,
    AwsJson11,
    RpcV2Cbor,
    AwsQuery,
    Ec2Query,
    RestJson1,
    RestXml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestLimits {
    body_bytes: usize,
    header_bytes: usize,
    uri_bytes: usize,
}

impl RequestLimits {
    pub fn new(
        body_bytes: usize,
        header_bytes: usize,
        uri_bytes: usize,
    ) -> Result<Self, EncodeError> {
        let limits = Self {
            body_bytes,
            header_bytes,
            uri_bytes,
        };
        if body_bytes == 0 || body_bytes > inventory_core::record::MAX_RECORD_BYTES {
            return Err(EncodeError::Limit("body"));
        }
        if header_bytes == 0 || header_bytes > 65_536 {
            return Err(EncodeError::Limit("headers"));
        }
        if uri_bytes == 0 || uri_bytes > 16_384 {
            return Err(EncodeError::Limit("URI"));
        }
        Ok(limits)
    }

    pub const fn body_bytes(self) -> usize {
        self.body_bytes
    }

    pub const fn header_bytes(self) -> usize {
        self.header_bytes
    }

    pub const fn uri_bytes(self) -> usize {
        self.uri_bytes
    }
}

impl Default for RequestLimits {
    fn default() -> Self {
        Self {
            body_bytes: inventory_core::record::MAX_RECORD_BYTES,
            header_bytes: 65_536,
            uri_bytes: 16_384,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RequestContext<'a> {
    pub service: Service<'a>,
    pub operation: Operation<'a>,
    pub protocol: Protocol,
    pub input: MapRef<'a>,
    pub endpoint: &'a str,
    pub limits: RequestLimits,
    pub idempotency_token: Option<&'a str>,
    pub compression: CompressionPolicy,
    pub addressing: Option<EndpointAddressing<'a>>,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Header(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    Allocation(#[from] std::collections::TryReserveError),
    #[error("request compression failed: {0}")]
    Compression(#[source] std::io::Error),
    #[error("request exceeds {0} limit")]
    Limit(&'static str),
    #[error("unsupported request feature: {0}")]
    Unsupported(&'static str),
    #[error("invalid value for {field:?}: {reason}")]
    InvalidValue {
        field: Option<FieldId>,
        reason: &'static str,
    },
    #[error("invalid request endpoint")]
    InvalidEndpoint,
    #[error("request input is missing")]
    MissingInput,
}
